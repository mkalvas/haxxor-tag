#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    haxxor_tag::actor::run().await
}
