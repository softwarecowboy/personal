use std::{fs::read_to_string, path::PathBuf};

use chrono::NaiveDate;
use pulldown_cmark::{html, Options, Parser};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::error::ApplicationError;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, ToSchema)]
pub struct Post {
    pub data: String,
    pub markdown: Markdown,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, ToSchema)]
pub struct Markdown {
    pub title: String,
    pub slug: String,
    pub tags: Vec<String>,
    pub date: NaiveDate,
    pub series: Option<Series>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, ToSchema)]
pub struct Series {
    pub title: String,
    pub ep: u8,
}

pub async fn parse_to_data(path: &PathBuf) -> Result<Post, ApplicationError> {
    let content = read_to_string(&path).map_err(|source| ApplicationError::ReadingError {
        path: path.clone(),
        source,
    })?;

    let mut split = content.splitn(3, "---");

    let markdown_part = split
        .clone()
        .nth(1)
        .ok_or_else(|| ApplicationError::ParsingError {
            path: path.clone(),
            reason: "Missing YAML frontmatter (expected content between --- markers)".to_string(),
        })?;
    println!("{}", markdown_part);
    let markdown: Markdown = serde_yaml::from_str(markdown_part)?;
    let content = split.nth(2).ok_or_else(|| ApplicationError::ParsingError {
        path: path.clone(),
        reason: "Missing markdown content after frontmatter".to_string(),
    })?;

    let content_replaced_tags = replace_relative_paths(content).await;
    let html_content = markdown_to_html(&content_replaced_tags);

    Ok(Post {
        data: html_content,
        markdown: markdown,
    })
}

async fn replace_relative_paths(content: &str) -> String {
    use regex::Regex;
    // Matches [alt text](path) and filters for relative paths only
    let re = Regex::new(r"\[([^\]]+)\]\(([^)]+)\)").unwrap();
    re.replace_all(content, |caps: &regex::Captures| {
        let alt_text = &caps[1];
        let path = &caps[2];

        // Only replace if path is relative (doesn't start with http://, https://, or /)
        if path.starts_with("http://") || path.starts_with("https://") || path.starts_with("/") {
            format!("[{}]({})", alt_text, path)
        } else {
            format!("[{}](/static/misc/{})", alt_text, path)
        }
    })
    .to_string()
}

/// Convert markdown to HTML
fn markdown_to_html(markdown: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_TASKLISTS);

    let parser = Parser::new_ext(markdown, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}
