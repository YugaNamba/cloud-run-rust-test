use crate::{AppError, BQ_CLIENT};
use axum::Json;
use gcp_bigquery_client::model::query_request::QueryRequest;
use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Serialize)]
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

    let customers = parse_customers(rs.query_response().rows.as_ref());

    Ok(Json(customers))
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