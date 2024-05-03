#[utoipa::path(get, path = "/", tag = "Root")]
pub async fn root() -> &'static str {
    "Hello, World!"
}
