use std::collections::HashMap;

use chrono::Datelike;

use crate::data::Post;
use crate::error::ApplicationError;

pub trait Database {
    fn get_by_slug(&self, slug: String) -> Post;
    fn get_by_tag(&self, tag: String) -> Vec<Post>;
    fn get_by_series(&self, series: String) -> Vec<Post>;
    fn get_by_keyword(&self, keyword: String) -> Vec<Post>;
    fn get_by_year_month(&self, year: i32, month: Option<u32>) -> Vec<Post>;
    fn get_last_n_posts(&self, n: usize) -> Vec<Post>;
    fn get_all_tags_with_count(&self) -> Vec<(String, u8)>;
    fn get_all_dates_with_count(&self) -> Vec<((i32, u32), u8)>;

    fn insert_parsed_to_database(&mut self, post: Post) -> Result<(), ApplicationError>;
}

pub struct InMemDatabase {
    pub by_slug: HashMap<String, Post>,
    pub by_tag: HashMap<String, String>,
    pub by_series: HashMap<String, String>,
    pub by_keyword: HashMap<String, String>,
    pub by_date: HashMap<(i32, u32), String>,
}

impl InMemDatabase {
    pub fn new() -> InMemDatabase {
        InMemDatabase {
            by_slug: HashMap::new(),
            by_tag: HashMap::new(),
            by_keyword: HashMap::new(),
            by_date: HashMap::new(),
            by_series: HashMap::new(),
        }
    }
}

impl Database for InMemDatabase {
    fn insert_parsed_to_database(&mut self, post: Post) -> Result<(), ApplicationError> {
        let slug = post.markdown.slug.clone();
        let date: (i32, u32) = (post.markdown.date.year(), post.markdown.date.month());

        // Insert by slug
        self.by_slug.entry(slug.clone()).or_insert(post.clone());

        // Insert by date
        self.by_date.entry(date).or_insert(slug.clone());

        // Insert by tags
        for tag in &post.markdown.tags {
            self.by_tag
                .entry(tag.clone())
                .or_insert_with(String::new)
                .push_str(&format!("{},", slug));
        }

        // Insert by series
        if let Some(series) = &post.markdown.series {
            self.by_series
                .entry(series.title.clone())
                .or_insert_with(String::new)
                .push_str(&format!("{},", slug));
        }

        Ok(())
    }

    fn get_by_slug(&self, slug: String) -> Post {
        self.by_slug.get(&slug).cloned().expect("Post not found")
    }

    fn get_by_tag(&self, tag: String) -> Vec<Post> {
        self.by_tag
            .get(&tag)
            .map(|slugs| {
                slugs
                    .split(',')
                    .filter(|s| !s.is_empty())
                    .filter_map(|slug| self.by_slug.get(slug).cloned())
                    .collect()
            })
            .unwrap_or_default()
    }

    fn get_by_series(&self, series: String) -> Vec<Post> {
        self.by_series
            .get(&series)
            .map(|slugs| {
                slugs
                    .split(',')
                    .filter(|s| !s.is_empty())
                    .filter_map(|slug| self.by_slug.get(slug).cloned())
                    .collect()
            })
            .unwrap_or_default()
    }

    fn get_by_keyword(&self, _keyword: String) -> Vec<Post> {
        Vec::new()
    }

    fn get_by_year_month(&self, year: i32, month: Option<u32>) -> Vec<Post> {
        self.by_date
            .iter()
            .filter(|((y, m), _)| *y == year && month.map_or(true, |month| *m == month))
            .filter_map(|(_, slug)| self.by_slug.get(slug).cloned())
            .collect()
    }

    fn get_last_n_posts(&self, n: usize) -> Vec<Post> {
        let mut posts: Vec<Post> = self.by_slug.values().cloned().collect();
        posts.sort_by(|a, b| b.markdown.date.cmp(&a.markdown.date));
        posts.into_iter().take(n).collect()
    }

    fn get_all_tags_with_count(&self) -> Vec<(String, u8)> {
        self.by_tag
            .iter()
            .map(|(tag, slugs)| {
                let count = slugs.split(',').filter(|s| !s.is_empty()).count() as u8;
                (tag.clone(), count)
            })
            .collect()
    }

    fn get_all_dates_with_count(&self) -> Vec<((i32, u32), u8)> {
        let mut date_counts: HashMap<(i32, u32), u8> = HashMap::new();

        for post in self.by_slug.values() {
            let date = (post.markdown.date.year(), post.markdown.date.month());
            *date_counts.entry(date).or_insert(0) += 1;
        }

        date_counts.into_iter().collect()
    }
}
