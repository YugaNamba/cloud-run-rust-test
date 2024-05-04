use crate::AppError;
use anyhow::Result;
use axum::{extract::Path, Json};
use gcp_bigquery_client::model::{query_request::QueryRequest, query_response::ResultSet};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::init_bq_client;

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct Customer {
    customer_id: String,
    first_name: String,
    last_name: String,
    email: String,
    created_at: String,
}

#[utoipa::path(get, path = "/customers", tag = "Customer")]
pub async fn list() -> Result<Json<Vec<Customer>>, AppError> {
    let gcp_project_id = dotenv!("GCP_PROJECT_ID");

    let client = init_bq_client().await.expect("Failed to init BQ client");

    let query = format!(
        "SELECT *  FROM `{}.{}.{}` ORDER BY customer_id ASC LIMIT 1000",
        gcp_project_id, "test_tokyo", "customers"
    );
    let mut rs = client
        .job()
        .query(gcp_project_id, QueryRequest::new(&query))
        .await
        .map_err(|e| {
            tracing::error!("Failed to bq query {}: {:?}", &query, e);
            AppError(anyhow::anyhow!(e))
        })?;

    let mut customers: Vec<Customer> = Vec::new();
    while rs.next_row() {
        let customer = parse_to_customer(&rs)?;
        customers.push(customer);
    }
    Ok(Json(customers))
}

#[utoipa::path(get, path = "/customers/{id}", tag = "Customer")]
pub async fn get(Path(id): Path<String>) -> Result<Json<Customer>, AppError> {
    let gcp_project_id = dotenv!("GCP_PROJECT_ID");

    let client = init_bq_client().await.expect("Failed to init BQ client");

    let query = format!(
        "SELECT *  FROM `{}.{}.{}` WHERE customer_id = {} LIMIT 1",
        gcp_project_id, "test_tokyo", "customers", id
    );
    let mut rs = client
        .job()
        .query(gcp_project_id, QueryRequest::new(&query))
        .await
        .map_err(|e| {
            tracing::error!("Failed to bq query {}: {:?}", &query, e);
            AppError(anyhow::anyhow!(e))
        })?;

    let mut customers: Vec<Customer> = Vec::new();
    while rs.next_row() {
        let customer = parse_to_customer(&rs)?;
        customers.push(customer);
    }

    Ok(Json(customers[0].clone()))
}

fn parse_to_customer(row: &ResultSet) -> Result<Customer, AppError> {
    let email = row
        .get_string_by_name("email")
        .unwrap_or_default()
        .unwrap_or_default();
    let first_name = row
        .get_string_by_name("first_name")
        .unwrap_or_default()
        .unwrap_or_default();
    let last_name = row
        .get_string_by_name("last_name")
        .unwrap_or_default()
        .unwrap_or_default();
    let created_at = row
        .get_string_by_name("created_at")
        .unwrap_or_default()
        .unwrap_or_default();
    let customer_id = row
        .get_string_by_name("customer_id")
        .unwrap_or_default()
        .unwrap_or_default();
    let customer = Customer {
        email,
        first_name,
        last_name,
        created_at,
        customer_id,
    };
    Ok(customer)
}
