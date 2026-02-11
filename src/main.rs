use axum::{routing::get, Router};
use chrono::{Duration as ChronoDuration, Local, TimeZone};
use personal::db::{Database, InMemDatabase};
use personal::{
    error::ApplicationError,
    http::{handlers, state::AppState},
    repo_utils::clone_and_ingest_repository,
};
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
    let repo_url = "https://github.com/softwarecowboy/blog";
    let db = build_database(repo_url)
        .await
        .expect("Failed to load posts");
    println!("Loaded {} posts", db.by_slug.len());

    let state = AppState::new(db);
    let repo_url = repo_url.to_string();
    let db_handle = state.db.clone();
    tokio::spawn(async move {
        loop {
            let sleep_for = duration_until_next_midnight();
            tokio::time::sleep(sleep_for).await;
            match build_database(&repo_url).await {
                Ok(new_db) => {
                    let mut guard = db_handle.lock().expect("the database should be lockable");
                    *guard = new_db;
                }
                Err(err) => {
                    eprintln!("Failed to reload posts: {err}");
                }
            }
        }
    });

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

async fn build_database(repo_url: &str) -> Result<InMemDatabase, ApplicationError> {
    let posts = clone_and_ingest_repository(repo_url).await?;
    let mut db = InMemDatabase::new();
    for post in posts {
        db.insert_parsed_to_database(post)?;
    }
    Ok(db)
}

fn duration_until_next_midnight() -> std::time::Duration {
    let now = Local::now();
    let tomorrow = now.date_naive() + ChronoDuration::days(1);
    let next_midnight = Local
        .from_local_datetime(&tomorrow.and_hms_opt(0, 0, 0).unwrap())
        .unwrap();
    let duration = next_midnight - now;
    duration
        .to_std()
        .unwrap_or_else(|_| std::time::Duration::from_secs(0))
}
