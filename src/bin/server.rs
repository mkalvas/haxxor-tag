#[actix_web::main]
pub async fn main() -> std::io::Result<()> {
    xor_tag::server::serve().await
}
