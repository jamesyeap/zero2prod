use secrecy::ExposeSecret;
use sqlx::PgPool;
use std::net::TcpListener;
use zero2prod::configuration::get_configuration;
use zero2prod::startup::run;
use zero2prod::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration");

    // init TCP listener
    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(address).unwrap_or_else(|_| {
        panic!(
            "Failed to bind to specified port {}",
            configuration.application_port
        )
    });

    // init connection to Postgres database
    let connection_string = configuration.database.connection_string();
    let connection_pool = PgPool::connect(&connection_string.expose_secret())
        .await
        .expect("Failed to connect to Postgres");

    let server = run(listener, connection_pool)?;
    server.await
}
