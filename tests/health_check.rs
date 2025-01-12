use sqlx::{Connection, PgConnection};
use std::net::TcpListener;
use zero2prod::configuration::get_configuration;

#[tokio::test]
async fn health_check_works() {
    // Arrange
    let url = spawn_app();
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get(format!("{url}/health_check"))
        .send()
        .await
        .expect("Failed to send request");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    // Arrange
    let url = spawn_app();

    let configuration = get_configuration().expect("Failed to load config");
    let connection_string = configuration.database.connection_string();
    let mut connection = PgConnection::connect(&connection_string)
        .await
        .expect("Failed to connect to Postgres");
    let client = reqwest::Client::new();

    // Act
    let body = "name=John%20Doe&email=john_doe%40gmail.com";
    let response = client
        .post(format!("{url}/subscriptions"))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .unwrap();

    // Assert
    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(&mut connection)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.email, "john_doe@gmail.com");
    assert_eq!(saved.name, "John Doe");
}

#[tokio::test]
async fn subscribe_returns_a_400_for_invalid_form_data() {
    // Arrange
    let url = spawn_app();
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
            .post(format!("{url}/subscriptions"))
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
}
fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to any ports");
    let port = listener.local_addr().unwrap().port();

    let server = zero2prod::startup::run(listener).expect("Failed to start server");
    tokio::spawn(server);

    format!("http://127.0.0.1:{port}")
}
