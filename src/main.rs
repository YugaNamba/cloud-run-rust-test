use anyhow::{Error, Result};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use dotenv::dotenv;
use gcp_bigquery_client::model::query_request::QueryRequest;
use serde::Serialize;
use serde_json::Value;
use std::net::SocketAddr;

#[macro_use]
extern crate dotenv_codegen;

#[tokio::main]
async fn main() {
    dotenv().ok();
    // initialize tracing
    tracing_subscriber::fmt::init();

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        .route("/get_customers", get(get_customers));

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Hello, World!"
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
    let file_path = dotenv!("GCP_SERVICE_ACCOUNT_FILE_PATH");
    let gcp_project_id = dotenv!("GCP_PROJECT_ID");
    let sa_key = yup_oauth2::read_service_account_key(file_path)
        .await
        .map_err(|e| {
            tracing::error!("Failed to read service account key: {:?}", e);
            // ここで適切なデフォルトのレスポンスを返すか、Infallibleのエラーを返す
        })
        .unwrap();
    let client = gcp_bigquery_client::Client::from_service_account_key(sa_key, true)
        .await
        .map_err(|e| {
            tracing::error!("Failed to init gcp_bigquery_client: {:?}", e);
            // ここで適切なデフォルトのレスポンスを返すか、Infallibleのエラーを返す
        })
        .unwrap();

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
            // ここで適切なデフォルトのレスポンスを返すか、Infallibleのエラーを返す
        })
        .unwrap();

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
