use std::{collections::HashMap, fs::read_to_string, path::PathBuf};

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
    pub description: String,
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
    if markdown.description.chars().count() > 200 {
        return Err(ApplicationError::ParsingError {
            path: path.clone(),
            reason: "Frontmatter `description` must be at most 200 characters".to_string(),
        });
    }
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
    let re = Regex::new(r"\[([^\]]+)\]\(([^)]+)\)").unwrap();
    re.replace_all(content, |caps: &regex::Captures| {
        let alt_text = &caps[1];
        let path = &caps[2];
        if path.starts_with("http://") || path.starts_with("https://") || path.starts_with("/") {
            format!("[{}]({})", alt_text, path)
        } else {
            format!("[{}](/static/misc/{})", alt_text, path)
        }
    })
    .to_string()
}
fn markdown_to_html(markdown: &str) -> String {
    // Smart spacing:
    // - Single line break = continues same paragraph
    // - One blank line = new paragraph
    // - Two+ blank lines = new paragraph with extra <br> for spacing
    let lines: Vec<&str> = markdown.lines().collect();
    let mut processed = String::new();
    let mut blank_count = 0;

    for line in lines.iter() {
        if line.trim().is_empty() {
            blank_count += 1;
        } else {
            // Add extra breaks for multiple blank lines (2+ blanks = paragraph + <br>)
            if blank_count > 1 {
                processed.push_str("\n\n<br>\n\n");
            } else if blank_count == 1 {
                processed.push_str("\n\n");
            }
            processed.push_str(line);
            processed.push('\n');
            blank_count = 0;
        }
    }

    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_SMART_PUNCTUATION);
    options.insert(Options::ENABLE_HEADING_ATTRIBUTES);

    let parser = Parser::new_ext(&processed, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    add_heading_anchors(&html_output)
}

fn add_heading_anchors(html: &str) -> String {
    use regex::Regex;

    let id_re = Regex::new(r#"id\s*=\s*\"([^\"]+)\""#).unwrap();
    let tag_re = Regex::new(r"<[^>]+>").unwrap();
    let mut used: HashMap<String, usize> = HashMap::new();

    let mut output = String::with_capacity(html.len() + 256);
    let mut index = 0;
    let bytes = html.as_bytes();

    while index < html.len() {
        let remaining = &html[index..];
        let Some(rel_start) = remaining.find("<h") else {
            output.push_str(remaining);
            break;
        };

        let start = index + rel_start;
        output.push_str(&html[index..start]);

        if start + 2 >= html.len() {
            output.push_str(&html[start..]);
            break;
        }

        let level_char = bytes[start + 2] as char;
        if !('1'..='6').contains(&level_char) {
            output.push_str("<h");
            index = start + 2;
            continue;
        }

        let Some(open_end_rel) = html[start..].find('>') else {
            output.push_str(&html[start..]);
            break;
        };
        let open_end = start + open_end_rel;
        let attrs = &html[start + 3..open_end];

        let closing = format!("</h{}>", level_char);
        let after_open = open_end + 1;
        let Some(close_rel) = html[after_open..].find(&closing) else {
            output.push_str(&html[start..open_end + 1]);
            index = open_end + 1;
            continue;
        };

        let close_start = after_open + close_rel;
        let inner = &html[after_open..close_start];
        let close_end = close_start + closing.len();

        let mut base_id = id_re
            .captures(attrs)
            .and_then(|capture| capture.get(1))
            .map(|m| m.as_str().to_string());

        if base_id.is_none() {
            let text = tag_re.replace_all(inner, "");
            let slug = slugify(&text);
            if slug.is_empty() {
                output.push_str(&html[start..close_end]);
                index = close_end;
                continue;
            }
            base_id = Some(slug);
        }

        let base_id = base_id.unwrap();
        let mut unique_id = base_id.clone();
        let mut suffix = 1;
        while used.contains_key(&unique_id) {
            suffix += 1;
            unique_id = format!("{}-{}", base_id, suffix);
        }
        used.insert(unique_id.clone(), 1);

        let new_attrs = if id_re.is_match(attrs) {
            id_re
                .replace(attrs, format!("id=\"{}\"", unique_id))
                .to_string()
        } else if attrs.trim().is_empty() {
            format!(" id=\"{}\"", unique_id)
        } else {
            format!("{} id=\"{}\"", attrs, unique_id)
        };

        let hash_link = format!(
            "<a class=\"heading-hash\" href=\"#{}\" aria-label=\"Link to section\">#</a>",
            unique_id
        );

        output.push_str(&format!(
            "<h{level}{attrs}>{inner}{hash_link}</h{level}>",
            level = level_char,
            attrs = new_attrs,
            inner = inner,
            hash_link = hash_link
        ));

        index = close_end;
    }

    output
}

fn slugify(value: &str) -> String {
    let mut output = String::new();
    let mut last_dash = false;

    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            output.push(ch.to_ascii_lowercase());
            last_dash = false;
        } else if ch.is_whitespace() || ch == '-' {
            if !output.is_empty() && !last_dash {
                output.push('-');
                last_dash = true;
            }
        }
    }

    while output.ends_with('-') {
        output.pop();
    }

    output
}
