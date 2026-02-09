use std::path::Path;

use axum::{Router, routing::get};
use personal::{
    db::{Database, InMemDatabase},
    http::{handlers, state::AppState},
    repo_utils::{Repository, get_posts_from_repository},
};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi as SwaggerUiRouter;

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::get_post_by_slug,
        handlers::get_posts_by_tag,
        handlers::get_posts_by_series,
        handlers::get_posts_by_keyword,
        handlers::get_posts_by_date,
    ),
    components(schemas(personal::data::Post, personal::data::Markdown, personal::data::Series)),
    info(
        title = "Personal Blog API",
        version = "0.1.0",
        description = "API for accessing blog posts"
    )
)]
struct ApiDoc;

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
        .route("/api/posts/{slug}", get(handlers::get_post_by_slug))
        .route("/api/posts/by-tag", get(handlers::get_posts_by_tag))
        .route("/api/posts/by-series", get(handlers::get_posts_by_series))
        .route("/api/posts/by-keyword", get(handlers::get_posts_by_keyword))
        .route("/api/posts/by-date", get(handlers::get_posts_by_date))
        .merge(SwaggerUiRouter::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("Failed to bind to port");

    println!("Server running on http://127.0.0.1:3000");
    println!("Swagger UI available at http://127.0.0.1:3000/swagger-ui");

    axum::serve(listener, app)
        .await
        .expect("Server failed to start");
}
