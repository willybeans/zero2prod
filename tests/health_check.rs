//! tests/health_check.rs
// `tokio::test` is the testing equivalent of `tokio::main`.
// It also spares you from having to specify the `#[test]` attribute.
//
// You can inspect what code gets generated using
// `cargo expand --test health_check` (<- name of the test file)
use std::net::TcpListener;

#[tokio::test]
async fn health_check_works() {
    // Arrange
    let address = spawn_app();
    // We need to bring in `reqwest`
    // to perform HTTP requests against our application.
    let client = reqwest::Client::new();
    // Act
    let response = client
        // use the returned app address
        .get(&format!("{}/health_check", &address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}
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
fn spawn_app() -> String {
    // 0 as the port allows us to autogenerate a random port
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    // let server = zero2prod::run("127.0.0.1:0").expect("Failed to bind random port");
    // Launch the server as a background task
    // tokio::spawn returns a handle to the spawned future,
    // but we have no use for it here, hence the non-binding let

    //lets retrieve the port assigned by OS
    let port = listener.local_addr().unwrap().port();
    let server = zero2prod::run(listener).expect("Failed to bind address");
    let _ = tokio::spawn(server);

    //we return the application address to the caller!
    format!("http://127.0.0.1:{}", port)
}
