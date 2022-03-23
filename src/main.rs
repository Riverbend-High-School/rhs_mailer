use rocket::routes;
use rocket::serde::{self, Deserialize, Serialize};

use dotenv::dotenv;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use rayon::prelude::*;
use std::env;
use std::sync::{Arc, Mutex};

#[macro_use]
extern crate rocket;

#[macro_use]
extern crate lazy_static;

lazy_static! {
    static ref FROM_EMAIL: String = {
        dotenv().ok();
        env::var("FROM_EMAIL").expect("FROM_EMAIL must be set")
    };
    static ref CREDENTIALS: Credentials = {
        dotenv().ok();
        let username = env::var("SMTP_USERNAME").expect("SMTP_USERNAME must be set");
        let password = env::var("SMTP_PASSWORD").expect("SMTP_PASSWORD must be set");
        Credentials::new(username, password)
    };
    static ref SERVER: String = {
        dotenv().ok();
        env::var("SMTP_SERVER").expect("SMTP_SERVER must be set")
    };
    static ref MAILER: SmtpTransport = SmtpTransport::relay(&SERVER)
        .unwrap()
        .credentials(CREDENTIALS.clone())
        .build();
}

mod models;

#[post("/send_email", format = "application/json", data = "<email_requests>")]
pub async fn send_email(
    email_requests: serde::json::Json<Vec<models::email_request>>,
) -> Option<()> {
    let mailer = MAILER.to_owned();

    let errors = Arc::new(Mutex::new(Vec::new()));

    email_requests.par_iter().for_each(|email_request| {
        match Message::builder()
            .from(FROM_EMAIL.clone().parse().unwrap())
            .to(email_request.to_email.parse().unwrap())
            .subject(&email_request.subject)
            .body(email_request.body.clone())
        {
            Ok(message) => {
                if let Err(e) = mailer.send(&message) {
                    println!("Message attempted to send with error: {}", e);
                    errors.lock().unwrap().push(email_request);
                }
            }
            Err(e) => {
                println!("Failed to build message with error: {}", e);
                errors.lock().unwrap().push(email_request);
            }
        }
    });

    let num_errors = errors.lock().unwrap().len();

    if num_errors > 0 {
        println!("{} errors occurred", num_errors);
        None
    } else {
        Some(())
    }
}

#[rocket::main]
async fn main() {
    match rocket::build()
        .mount("/", routes![send_email])
        .launch()
        .await
    {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }
}
