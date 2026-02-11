/// Example: Clone a GitHub repository and ingest posts
/// 
/// Usage:
/// ```
/// cargo run --example clone_repo -- https://github.com/username/blog-posts
/// ```

use personal::db::{Database, InMemDatabase};
use personal::repo_utils::clone_and_ingest_repository;

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: cargo run --example clone_repo -- <repository_url>");
        eprintln!("Example: cargo run --example clone_repo -- https://github.com/username/blog-posts");
        std::process::exit(1);
    }
    
    let repo_url = &args[1];
    
    println!("Starting repository clone and ingestion...");
    println!("Repository URL: {}", repo_url);
    println!();
    
    // Clone repository and get posts
    let posts = match clone_and_ingest_repository(repo_url).await {
        Ok(posts) => posts,
        Err(e) => {
            eprintln!("Error: Failed to clone and ingest repository: {}", e);
            std::process::exit(1);
        }
    };
    
    // Insert posts into database
    let mut db = InMemDatabase::new();
    
    for post in posts {
        println!("Inserting post: {}", post.markdown.title);
        db.insert_parsed_to_database(post)
            .expect("Failed to insert post");
    }
    
    println!();
    println!("✓ Successfully loaded {} posts into database", db.by_slug.len());
    println!("✓ Resources copied to static/misc/");
    
    // Display summary
    println!();
    println!("Post Summary:");
    for (slug, post) in &db.by_slug {
        println!("  - {} ({})", post.markdown.title, slug);
    }
}
