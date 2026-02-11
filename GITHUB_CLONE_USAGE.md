# GitHub Repository Cloning Module

## Overview

This module enables cloning a GitHub repository containing blog posts and resources, automatically ingesting the posts into the in-memory database and copying resources to the static directory.

## Expected Repository Structure

Your GitHub repository should have the following structure:

```
repository/
├── posts/
│   ├── 00_post.md
│   ├── 01_another_post.md
│   └── ...
└── resources/
    ├── img1.png
    ├── diagram.svg
    └── ...
```

## Usage

### Option 1: Use in main application

Update your `main.rs` to use the `clone_and_ingest_repository` function:

```rust
use personal::db::{Database, InMemDatabase};
use personal::repo_utils::clone_and_ingest_repository;

#[tokio::main]
async fn main() {
    let mut db = InMemDatabase::new();
    
    // Replace with your repository URL
    let repo_url = "https://github.com/username/blog-posts";
    
    let posts = clone_and_ingest_repository(repo_url)
        .await
        .expect("Failed to clone and load posts from repository");
    
    for post in posts {
        db.insert_parsed_to_database(post)
            .expect("Failed to insert post");
    }
    
    println!("Loaded {} posts", db.by_slug.len());
}
```

### Option 2: Use the example CLI

Run the provided example with your repository URL:

```bash
cargo run --example clone_repo -- https://github.com/username/blog-posts
```

### Option 3: Use with environment variable

Create a `.env` file or set environment variable:

```bash
export BLOG_REPO_URL="https://github.com/username/blog-posts"
cargo run
```

Then in your code:

```rust
let repo_url = std::env::var("BLOG_REPO_URL")
    .expect("BLOG_REPO_URL environment variable not set");
let posts = clone_and_ingest_repository(&repo_url).await?;
```

## What it does

1. **Clones the repository** to a temporary directory
2. **Parses posts** from the `posts/` directory using existing `parse_to_data` function
3. **Copies resources** from `resources/` to `static/misc/`
4. **Returns parsed posts** ready for database insertion

## Post Format

Posts should follow the existing format with YAML frontmatter:

```markdown
---
title: "My Blog Post"
slug: "my-blog-post"
tags: ["rust", "web"]
date: 2024-01-15
series:
  title: "Learning Rust"
  ep: 1
---

# Post content here

Images and resources can be referenced using relative paths:
![Example image](image.png)

These will automatically be rewritten to point to `/static/misc/image.png`
```

## Error Handling

The function returns `Result<Vec<Post>, ApplicationError>` and can fail with:
- `GitError`: Failed to clone repository
- `IoError`: Failed to read/write files
- `ParsingError`: Failed to parse post frontmatter
- `YamlError`: Invalid YAML in frontmatter

## Dependencies

The following dependencies are used:
- `git2`: For cloning repositories
- `tempfile`: For temporary directory management (if not already present)

Both are automatically added to your `Cargo.toml`.
