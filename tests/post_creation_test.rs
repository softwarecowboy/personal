#[allow(unused)]
#[cfg(test)]
mod tests {
    use std::fs;
    use std::fs::read_dir;
    use std::path::Path;

    use personal::data::*;
    use personal::repo_utils;
    use personal::repo_utils::Repository;

    #[tokio::test]
    async fn test_reading_metadata() {
        let repo_dir = Path::new("./tests/data");
        let repository: Repository =
            Repository::try_from(repo_dir).expect("Failed to create repository");
        let dir_entry = read_dir(repository.posts).expect("should be readable posts dir");
        for entry in dir_entry {
            let entry = entry.expect("should be file");
            let path = entry.path();

            let post = parse_to_data(&path).await.expect("should be valid post");
            dbg!(&post);
            assert_eq!(post.markdown.slug, "example-title".to_owned());
        }
    }
}
