use std::io;

use actix_web::{
    body::BoxBody,
    dev::{ServiceResponse},
    cookie::{time::Duration, Key},
    http::{header::ContentType, StatusCode},
    middleware::{ErrorHandlerResponse, ErrorHandlers},
    web, middleware, App, HttpResponse, HttpServer, Result,
};
use actix_session::{config::PersistentSession, storage::CookieSessionStore, SessionMiddleware};
use actix_identity::{IdentityMiddleware};
use actix_files::Files;

//use bcrypt::{DEFAULT_COST, hash, verify};
use r2d2_sqlite::SqliteConnectionManager;
use handlebars::Handlebars;
use serde_json::json;
 
mod helpers;
mod agent;

const TEN_MINUTES: Duration = Duration::minutes(10);

#[actix_web::main]
async fn main() -> io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let manager = SqliteConnectionManager::file("credentials.db");
    let pool = r2d2::Pool::builder().max_size(10).build(manager).unwrap();
     
    helpers::db::init_db(&pool.clone());
    
    let mut handlebars = Handlebars::new();
    handlebars
        .register_templates_directory(".html", "./templates")
        .unwrap();
    let handlebars_ref = web::Data::new(handlebars);
    let secret_key = Key::generate();

    HttpServer::new(move || {
        let pool = pool.clone();
        App::new()
            .wrap(error_handlers())
            .app_data(web::Data::new(pool))
            .app_data(handlebars_ref.clone())
            .configure(helpers::routes::init_routes)
            .service(Files::new("/css", "./static/css").show_files_listing())
            .service(Files::new("/js", "./static/js").show_files_listing())
            .service(Files::new("/img", "./static/img").show_files_listing())
            .service(Files::new("/data", "./static/data").show_files_listing())
            .service(Files::new("/vendor", "./static/vendor").show_files_listing())
            .wrap(IdentityMiddleware::default())
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), secret_key.clone())
                    .cookie_name("auth".to_owned())
                    .cookie_secure(false)
                    .session_lifecycle(PersistentSession::default().session_ttl(TEN_MINUTES))
                    .build(),
            )
            .wrap(middleware::NormalizePath::trim())
            .wrap(middleware::Logger::default())
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

// Custom error handlers, to return HTML responses when an error occurs.
fn error_handlers() -> ErrorHandlers<BoxBody> {
    ErrorHandlers::new().handler(StatusCode::NOT_FOUND, not_found)
}

// Error handler for a 404 Page not found error.
fn not_found<B>(res: ServiceResponse<B>) -> Result<ErrorHandlerResponse<BoxBody>> {
    let response = get_error_response(&res, "Page not found");
    Ok(ErrorHandlerResponse::Response(ServiceResponse::new(
        res.into_parts().0,
        response.map_into_left_body(),
    )))
}

// Generic error handler.
fn get_error_response<B>(res: &ServiceResponse<B>, error: &str) -> HttpResponse<BoxBody> {
    let request = res.request();

    // Provide a fallback to a simple plain text response in case an error occurs during the
    // rendering of the error page.
    let fallback = |e: &str| {
        HttpResponse::build(res.status())
            .content_type(ContentType::plaintext())
            .body(e.to_string())
    };

    let hb = request
        .app_data::<web::Data<Handlebars>>()
        .map(|t| t.get_ref());
    match hb {
        Some(hb) => {
            let data = json!({
                "error": error,
                "status_code": res.status().as_str()
            });
            let body = hb.render("error", &data);

            match body {
                Ok(body) => HttpResponse::build(res.status())
                    .content_type(ContentType::html())
                    .body(body),
                Err(_) => fallback(error),
            }
        }
        None => fallback(error),
    }
}