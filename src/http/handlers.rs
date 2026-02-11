use askama::Template;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Html,
};
use serde::Deserialize;
use std::collections::HashMap;

use crate::{data::Post, db::Database, http::state::AppState};
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

#[derive(Deserialize)]
pub struct TagQuery {
    pub tag: String,
}

#[derive(Deserialize)]
pub struct SeriesQuery {
    pub series: String,
}

#[derive(Deserialize)]
pub struct KeywordQuery {
    pub keyword: String,
}

#[derive(Deserialize)]
pub struct DateQuery {
    pub year: i32,
    pub month: Option<u32>,
}

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

fn prepare_nav_data(
    db: &crate::db::InMemDatabase,
) -> (Vec<(String, u8)>, Vec<(i32, Vec<(String, u32, u8)>)>) {
    let mut tags_with_count = db.get_all_tags_with_count();
    tags_with_count.sort_by(|a, b| b.1.cmp(&a.1));
    let dates_with_count = db.get_all_dates_with_count();
    let mut dates_by_year: HashMap<i32, Vec<(String, u32, u8)>> = HashMap::new();

    for ((year, month), count) in dates_with_count {
        dates_by_year.entry(year).or_insert_with(Vec::new).push((
            month_name(month).to_string(),
            month,
            count,
        ));
    }

    let mut dates_by_year: Vec<(i32, Vec<(String, u32, u8)>)> = dates_by_year.into_iter().collect();
    dates_by_year.sort_by(|a, b| b.0.cmp(&a.0));

    for (_, months) in &mut dates_by_year {
        months.sort_by(|a, b| a.1.cmp(&b.1));
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
