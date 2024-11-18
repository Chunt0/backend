use backend::config::get_config;
use backend::startup::run;
use backend::telemetry::{get_subscriber, init_subscriber};
use core::panic;
use sqlx::postgres::PgPoolOptions;
use std::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let subscriber = get_subscriber("backend".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);
    let config = match get_config() {
        Ok(config) => config,
        Err(e) => panic!("Failed to read config.yaml... {e}"),
    };

    let connection_pool = PgPoolOptions::new().connect_lazy_with(config.database.with_db());

    let address = format!("{}:{}", config.application.host, config.application.port);

    let listener = match TcpListener::bind(address) {
        Ok(listener) => listener,
        Err(e) => panic!("Failed to bid to address... {e}"),
    };

    run(listener, connection_pool).expect("Failed to run").await
}
