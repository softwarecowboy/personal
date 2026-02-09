mod database_tests {
    use std::path::Path;

    use personal::{
        db::*,
        error::ApplicationError,
        repo_utils::{Repository, get_posts_from_repository},
    };

    #[tokio::test]
    async fn test_insert_to_in_mem_db() -> Result<(), ApplicationError> {
        let mut in_mem_db = InMemDatabase::new();
        let repo: Repository = Repository::try_from(Path::new("./tests/data"))?;
        let posts = get_posts_from_repository(repo).await?;

        for post in posts.clone() {
            in_mem_db.insert_parsed_to_database(post)?
        }

        assert_eq!(
            in_mem_db.get_by_slug("example-title".to_owned()),
            posts.iter().next().unwrap().clone()
        );
        Ok(())
    }
}
