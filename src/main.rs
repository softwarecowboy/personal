use std::path::Path;

use axum::{routing::get, Router};
use personal::db::{Database, InMemDatabase};
use personal::{
    http::{handlers, state::AppState},
    repo_utils::{get_posts_from_repository, Repository},
};
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
    let mut db = InMemDatabase::new();

    // todo switch to repo
    let repo_path = Path::new("tests/data");
    let repo = Repository::try_from(repo_path).expect("Failed to create repository");
    let posts = get_posts_from_repository(repo)
        .await
        .expect("Failed to load posts");

    // todo perform periodically/middleware
    for post in posts {
        db.insert_parsed_to_database(post)
            .expect("Failed to insert post");
    }

    println!("Loaded {} posts", db.by_slug.len());

    let state = AppState::new(db);

    let app = Router::new()
        .route("/", get(handlers::html_index))
        .route("/posts/{slug}", get(handlers::html_get_post_by_slug))
        .route("/posts/by-tag", get(handlers::html_get_posts_by_tag))
        .route("/posts/by-series", get(handlers::html_get_posts_by_series))
        .route(
            "/posts/by-keyword",
            get(handlers::html_get_posts_by_keyword),
        )
        .route("/posts/by-date", get(handlers::html_get_posts_by_date))
        .nest_service("/static", ServeDir::new("static"))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("Failed to bind to port");

    println!("Server running on http://127.0.0.1:3000");

    axum::serve(listener, app)
        .await
        .expect("Server failed to start");
}
