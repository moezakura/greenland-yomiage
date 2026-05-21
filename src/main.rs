use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    yomiage::bootstrap::init_tracing();
    yomiage::bootstrap::run().await
}
