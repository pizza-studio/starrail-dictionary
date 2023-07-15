use crud::delete_all_dictionary;
use tokio;
use tracing_unwrap::ResultExt;
use update_data::update_all_data;

use tracing::{debug, error, info};
use tracing_appender::{non_blocking, rolling};
use tracing_subscriber::{
    filter::EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt, Registry,
};

#[tokio::main]
async fn main() {
    init_tracing();

    info!("Start updating...");
    update_all_data().await.unwrap_or_log()
}

fn init_tracing() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new("info"));

    let formatting_layer = fmt::layer()
        .pretty()
        .with_file(false)
        .with_writer(std::io::stderr);

    let file_appender = rolling::hourly("./logs/update_data", "app.log");
    let (non_blocking_appender, _guard) = non_blocking(file_appender);
    let file_layer = fmt::layer()
        .compact()
        .with_ansi(false)
        .with_writer(non_blocking_appender);

    Registry::default()
        .with(env_filter)
        .with(formatting_layer)
        .with(file_layer)
        .init();
}