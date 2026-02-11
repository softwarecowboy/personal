use std::fs;
use std::path::{Path, PathBuf};

use git2::Repository as GitRepository;

use crate::{
    data::{parse_to_data, Post},
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

/// Clone a GitHub repository and ingest posts into the database
///
/// # Arguments
/// * `repo_url` - The GitHub repository URL (e.g., "https://github.com/user/repo")
///
/// # Returns
/// A vector of parsed posts from the repository
///
/// # Effects
/// - Clones the repository to a temporary directory
/// - Copies all resources to static/misc/
/// - Returns parsed posts ready for database insertion
pub async fn clone_and_ingest_repository(repo_url: &str) -> Result<Vec<Post>, ApplicationError> {
    // Create a temporary directory for cloning
    let temp_dir = tempfile::tempdir()?;
    let clone_path = temp_dir.path();

    println!("Cloning repository from {} to {:?}", repo_url, clone_path);

    // Clone the repository
    GitRepository::clone(repo_url, clone_path)?;

    println!("Repository cloned successfully");

    // Create Repository structure from cloned path
    let repo = Repository::try_from(clone_path)?;

    // Copy resources to static/misc/
    let static_misc = Path::new("static/misc");
    if !static_misc.exists() {
        fs::create_dir_all(static_misc)?;
        println!("Created directory: static/misc/");
    }

    // Copy all files from resources directory
    if repo.resources.exists() {
        println!(
            "Copying resources from {:?} to {:?}",
            repo.resources, static_misc
        );
        copy_dir_all(&repo.resources, static_misc)?;
        println!("Resources copied successfully");
    } else {
        println!("Warning: No resources directory found in repository");
    }

    // Get all posts from the repository
    let posts = get_posts_from_repository(repo).await?;

    println!("Loaded {} posts from repository", posts.len());

    Ok(posts)
}

/// Recursively copy all contents from source directory to destination directory
fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if file_type.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
            println!("  Copied: {:?}", entry.file_name());
        }
    }

    Ok(())
}
