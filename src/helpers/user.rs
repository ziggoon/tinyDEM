use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub username: String,
    pub password: String,
    #[serde(default)]
    pub admin: u8,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Login {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Claims {
    pub username: String,
    pub tstamp: usize,
}