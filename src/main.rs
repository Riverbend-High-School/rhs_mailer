use dotenv::dotenv;
use lettre::message::{header, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use rocket::response::content::Json;
use rocket::routes;
use rocket::serde::{self, Deserialize, Serialize};
use serde_json::json;
use std::env;
use std::sync::{Arc, Mutex};
use std::thread::sleep;

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

mod auth;
mod models;
mod util;

#[post("/send_email", format = "application/json", data = "<email_requests>")]
pub async fn send_email(
    _token: auth::Token,
    email_requests: serde::json::Json<Vec<models::email_request>>,
) -> Json<String> {
    let mailer = MAILER.to_owned();

    let mut errors = Vec::new();

    let from: lettre::message::Mailbox = FROM_EMAIL.clone().parse().unwrap();
    let one_second = std::time::Duration::from_secs(1);

    info!("Sending {} emails", email_requests.len());

    email_requests.iter().for_each(|email_request| {
        match Message::builder()
            .from(from.clone())
            .to(email_request.to_email.parse().unwrap())
            .subject(&email_request.subject)
            .singlepart(
                SinglePart::builder()
                    .header(header::ContentType::TEXT_HTML)
                    .body(email_request.body.clone()),
            ) {
            Ok(message) => {
                if let Err(e) = mailer.send(&message) {
                    warn!("Message attempted to send with error: {}", e);
                    errors.push(email_request);
                }
            }
            Err(e) => {
                warn!("Failed to build message with error: {}", e);
                errors.push(email_request);
            }
        }
        sleep(one_second);
    });

    let num_errors = errors.len();

    if num_errors > 0 {
        warn!("{} errors occurred when sending emails!", num_errors);
        make_json_response!(500, format!("Had {} errors", num_errors), errors)
    } else {
        make_json_response!(200, "Successfully sent emails!")
    }
}

#[rocket::main]
async fn main() {
    dotenv().ok();

    match rocket::build()
        .mount("/", routes![send_email])
        .attach(util::CORS)
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
