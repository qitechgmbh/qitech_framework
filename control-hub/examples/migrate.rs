#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    control_hub::migrate("http://localhost:8123").await
}
