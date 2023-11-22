#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    xor_tag::server::serve().await
}
