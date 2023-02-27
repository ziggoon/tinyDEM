use std::io;

use actix_web::{
    body::BoxBody,
    dev::ServiceResponse,
    http::{header::ContentType, StatusCode},
    middleware::{ErrorHandlerResponse, ErrorHandlers},
    web, App, HttpResponse, HttpServer, Result,
};

//use bcrypt::{DEFAULT_COST, hash, verify};
use r2d2_sqlite::SqliteConnectionManager;
use handlebars::Handlebars;
use serde_json::json;
 
mod helpers;

#[actix_web::main]
async fn main() -> io::Result<()> {
    let manager = SqliteConnectionManager::file("credentials.db");
    let pool = r2d2::Pool::builder().max_size(10).build(manager).unwrap();
    let test_user = helpers::user::User {
        username: String::from("tester"),
        password: String::from("test123"),
        admin: 0,
    }; 
    helpers::db::init_db(&pool.clone());
    
    let mut handlebars = Handlebars::new();
    handlebars
        .register_templates_directory(".html", "./templates")
        .unwrap();
    let handlebars_ref = web::Data::new(handlebars);

    HttpServer::new(move || {
        let pool = pool.clone();
        App::new()
            .wrap(error_handlers())
            .app_data(web::Data::new(pool))
            .app_data(handlebars_ref.clone())
            .service(web::scope("/")).configure(helpers::routes::init_routes)
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