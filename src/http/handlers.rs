use askama::Template;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, Json},
};
use serde::Deserialize;
use std::collections::HashMap;

use crate::{data::Post, db::Database, http::state::AppState};

// Template structs
#[derive(Template)]
#[template(path = "post.html")]
pub struct PostTemplate {
    pub post: Post,
    pub tags_with_count: Vec<(String, u8)>,
    pub dates_by_year: Vec<(i32, Vec<(String, u32, u8)>)>,
}

#[derive(Template)]
#[template(path = "posts_list.html")]
pub struct PostsListTemplate {
    pub posts: Vec<Post>,
    pub tags_with_count: Vec<(String, u8)>,
    pub dates_by_year: Vec<(i32, Vec<(String, u32, u8)>)>,
}

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub posts: Vec<Post>,
    pub tags_with_count: Vec<(String, u8)>,
    pub dates_by_year: Vec<(i32, Vec<(String, u32, u8)>)>,
}

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

// HTML Handlers - Mirror the API routes but return HTML

pub async fn html_get_post_by_slug(
    Path(slug): Path<String>,
    State(state): State<AppState>,
) -> Result<Html<String>, StatusCode> {
    let db = state.db.lock().expect("the database should be lockable");
    let post = db.get_by_slug(slug);
    let (tags_with_count, dates_by_year) = prepare_nav_data(&*db);
    let template = PostTemplate {
        post,
        tags_with_count,
        dates_by_year,
    };
    let html = template
        .render()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Html(html))
}

pub async fn html_get_posts_by_tag(
    Query(params): Query<TagQuery>,
    State(state): State<AppState>,
) -> Result<Html<String>, StatusCode> {
    let db = state
        .db
        .lock()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let posts = db.get_by_tag(params.tag);
    let (tags_with_count, dates_by_year) = prepare_nav_data(&*db);
    let template = PostsListTemplate {
        posts,
        tags_with_count,
        dates_by_year,
    };
    let html = template
        .render()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Html(html))
}

pub async fn html_get_posts_by_series(
    Query(params): Query<SeriesQuery>,
    State(state): State<AppState>,
) -> Result<Html<String>, StatusCode> {
    let db = state
        .db
        .lock()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let posts = db.get_by_series(params.series);
    let (tags_with_count, dates_by_year) = prepare_nav_data(&*db);
    let template = PostsListTemplate {
        posts,
        tags_with_count,
        dates_by_year,
    };
    let html = template
        .render()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Html(html))
}

pub async fn html_get_posts_by_keyword(
    Query(params): Query<KeywordQuery>,
    State(state): State<AppState>,
) -> Result<Html<String>, StatusCode> {
    let db = state
        .db
        .lock()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let posts = db.get_by_keyword(params.keyword);
    let (tags_with_count, dates_by_year) = prepare_nav_data(&*db);
    let template = PostsListTemplate {
        posts,
        tags_with_count,
        dates_by_year,
    };
    let html = template
        .render()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Html(html))
}

pub async fn html_get_posts_by_date(
    Query(params): Query<DateQuery>,
    State(state): State<AppState>,
) -> Result<Html<String>, StatusCode> {
    let db = state
        .db
        .lock()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let posts = db.get_by_year_month(params.year, params.month);
    let (tags_with_count, dates_by_year) = prepare_nav_data(&*db);
    let template = PostsListTemplate {
        posts,
        tags_with_count,
        dates_by_year,
    };
    let html = template
        .render()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Html(html))
}
#[utoipa::path(
    get,
    path = "/api/posts/latest",
    responses(
        (status = 200, description = "Latest posts", body = Vec<Post>),
        (status = 404, description = "No posts found")
    )
)]
pub async fn get_latest_posts(State(state): State<AppState>) -> Json<Vec<Post>> {
    let db = state.db.lock().unwrap();
    let posts = db.get_last_n_posts(10);
    Json(posts)
}

pub async fn html_index(State(state): State<AppState>) -> Result<Html<String>, StatusCode> {
    let db = state
        .db
        .lock()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let posts = db.get_last_n_posts(10);
    let (tags_with_count, dates_by_year) = prepare_nav_data(&*db);
    let template = IndexTemplate {
        posts,
        tags_with_count,
        dates_by_year,
    };
    let html = template
        .render()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Html(html))
}

// Helper function to prepare navigation data
fn prepare_nav_data(db: &dyn Database) -> (Vec<(String, u8)>, Vec<(i32, Vec<(String, u32, u8)>)>) {
    // Get tags sorted by count (descending)
    let mut tags_with_count = db.get_all_tags_with_count();
    tags_with_count.sort_by(|a, b| b.1.cmp(&a.1));

    // Get dates grouped by year with months
    let dates_with_count = db.get_all_dates_with_count();
    let mut dates_by_year: HashMap<i32, Vec<(String, u32, u8)>> = HashMap::new();

    for ((year, month), count) in dates_with_count {
        dates_by_year.entry(year).or_insert_with(Vec::new).push((
            month_name(month).to_string(),
            month,
            count,
        ));
    }

    // Convert to sorted vec (years descending, months ascending)
    let mut dates_by_year: Vec<(i32, Vec<(String, u32, u8)>)> = dates_by_year.into_iter().collect();
    dates_by_year.sort_by(|a, b| b.0.cmp(&a.0)); // Sort years descending

    for (_, months) in &mut dates_by_year {
        months.sort_by(|a, b| a.1.cmp(&b.1)); // Sort months by number ascending
    }

    (tags_with_count, dates_by_year)
}

fn month_name(month: u32) -> &'static str {
    match month {
        1 => "January",
        2 => "February",
        3 => "March",
        4 => "April",
        5 => "May",
        6 => "June",
        7 => "July",
        8 => "August",
        9 => "September",
        10 => "October",
        11 => "November",
        12 => "December",
        _ => "Unknown",
    }
}
