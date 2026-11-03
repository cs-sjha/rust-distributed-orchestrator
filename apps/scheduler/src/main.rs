#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().with_env_filter("info").init();
    tracing::info!("scheduler service scaffold started; durable claiming is a future milestone");
}
