//! tests/health_check.rs
// `tokio::test` is the testing equivalent of `tokio::main`.
// It also spares you from having to specify the `#[test]` attribute.
//
// You can inspect what code gets generated using
// `cargo expand --test health_check` (<- name of the test file)
use std::net::TcpListener;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use zero2prod::startup::run;
use zero2prod::configuration::{get_configuration, DatabaseSettings};
use uuid::Uuid;


// Launch our application in the background ~somehow~
// this is the only part of our test that is intrinsically
// tied to our code. so this can be replaced with something else (express ruby etc)
// and simply replaced with a bash command or something
// async fn spawn_app() -> std::io::Result<()> {
//     zero2prod::run().await
// }

// No .await call, therefore no need for `spawn_app` to be async now.
// We are also running tests, so it is not worth it to propagate errors:
// if we fail to perform the required setup we can just panic and crash
// all the things.
// old spawn app from before PGPOOL addition
// fn spawn_app() -> String {
//     // 0 as the port allows us to autogenerate a random port
//     let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
//     // let server = zero2prod::run("127.0.0.1:0").expect("Failed to bind random port");
//     // Launch the server as a background task
//     // tokio::spawn returns a handle to the spawned future,
//     // but we have no use for it here, hence the non-binding let

//     //lets retrieve the port assigned by OS
//     let port = listener.local_addr().unwrap().port();
//     let server = run(listener).expect("Failed to bind address");
//     let _ = tokio::spawn(server);

//     //we return the application address to the caller!
//     format!("http://127.0.0.1:{}", port)
// }

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

async fn spawn_app() -> TestApp {
    let listener = TcpListener::bind("127.0.0.1:0")
    .expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);
    let mut configuration = get_configuration().expect("Failed to read configuration.");
    configuration.database.database_name = Uuid::new_v4().to_string();
    let connection_pool = configure_database(&configuration.database).await;
    
    let server = run(listener, connection_pool.clone())
    .expect("Failed to bind address");
    let _ = tokio::spawn(server);
    TestApp {
    address,
    db_pool: connection_pool,
    }
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
  // Create database
  let mut connection = PgConnection::connect(&config.connection_string_without_db())
    .await
    .expect("Failed to connect to Postgres");
  connection
    .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
    .await
    .expect("Failed to create database.");

  // Migrate database
  let connection_pool = PgPool::connect(&config.connection_string())
    .await
    .expect("Failed to connect to Postgres.");
  sqlx::migrate!("./migrations")
    .run(&connection_pool)
    .await
    .expect("Failed to migrate the database");

  connection_pool
}


#[tokio::test]
async fn health_check_works() {
    // Arrange
    let app = spawn_app().await;
    // We need to bring in `reqwest`
    // to perform HTTP requests against our application.
    let client = reqwest::Client::new();
    // Act
    let response = client
        // use the returned app address
        .get(&format!("{}/health_check", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    //Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    //Act
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(&format!("{}/subscriptions", &app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        //Act
        let response = client
            .post(&format!("{}/subscriptions", &app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");

        //Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            // Additional customised error message on test failure
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        )
    }
}
