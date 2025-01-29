use axum::{
    extract::{Form, Path, Request, State},
    handler::HandlerWithoutStateExt,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use serde::Deserialize;
use std::{fs, net::SocketAddr};
use tower::ServiceExt;
use tower_http::{
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use askama::Template;
use sqlx::{
    sqlite::{SqlitePool, SqlitePoolOptions},
    types::time::Date,
};
use std::{env, sync::Arc};

// Each post template will be populated with the values
// located in the shared state of the handlers.
#[derive(Template)]
#[template(path = "posts.html")]
pub struct PostTemplate<'a> {
    pub post_title: &'a str,
    pub post_date: String,
    pub post_body: &'a str,
}

#[derive(Debug, sqlx::FromRow)]
struct BlogPost {
    id: i64,
    date_published: i64,
    date_last_edited: i64,
    title: String,
    content: String,
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    // "sqlite:///home/jaunger/jaunger.db"
    let url = env::var("DATABASE_URL").unwrap();
    let pool = SqlitePool::connect(url.as_str()).await?;
    let blog: Vec<BlogPost> = sqlx::query_as!(BlogPost, "SELECT * FROM blog")
        .fetch_all(&pool)
        .await?;

    println!("{}", blog[0].content);

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("{}=debug,tower_http=debug", env!("CARGO_CRATE_NAME")).into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tokio::join!(
        serve(using_serve_dir(), 3001),
        serve(using_serve_dir_with_assets_fallback(), 3002),
        serve(using_serve_dir_only_from_root_via_fallback(), 3003),
        serve(using_serve_dir_with_handler_as_service(), 3004),
        serve(two_serve_dirs(), 3005),
        serve(calling_serve_dir_from_a_handler(), 3006),
        serve(using_serve_file_from_a_route(), 3307),
    );

    Ok(())
}

async fn handler() -> impl IntoResponse {
    match fs::read_to_string("assets/tetris.html") {
        Ok(content) => Html(content),
        Err(e) => Html(format!("Error : {}", e)),
    }
}

fn using_serve_dir() -> Router {
    // serve the file in the "assets" directory under `/home`
    Router::new().nest_service("/home", ServeDir::new("assets"))
}

fn using_serve_dir_with_assets_fallback() -> Router {
    // `ServeDir` allows setting a fallback if an asset is not found
    // so with this `GET /assets/doesnt-exist.jpg` will return `index.html`
    // rather than a 404
    let serve_dir = ServeDir::new("assets").not_found_service(ServeFile::new("assets/index.html"));

    Router::new()
        .route("/foo", get(|| async { "Hi from /foo" }))
        .nest_service("/assets", serve_dir.clone())
        .fallback_service(serve_dir)
}

fn using_serve_dir_only_from_root_via_fallback() -> Router {
    // you can also serve the assets directly from the root (not nested under `/assets`)
    // by only setting a `ServeDir` as the fallback
    let serve_dir = ServeDir::new("assets").not_found_service(ServeFile::new("assets/index.html"));

    Router::new()
        .route("/foo", get(|| async { "Hi from /foo" }))
        .fallback_service(serve_dir)
}

fn using_serve_dir_with_handler_as_service() -> Router {
    async fn handle_404() -> (StatusCode, &'static str) {
        (StatusCode::NOT_FOUND, "Not found")
    }

    // you can convert handler function to service
    let service = handle_404.into_service();

    let serve_dir = ServeDir::new("assets").not_found_service(service);

    Router::new()
        .route("/foo", get(|| async { "Hi from /foo" }))
        .fallback_service(serve_dir)
}

fn two_serve_dirs() -> Router {
    // you can also have two `ServeDir`s nested at different paths
    let serve_dir_from_assets = ServeDir::new("assets");
    let serve_dir_from_dist = ServeDir::new("dist");

    Router::new()
        .nest_service("/assets", serve_dir_from_assets)
        .nest_service("/dist", serve_dir_from_dist)
}

#[allow(clippy::let_and_return)]
fn calling_serve_dir_from_a_handler() -> Router {
    // via `tower::Service::call`, or more conveniently `tower::ServiceExt::oneshot` you can
    // call `ServeDir` yourself from a handler
    Router::new().nest_service(
        "/home",
        get(|request: Request| async {
            let service = ServeDir::new("assets");
            let result = service.oneshot(request).await;
            result
        })
        .post(accept_form),
    )
}

fn using_serve_file_from_a_route() -> Router {
    Router::new().route_service("/foo", ServeFile::new("assets/tetris.html"))
}

async fn serve(app: Router, port: u16) {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app.layer(TraceLayer::new_for_http()))
        .await
        .unwrap();
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct Input {
    name: String,
    email: String,
}

async fn accept_form(Form(input): Form<Input>) {
    dbg!(&input);
}

// We use two extractors in the arguments
// Path to grab the query and State that has all our posts
async fn post(
    Path(query_title): Path<String>,
    State(state): State<Arc<Vec<BlogPost>>>,
) -> impl IntoResponse {
    // A default template or else the compiler complains
    let mut template = PostTemplate {
        post_title: "none",
        post_date: "none".to_string(),
        post_body: "none",
    };

    // We look for any post with the same title as the user's query
    for i in 0..state.len() {
        if query_title == state[i].title {
            // We found one so mutate the template variable and
            // populate it with the post that the user requested
            template = PostTemplate {
                post_title: &state[i].title,
                post_date: state[i].date_published.to_string(),
                post_body: &state[i].content,
            };
            break;
        } else {
            continue;
        }
    }

    // 404 if no title found matching the user's query
    if &template.post_title == &"none" {
        return (StatusCode::NOT_FOUND, "404 not found").into_response();
    }

    // render the template into HTML and return it to the user
    match template.render() {
        Ok(html) => Html(html).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "try again later").into_response(),
    }
}

// create an Axum template for our homepage
// index_title is the html page's title 
// index_links are the titles of the blog posts 
#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate<'a> {
    pub index_title: String,
    pub index_links: &'a Vec<String>,
}

// Then populate the template with all post titles
async fn index(State(state): State<Arc<Vec<BlogPost>>>) -> impl IntoResponse{

    let s = state.clone();
    let mut plinks: Vec<String> = Vec::new();

    for i in 0 .. s.len() {
        plinks.push(s[i].title.clone());
    }

    let template = IndexTemplate{index_title: String::from("My blog"), index_links: &plinks};

    match template.render() {
            Ok(html) => Html(html).into_response(),
         Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template. Error {}", err),
            ).into_response(),
    }
}
