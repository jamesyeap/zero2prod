use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::net::TcpListener;
use uuid::Uuid;
use zero2prod::configuration::{get_configuration, DatabaseSettings, Settings};

#[tokio::test]
async fn health_check_works() {
    // Arrange
    let test_app = spawn_app().await;
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get(format!("{}/health_check", test_app.address))
        .send()
        .await
        .expect("Failed to send request");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());

    // Clean Up
    test_app.cleanup().await;
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    // Arrange
    let test_app = spawn_app().await;
    let client = reqwest::Client::new();

    // Act
    let body = "name=John%20Doe&email=john_doe%40gmail.com";
    let response = client
        .post(format!("{}/subscriptions", test_app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .unwrap();

    // Assert
    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(&test_app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.email, "john_doe@gmail.com");
    assert_eq!(saved.name, "John Doe");

    // Clean Up
    test_app.cleanup().await;
}

#[tokio::test]
async fn subscribe_returns_a_400_for_invalid_form_data() {
    // Arrange
    let test_app = spawn_app().await;
    let client = reqwest::Client::new();

    // Act
    let negative_test_cases = vec![
        ("email=john_doe%40gmail.com", "missing the name"),
        ("name=John%20Doe", "missing the email"),
        ("", "missing both then name and email"),
    ];

    // Assert
    for (body, description) in negative_test_cases {
        let response = client
            .post(format!("{}/subscriptions", test_app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .unwrap();

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 when the payload was missing the {description}"
        );
    }

    // Clean Up
    test_app.cleanup().await;
}

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
    database_name: String, // save name of temporary database for clean up
}
async fn spawn_app() -> TestApp {
    let mut configuration = get_configuration().expect("Failed to load config");
    configuration.database.database_name = Uuid::new_v4().to_string();

    let db_pool = create_test_database(&mut configuration.database).await;

    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to any ports");
    let port = listener.local_addr().unwrap().port();

    let server =
        zero2prod::startup::run(listener, db_pool.clone()).expect("Failed to start server");
    tokio::spawn(server);

    TestApp {
        address: format!("http://127.0.0.1:{port}"),
        db_pool,
        database_name: configuration.database.database_name,
    }
}

async fn create_test_database(config: &DatabaseSettings) -> PgPool {
    let connection_string = config.connection_string_without_db();
    let mut connection = PgConnection::connect(&connection_string)
        .await
        .expect("Failed to connect to Postgres");

    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database.");

    let connection_pool = PgPool::connect(&config.connection_string())
        .await
        .expect("Failed to connect to Postgres");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");

    connection_pool
}

impl TestApp {
    pub async fn cleanup(&self) {
        let config = get_configuration().expect("Failed to read configuration");

        self.db_pool.close().await;

        let connection_string = config.database.connection_string_without_db();
        let mut connection = PgConnection::connect(&connection_string)
            .await
            .expect("Failed to connect to Postgres");

        connection
            .execute(format!(r#"DROP DATABASE "{}";"#, self.database_name).as_str())
            .await
            .unwrap_or_else(|e| {
                panic!(
                    r#"Failed to drop database: {} due to error: {}"#,
                    self.database_name, e
                )
            });
    }
}
