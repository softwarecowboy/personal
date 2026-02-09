use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::Deserialize;

use crate::{data::Post, db::Database, http::state::AppState};

#[utoipa::path(
    get,
    path = "/api/posts/{slug}",
    responses(
        (status = 200, description = "Post found", body = Post),
        (status = 404, description = "Post not found")
    ),
    params(
        ("slug" = String, Path, description = "Post slug")
    )
)]
pub async fn get_post_by_slug(
    Path(slug): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<Post>, StatusCode> {
    let db = state.db.lock().expect("the database should be lockable");
    let post = db.get_by_slug(slug);
    Ok(Json(post))
}

#[derive(Deserialize)]
pub struct TagQuery {
    pub tag: String,
}

#[utoipa::path(
    get,
    path = "/api/posts/by-tag",
    responses(
        (status = 200, description = "Posts found", body = Vec<Post>),
        (status = 404, description = "No posts found")
    ),
    params(
        ("tag" = String, Query, description = "Tag to filter by")
    )
)]
pub async fn get_posts_by_tag(
    Query(params): Query<TagQuery>,
    State(state): State<AppState>,
) -> Json<Vec<Post>> {
    let db = state.db.lock().unwrap();
    let posts = db.get_by_tag(params.tag);
    Json(posts)
}

#[derive(Deserialize)]
pub struct SeriesQuery {
    pub series: String,
}

#[utoipa::path(
    get,
    path = "/api/posts/by-series",
    responses(
        (status = 200, description = "Posts found", body = Vec<Post>),
        (status = 404, description = "No posts found")
    ),
    params(
        ("series" = String, Query, description = "Series to filter by")
    )
)]
pub async fn get_posts_by_series(
    Query(params): Query<SeriesQuery>,
    State(state): State<AppState>,
) -> Json<Vec<Post>> {
    let db = state.db.lock().unwrap();
    let posts = db.get_by_series(params.series);
    Json(posts)
}

#[derive(Deserialize)]
pub struct KeywordQuery {
    pub keyword: String,
}

#[utoipa::path(
    get,
    path = "/api/posts/by-keyword",
    responses(
        (status = 200, description = "Posts found", body = Vec<Post>),
        (status = 404, description = "No posts found")
    ),
    params(
        ("keyword" = String, Query, description = "Keyword to search for")
    )
)]
pub async fn get_posts_by_keyword(
    Query(params): Query<KeywordQuery>,
    State(state): State<AppState>,
) -> Json<Vec<Post>> {
    let db = state.db.lock().unwrap();
    let posts = db.get_by_keyword(params.keyword);
    Json(posts)
}

#[derive(Deserialize)]
pub struct DateQuery {
    pub year: i32,
    pub month: Option<u32>,
}

#[utoipa::path(
    get,
    path = "/api/posts/by-date",
    responses(
        (status = 200, description = "Posts found", body = Vec<Post>),
        (status = 404, description = "No posts found")
    ),
    params(
        ("year" = i32, Query, description = "Year to filter by"),
        ("month" = Option<u32>, Query, description = "Optional month to filter by")
    )
)]
pub async fn get_posts_by_date(
    Query(params): Query<DateQuery>,
    State(state): State<AppState>,
) -> Json<Vec<Post>> {
    let db = state.db.lock().unwrap();
    let posts = db.get_by_year_month(params.year, params.month);
    Json(posts)
}
