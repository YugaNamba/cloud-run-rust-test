use anyhow::{Error, Result};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use dotenv::dotenv;
use gcp_bigquery_client::{model::query_request::QueryRequest, Client};
use once_cell::sync::Lazy;
use serde::Serialize;
use serde_json::Value;
use std::net::SocketAddr;
use tokio::sync::Mutex;

mod root;

#[macro_use]
extern crate dotenv_codegen;

// BQ_CLIENTはオプションのMutexでラップされます。
static BQ_CLIENT: Lazy<Mutex<Option<Client>>> = Lazy::new(|| Mutex::new(None));

#[tokio::main]
async fn main() {
    dotenv().ok();
    // initialize tracing
    tracing_subscriber::fmt::init();

    // BQ_CLIENTの初期化
    {
        let mut bq_client = BQ_CLIENT.lock().await;
        *bq_client = Some(init_bq_client().await.expect("Failed to init BQ client"));
    }

    // build our application with a route
    let app = Router::new()
        .route("/", get(root::root))
        .route("/get_customers", get(get_customers));

    // run our app with hyper
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn init_bq_client() -> Result<Client, String> {
    let file_path = dotenv!("GCP_SERVICE_ACCOUNT_FILE_PATH");
    let sa_key = yup_oauth2::read_service_account_key(file_path)
        .await
        .map_err(|e| {
            tracing::error!("Failed to read service account key: {:?}", e);
            "Failed to read service account key".to_string()
        })?;

    let client = Client::from_service_account_key(sa_key, true)
        .await
        .map_err(|e| {
            tracing::error!("Failed to init gcp_bigquery_client: {:?}", e);
            "Failed to init gcp_bigquery_client".to_string()
        })?;

    Ok(client)
}

#[derive(Debug)]
struct AppError(Error);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        tracing::error!("Application error: {:#}", self.0);
        (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong").into_response()
    }
}

#[derive(Debug, Serialize)]
pub struct Customer {
    id: String,
    first_name: String,
    last_name: String,
    email: String,
    created_at: String,
}

async fn get_customers() -> Result<Json<Vec<Customer>>, AppError> {
    let gcp_project_id = dotenv!("GCP_PROJECT_ID");

    // BQ_CLIENTのMutexをロックして中のClientを取得
    let client = BQ_CLIENT.lock().await;
    let client = client.as_ref().expect("BQ client not initialized");

    let query = format!(
        "SELECT *  FROM `{}.{}.{}` ORDER BY customer_id ASC LIMIT 1000",
        gcp_project_id, "test_tokyo", "customers"
    );
    let rs = client
        .job()
        .query(gcp_project_id, QueryRequest::new(&query))
        .await
        .map_err(|e| {
            tracing::error!("Failed to bq query {}: {:?}", &query, e);
            AppError(anyhow::anyhow!(e))
        })?;

    let mut customers: Vec<Customer> = vec![];
    if let Some(rows) = &rs.query_response().rows {
        for row in rows {
            if let Some(columns) = &row.columns {
                let id = get_cleaned_value(&columns[0].value);
                let first_name = get_cleaned_value(&columns[1].value);
                let last_name = get_cleaned_value(&columns[2].value);
                let email = get_cleaned_value(&columns[3].value);
                let created_at = get_cleaned_value(&columns[4].value);

                let customer = Customer {
                    id,
                    first_name,
                    last_name,
                    email,
                    created_at,
                };

                customers.push(customer);
            }
        }
    }
    Ok(Json(customers))
}

fn get_cleaned_value(opt: &Option<Value>) -> String {
    opt.clone()
        .unwrap_or_default()
        .to_string()
        .trim_matches('\"')
        .to_string()
}
