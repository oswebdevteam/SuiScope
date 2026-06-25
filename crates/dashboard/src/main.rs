#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    if let Err(e) = suiscope_dashboard::start(7731).await {
        eprintln!("Dashboard server error: {e}");
        std::process::exit(1);
    }
}
