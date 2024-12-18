use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;

// Structure to store URL mappings
struct AppState {
    url_database: Mutex<HashMap<String, String>>,
}

#[derive(Deserialize)]
struct UrlRequest {
    long_url: String,
}

#[derive(Serialize)]
struct UrlResponse {
    short_url: String,
}

// Generate a random short code
fn generate_short_code() -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(6)
        .map(char::from)
        .collect()
}

// Handler for creating short URLs
async fn shorten_url(
    data: web::Data<AppState>,
    url_req: web::Json<UrlRequest>,
) -> impl Responder {
    let short_code = generate_short_code();
    let mut url_database = data.url_database.lock().unwrap();
    
    url_database.insert(short_code.clone(), url_req.long_url.clone());
    
    HttpResponse::Ok().json(UrlResponse {
        short_url: format!("http://localhost:8080/{}", short_code),
    })
}

// Handler for redirecting short URLs
async fn redirect(
    data: web::Data<AppState>,
    path: web::Path<String>,
) -> impl Responder {
    let url_database = data.url_database.lock().unwrap();
    
    if let Some(long_url) = url_database.get(&path.into_inner()) {
        HttpResponse::Found()
            .append_header(("Location", long_url.clone()))
            .finish()
    } else {
        HttpResponse::NotFound().body("URL not found")
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let app_state = web::Data::new(AppState {
        url_database: Mutex::new(HashMap::new()),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .route("/shorten", web::post().to(shorten_url))
            .route("/{short_code}", web::get().to(redirect))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
