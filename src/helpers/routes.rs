use actix_web::{get, post, web, HttpResponse, HttpRequest, http::header};
use bcrypt::{DEFAULT_COST, hash, verify};
use handlebars::Handlebars;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use chrono::{DateTime, Duration, Utc};

use crate::helpers::db::{get_user, insert_user};
use crate::helpers::user::{User, Login, Claims};
use crate::agent::snmpwalk::query;

#[get("/home")]
async fn index(hb: web::Data<Handlebars<'_>>) -> HttpResponse {
    let body = hb.render("index", &String::from("anon")).unwrap();
    HttpResponse::Ok().body(body)
}

#[get("/auth")]
async fn authed(hb: web::Data<Handlebars<'_>>, _req: HttpRequest) -> HttpResponse {
    query();
    let body = hb.render("authed", &()).unwrap();
    HttpResponse::Ok().body(body)
}

#[get("/login")]
async fn login_get(hb: web::Data<Handlebars<'_>>) -> HttpResponse {
    let body = hb.render("login", &()).unwrap();

    HttpResponse::Ok().body(body)
}

#[post("/login")]
async fn login_post(hb: web::Data<Handlebars<'_>>, form: web::Form<Login>, pool: web::Data<Pool<SqliteConnectionManager>>) -> HttpResponse {
    let form_data = Login {
        username: form.username.clone(),
        password: form.password.clone(),
    };

    let user = match get_user(&pool, &form_data.username) {
        Some(user) => user,
        None => {
            let html = hb.render("login", &("Invalid username or password.")).unwrap();
            return HttpResponse::Unauthorized().body(html);
        }
    };

    // Verify the password
    if verify(&form_data.password, &user.password).unwrap() {
        // Password is correct, redirect to the dashboard
        let secret = String::from("s3cr3t_k3y");
        let mut _date: DateTime<Utc> = Utc::now() + Duration::hours(1);

        let claims = Claims {
            username: form_data.password.clone(),
            tstamp: _date.timestamp() as usize,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(&secret.as_bytes()),
        ).unwrap();

        HttpResponse::Found()
            .header(header::AUTHORIZATION, format!("Bearer {}", token))
            .header("Location", "/auth")
            .finish()
            
    } else {
        // Password is incorrect, render the login form again with an error message
        let html = hb.render("login", &("Invalid username or password.")).unwrap();
        HttpResponse::Unauthorized().body(html)
    }
}

#[get("/register")]
async fn register_get(hb: web::Data<Handlebars<'_>>) -> HttpResponse {
    let body = hb.render("register", &()).unwrap();

    HttpResponse::Ok().body(body)
}

#[post("/register")]
async fn register_post(pool: web::Data<Pool<SqliteConnectionManager>>, form: web::Form<User>) -> HttpResponse {
    let user = User {
        username: form.username.clone(),
        password: hash(form.password.clone(), DEFAULT_COST).unwrap(),
        admin: form.admin.clone(),
    };

    let result = insert_user(&pool, user);

    match result {
        Ok(_) => HttpResponse::Ok().body("Registration successful!"),
        Err(_) => HttpResponse::BadRequest().body("Username already taken"),
    }
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(index);
    cfg.service(authed);
    cfg.service(login_get);
    cfg.service(login_post);
    cfg.service(register_get);
    cfg.service(register_post);
}