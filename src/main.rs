use axum::{
    extract::{Form, Path, Request, State},
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
    sqlite::SqlitePool,
    types::time::Date,
};
use std::{env, sync::Arc};

// Our custom Askama filter to replace spaces with dashes in the title
mod filters {

    // now in our templates with can add tis filter e.g. {{ post_title|rmdash }}
    pub fn rmdashes(title: &str) -> askama::Result<String> {
        Ok(title.replace("-", " ").into())
    }
}

// create an Axum template for our homepage
// index_title is the html page's title
// index_links are the titles of the blog posts
#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate<'a> {
    pub title: &'a str,
    pub index_links: &'a Vec<String>,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct Input {
    name: String,
    email: String,
}

// Each post template will be populated with the values
// located in the shared state of the handlers.
#[derive(Template)]
#[template(path = "posts.html")]
pub struct PostTemplate<'a> {
    pub title: &'a str,
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
        .await
        .unwrap();
    println!("{} {}", blog[0].title, blog[0].content);
    let shared_state = Arc::new(blog);

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("{}=debug,tower_http=debug", env!("CARGO_CRATE_NAME")).into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tokio::join!(
        serve(calling_serve_dir_from_a_handler(shared_state), 4000),
    );

    Ok(())
}

#[allow(clippy::let_and_return)]
fn calling_serve_dir_from_a_handler(shared_state: Arc<Vec<BlogPost>>) -> Router {
    // via `tower::Service::call`, or more conveniently `tower::ServiceExt::oneshot` you can
    // call `ServeDir` yourself from a handler
    Router::new()
        .route("/", get(index))
        .route("/post/{query_title}", get(post))
        .with_state(shared_state)
        .nest_service(
            "/assets",
            get(|request: Request| async {
                let service = ServeDir::new("assets");
                let result = service.oneshot(request).await;
                result
            })
            .post(accept_form),
        )
}

async fn serve(app: Router, port: u16) {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app.layer(TraceLayer::new_for_http()))
        .await
        .unwrap();
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
    tracing::debug!("HI");
    // A default template or else the compiler complains
    let mut template = PostTemplate {
        title: "none",
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
                title: "Blog", 
                post_title: &state[i].title,
                post_date: state[i].date_published.to_string(),
                post_body: &state[i].content,
            };
            break;
        }
    }

    // 404 if no title found matching the user's query
    if &template.title == &"none" {
        return (StatusCode::NOT_FOUND, "404 not found").into_response();
    }

    // render the template into HTML and return it to the user
    match template.render() {
        Ok(html) => Html(html).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "try again later").into_response(),
    }
}

// Then populate the template with all post titles
async fn index(State(state): State<Arc<Vec<BlogPost>>>) -> impl IntoResponse {
    let mut plinks: Vec<String> = Vec::new();

    for i in 0..state.len() {
        plinks.push(state[i].title.clone());
    }

    let template = IndexTemplate {
        title: "Home",
        index_links: &plinks,
    };

    match template.render() {
        Ok(html) => Html(html).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to render template. Error {}", err),
        )
            .into_response(),
    }
}
