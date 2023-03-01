use actix_web::{web, HttpResponse, HttpRequest, HttpMessage};
use actix_identity::{Identity};

use bcrypt::{DEFAULT_COST, hash, verify};

use handlebars::Handlebars;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

use std::collections::BTreeMap;
use crate::helpers::db::{get_user, insert_user};
use crate::helpers::user::{User, Login};


async fn index(hb: web::Data<Handlebars<'_>>, identity: Option<Identity>) -> HttpResponse {
    let id = match identity.map(|id| id.id()) {
        None => return HttpResponse::Unauthorized().body("auth required"),
        Some(Ok(id)) => id,
        Some(Err(_err)) => return HttpResponse::InternalServerError().body("err"),
    };
    //println!("userid: {:?}", id);

    let mut data = BTreeMap::new();
    data.insert("username".to_string(), id);

    let body = hb.render("index", &data).unwrap();
    HttpResponse::Ok().body(body)
}

async fn dashboard(hb: web::Data<Handlebars<'_>>, identity: Option<Identity>) -> HttpResponse {
    let id = match identity.map(|id| id.id()) {
        None => return HttpResponse::Unauthorized().body("auth required"),
        Some(Ok(id)) => id,
        Some(Err(_err)) => return HttpResponse::InternalServerError().body("err"),
    };
    //println!("userid: {:?}", id);
    let mut data = BTreeMap::new();
    data.insert("name".to_string(), id);
    let body = hb.render("dashboard", &data).unwrap();
    HttpResponse::Ok().body(body) 
}

async fn chart(hb: web::Data<Handlebars<'_>>, identity: Option<Identity>) -> HttpResponse {
    let id = match identity.map(|id| id.id()) {
        None => return HttpResponse::Unauthorized().body("auth required"),
        Some(Ok(id)) => id,
        Some(Err(_err)) => return HttpResponse::InternalServerError().body("err"),
    };
    let body = hb.render("charts", &()).unwrap();
    HttpResponse::Ok().body(body)
}

async fn form(hb: web::Data<Handlebars<'_>>, identity: Option<Identity>) -> HttpResponse {
    let id = match identity.map(|id| id.id()) {
        None => return HttpResponse::Unauthorized().body("auth required"),
        Some(Ok(id)) => id,
        Some(Err(_err)) => return HttpResponse::InternalServerError().body("err"),
    };
    let body = hb.render("forms", &()).unwrap();
    HttpResponse::Ok().body(body)
}

async fn table(hb: web::Data<Handlebars<'_>>, identity: Option<Identity>) -> HttpResponse {
    let id = match identity.map(|id| id.id()) {
        None => return HttpResponse::Unauthorized().body("auth required"),
        Some(Ok(id)) => id,
        Some(Err(_err)) => return HttpResponse::InternalServerError().body("err"),
    };
    let body = hb.render("tables", &()).unwrap();
    HttpResponse::Ok().body(body)
}

async fn login_get(hb: web::Data<Handlebars<'_>>, _req: HttpRequest) -> HttpResponse {
    let body = hb.render("login", &()).unwrap();
    HttpResponse::Ok().body(body)
}

async fn login_post(hb: web::Data<Handlebars<'_>>, form: web::Form<Login>, pool: web::Data<Pool<SqliteConnectionManager>>, _req: HttpRequest) -> HttpResponse {
    println!("Login post req");
    let form_data = Login {
        username: form.username.clone(),
        password: form.password.clone(),
    };

    let user = match get_user(&pool, &form_data.username) {
        Some(user) => user,
        None => {
            println!("No user");
            let html = hb.render("login", &("Invalid username or password.")).unwrap();
            return HttpResponse::Unauthorized().body(html);
        }
    };

    // Verify the password
    if verify(&form_data.password, &user.password).unwrap() {
        Identity::login(&_req.extensions(), form_data.username).unwrap();
        HttpResponse::Found()
            .append_header(("Location", "/"))
            .finish()    
    } else {
        // Password is incorrect, render the login form again with an error message
        let html = hb.render("login", &("Invalid username or password.")).unwrap();
        HttpResponse::Unauthorized().body(html)
    }
}

async fn register_get(hb: web::Data<Handlebars<'_>>, _req: HttpRequest) -> HttpResponse {
    let body = hb.render("register", &()).unwrap();
    HttpResponse::Ok().body(body)
}

async fn register_post(pool: web::Data<Pool<SqliteConnectionManager>>, form: web::Form<User>, _req: HttpRequest) -> HttpResponse {
    let user = User {
        username: form.username.clone(),
        password: hash(form.password.clone(), DEFAULT_COST).unwrap(),
        admin: form.admin,
    };

    let result = insert_user(&pool, user);

    match result {
        Ok(_) => HttpResponse::Ok().body("Registration successful!"),
        Err(_) => HttpResponse::BadRequest().body("Username already taken"),
    }
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/login")
        .route(web::get().to(login_get))
        .route(web::post().to(login_post))
    );
    cfg.service(web::resource("/register")
        .route(web::get().to(register_get))
        .route(web::post().to(register_post))
    );
    cfg.service(web::resource("/dashboard")
        .route(web::get().to(dashboard))
    );
    cfg.service(web::resource("/charts")
        .route(web::get().to(chart))
    );
    cfg.service(web::resource("/forms")
        .route(web::get().to(form))
    );
    cfg.service(web::resource("/tables")
        .route(web::get().to(table))
    );
    cfg.service(web::resource("/")
        .route(web::get().to(index))
    );
}
