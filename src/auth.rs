use rocket::http::Status;
use rocket::request::{self, FromRequest, Request};

#[derive(Clone, PartialEq)]
pub struct Token(pub String);

#[derive(Debug)]
pub enum ApiTokenError {
    Missing,
    Invalid,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Token {
    type Error = ApiTokenError;

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let key = req.query_value::<&str>("token");

        match key {
            Some(r) => match r {
                Ok(s) if validate_token(s.to_owned()) => {
                    request::Outcome::Success(Token(s.to_owned()))
                }
                _ => request::Outcome::Failure((Status::Unauthorized, ApiTokenError::Invalid)),
            },
            None => request::Outcome::Failure((Status::Unauthorized, ApiTokenError::Invalid)),
        }
    }
}

pub fn validate_token(token_val: String) -> bool {
    info!("Validating token: {}", token_val);
    let auth = std::env::var("AUTH_TOKEN").expect("AUTH_TOKEN must be set");
    auth == token_val
}
