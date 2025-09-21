use crate::error::handler::TaskExecuteError;
use crate::handlers::common::get_error;
use custom_logger as log;
use serde_derive::{Deserialize, Serialize};
use std::env;

#[derive(Deserialize, Serialize, Debug)]
struct Claims {
    user: String,
    iss: String,
    sub: String,
    aud: String,
    exp: u64,
}

pub fn validate_jwt(token: String) -> Result<(), TaskExecuteError> {
    let mut validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256);
    validation.set_audience(&vec!["ai-workflow"]);
    validation.set_issuer(&vec!["https://ai-workflow"]);
    validation.set_required_spec_claims(&["iss", "aud", "exp", "sub", "user"]);
    let jwt_secret = match env::var("JWT_SECRET") {
        Ok(var) => var,
        Err(_) => "secret".to_string(),
    };
    let secret = jsonwebtoken::DecodingKey::from_secret(jwt_secret.as_bytes());
    // decode token
    let jwt = jsonwebtoken::decode::<Claims>(&token, &secret, &validation)
        .map_err(|e| get_error(e.to_string()))?;
    log::trace!("jwt details {:?}", jwt);
    Ok(())
}
