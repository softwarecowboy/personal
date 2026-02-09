use std::path::{Path, PathBuf};

use crate::{
    data::{Post, parse_to_data},
    error::ApplicationError,
};

pub struct Repository {
    pub posts: PathBuf,
    pub resources: PathBuf,
}

impl TryFrom<&Path> for Repository {
    type Error = std::io::Error;

    fn try_from(value: &Path) -> Result<Self, Self::Error> {
        if value.is_dir() {
            let posts = value.join("posts");
            let resources = value.join("resources");
            return Ok(Repository { posts, resources });
        } else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "given path isn't a dir",
            ));
        }
    }
}

pub async fn get_posts_from_repository(repo: Repository) -> Result<Vec<Post>, ApplicationError> {
    let dir_entry = std::fs::read_dir(repo.posts)?;
    let mut result: Vec<Post> = Vec::new();
    for entry in dir_entry {
        let entry = entry.expect("should be file");
        let path = entry.path();

        let post = parse_to_data(&path).await?;
        result.push(post);
    }
    Ok(result)
}
