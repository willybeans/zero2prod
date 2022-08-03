use std::net::TcpListener;
use zero2prod::startup::run;
use zero2prod::configuration::get_configuration;
use sqlx::{Connection, PgPool};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");

    // let port = listener.local_addr().unwrap().port();
    // // Bubble up the io::Error if we failed to bind the address
    // // Otherwise call .await on our Server
    // println!("http://127.0.0.1:{}", port);

    // run(listener)?.await
    // Panic if we can't read configuration
    let configuration = get_configuration().expect("Failed to read configuration");
    let connection = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connection to postgres");
    // We have removed the hardcoded 8000 - its now coming from our settings!
    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(address)?;
    run(listener, connection)?.await
}
