#[macro_use]
extern crate diesel;
#[macro_use]
extern crate custom_derive;
#[macro_use]
extern crate enum_derive;

use dotenv;
use std::env;
use std::error::Error;

mod services;
mod models;
mod transactions;
mod schema;
mod regex;
mod postgres;
mod proto;
mod time;
mod token;

const ERR_NO_PORT: &str = "Service port must be set";
const ENV_SERVICE_PORT: &str = "SERVICE_PORT";
const DEFAULT_IP: &str = "127.0.0.1";

use std::sync::Once;

static INIT: Once = Once::new();

pub fn initialize() {
    INIT.call_once(|| {
        dotenv::dotenv().ok(); // seting up environment variables
        postgres::open_stream(); // checking connectivity
    });
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    initialize();
    let port = env::var(ENV_SERVICE_PORT)
        .expect(ERR_NO_PORT);

   let addr = format!("{}:{}", DEFAULT_IP, port);
   services::session::start_server(addr).await?;
   Ok(())
}