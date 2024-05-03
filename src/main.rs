use anyhow::{Error, Result};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use dotenv::dotenv;
use gcp_bigquery_client::Client;
use tokio::net::TcpListener;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

mod customers;
mod root;

#[macro_use]
extern crate dotenv_codegen;

#[tokio::main]
async fn main() {
    dotenv().ok();
    // initialize tracing
    tracing_subscriber::fmt::init();
    // build our application with a route
    let app = Router::new()
        .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/", get(root::root))
        .route("/customers", get(customers::list))
        .route("/customers/:id", get(customers::get));

    // run our app with hyper
    println!("listening on http://localhost:8080");
    axum::serve(TcpListener::bind("0.0.0.0:8080").await.unwrap(), app)
        .await
        .unwrap();
}

pub async fn init_bq_client() -> Result<Client, String> {
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
pub struct AppError(Error);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        tracing::error!("Application error: {:#}", self.0);
        (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong").into_response()
    }
}

#[derive(OpenApi)]
#[openapi(
    paths(
        root::root,
        customers::list,
        customers::get
    ),
    components(schemas(
        customers::Customer
    )),
    tags((name = "Customer"))
)]
struct ApiDoc;
