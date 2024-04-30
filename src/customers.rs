use crate::AppError;
use axum::{extract::Path, Json};
use gcp_bigquery_client::model::query_request::QueryRequest;
use serde::Serialize;
use serde_json::Value;
use crate::init_bq_client;
#[derive(Debug, Serialize, Clone)]
pub struct Customer {
    id: String,
    first_name: String,
    last_name: String,
    email: String,
    created_at: String,
}

pub async fn list() -> Result<Json<Vec<Customer>>, AppError> {
    let gcp_project_id = dotenv!("GCP_PROJECT_ID");

    // BQ_CLIENTのMutexをロックして中のClientを取得
    let client = init_bq_client().await.expect("Failed to init BQ client");

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

    let customers = parse_customers(rs.query_response().rows.as_ref());

    Ok(Json(customers))
}

pub async fn get(Path(id): Path<String>) -> Result<Json<Customer>, AppError> {
    println!("id: {}", id);
    let gcp_project_id = dotenv!("GCP_PROJECT_ID");

    // BQ_CLIENTのMutexをロックして中のClientを取得
    let client = init_bq_client().await.expect("Failed to init BQ client");

    let query = format!(
        "SELECT *  FROM `{}.{}.{}` WHERE customer_id = {} LIMIT 1",
        gcp_project_id, "test_tokyo", "customers", id
    );
    let rs = client
        .job()
        .query(gcp_project_id, QueryRequest::new(&query))
        .await
        .map_err(|e| {
            tracing::error!("Failed to bq query {}: {:?}", &query, e);
            AppError(anyhow::anyhow!(e))
        })?;

    let customers = parse_customers(rs.query_response().rows.as_ref());

    Ok(Json(customers[0].clone()))
}

fn parse_customers(
    rows: Option<&Vec<gcp_bigquery_client::model::table_row::TableRow>>,
) -> Vec<Customer> {
    rows.unwrap_or(&Vec::new())
        .iter()
        .filter_map(|row| row.columns.as_ref())
        .map(|columns| {
            let id = get_cleaned_value(&columns[0].value);
            let first_name = get_cleaned_value(&columns[1].value);
            let last_name = get_cleaned_value(&columns[2].value);
            let email = get_cleaned_value(&columns[3].value);
            let created_at = get_cleaned_value(&columns[4].value);

            Customer {
                id,
                first_name,
                last_name,
                email,
                created_at,
            }
        })
        .collect()
}

fn get_cleaned_value(opt: &Option<Value>) -> String {
    opt.as_ref()
        .map_or_else(String::new, |v| v.to_string().trim_matches('"').to_string())
}
