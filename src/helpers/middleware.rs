use actix_web::{HttpResponse};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};

use crate::helpers::user::Claims;

pub fn check_token(_token: &str) -> Result<(), HttpResponse> {
    let secret = String::from("s3cr3t_k3y");
    let _decode = decode::<Claims>(
        _token,
        &DecodingKey::from_secret(&secret.as_bytes()),
        &Validation::new(Algorithm::HS256),
    );
    match _decode {
        Ok(_decode) => Ok(()),
        Err(_) => Err(HttpResponse::Unauthorized().body("invalid token!")),
    } 
}