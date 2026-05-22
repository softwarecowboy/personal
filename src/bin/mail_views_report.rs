use chrono::Local;
use personal::{
    data::Post,
    repo_utils::{clone_and_ingest_repository, load_from_local_path},
    views::ViewCounterStore,
};
use std::{
    collections::{HashMap, HashSet},
    env,
    error::Error,
    io::{self, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    thread,
    time::Duration,
};

const DEFAULT_REMOTE_REPO: &str = "https://github.com/softwarecowboy/blog";

struct Config {
    repo_source: String,
    views_file: PathBuf,
    email_to: String,
    email_from: String,
    email_subject_prefix: String,
    sendmail_bin: String,
    interval_seconds: Option<u64>,
    include_zero_views: bool,
    max_posts: usize,
}

impl Config {
    fn from_env_and_args() -> Result<Self, Box<dyn Error>> {
        let args: Vec<String> = env::args().collect();
        let repo_source = args
            .get(1)
            .cloned()
            .or_else(|| env::var("REPORT_REPO_SOURCE").ok())
            .unwrap_or_else(|| DEFAULT_REMOTE_REPO.to_string());

        let views_file = env::var("VIEW_COUNTS_FILE")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/opt/personal/runtime/views.tsv"));

        let email_to = env::var("REPORT_EMAIL_TO")?;
        let email_from = env::var("REPORT_EMAIL_FROM")?;
        let email_subject_prefix = env::var("REPORT_EMAIL_SUBJECT_PREFIX")
            .unwrap_or_else(|_| "Personal blog views report".to_string());
        let sendmail_bin =
            env::var("REPORT_SENDMAIL_BIN").unwrap_or_else(|_| "/usr/sbin/sendmail".to_string());
        let interval_seconds = env::var("REPORT_INTERVAL_SECONDS")
            .ok()
            .map(|value| value.parse::<u64>())
            .transpose()?;
        let include_zero_views = env::var("REPORT_INCLUDE_ZERO_VIEWS")
            .ok()
            .map(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"))
            .unwrap_or(false);
        let max_posts = env::var("REPORT_MAX_POSTS")
            .ok()
            .map(|value| value.parse::<usize>())
            .transpose()?
            .unwrap_or(20);

        Ok(Self {
            repo_source,
            views_file,
            email_to,
            email_from,
            email_subject_prefix,
            sendmail_bin,
            interval_seconds,
            include_zero_views,
            max_posts,
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config = Config::from_env_and_args()?;

    if let Some(interval_seconds) = config.interval_seconds {
        loop {
            run_once(&config).await?;
            thread::sleep(Duration::from_secs(interval_seconds));
        }
    } else {
        run_once(&config).await?;
    }

    Ok(())
}

async fn run_once(config: &Config) -> Result<(), Box<dyn Error>> {
    let posts = sync_posts(&config.repo_source).await?;
    let views = ViewCounterStore::load(config.views_file.clone())?;
    let report = build_report(config, &posts, &views);
    send_email(config, &report)?;

    println!(
        "Sent view report to {} using {}",
        config.email_to, config.sendmail_bin
    );

    Ok(())
}

async fn sync_posts(repo_source: &str) -> Result<Vec<Post>, Box<dyn Error>> {
    if Path::new(repo_source).exists() {
        Ok(load_from_local_path(repo_source).await?)
    } else {
        Ok(clone_and_ingest_repository(repo_source).await?)
    }
}

fn build_report(config: &Config, posts: &[Post], views: &ViewCounterStore) -> String {
    let posts_by_slug: HashMap<&str, &Post> = posts
        .iter()
        .map(|post| (post.markdown.slug.as_str(), post))
        .collect();
    let mut rows = views.snapshot_sorted();

    if config.include_zero_views {
        let seen: HashSet<String> = rows.iter().map(|(slug, _)| slug.clone()).collect();
        for post in posts {
            if !seen.contains(&post.markdown.slug) {
                rows.push((post.markdown.slug.clone(), 0));
            }
        }
        rows.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    }

    let total_views: u64 = rows.iter().map(|(_, count)| *count).sum();
    let shown_rows = rows.into_iter().take(config.max_posts).collect::<Vec<_>>();

    let mut report = String::new();
    report.push_str("Personal Blog View Report\n");
    report.push_str("=========================\n\n");
    report.push_str(&format!(
        "Generated: {}\n",
        Local::now().format("%Y-%m-%d %H:%M:%S %Z")
    ));
    report.push_str(&format!("Repository source: {}\n", config.repo_source));
    report.push_str(&format!("Views file: {}\n", config.views_file.display()));
    report.push_str(&format!("Posts loaded: {}\n", posts.len()));
    report.push_str(&format!("Total counted views: {}\n\n", total_views));

    if shown_rows.is_empty() {
        report.push_str("No view data recorded yet.\n");
        return report;
    }

    report.push_str("Top posts\n");
    report.push_str("---------\n");

    for (index, (slug, count)) in shown_rows.iter().enumerate() {
        if let Some(post) = posts_by_slug.get(slug.as_str()) {
            report.push_str(&format!(
                "{}. {}\n   slug: {}\n   date: {}\n   views: {}\n   description: {}\n\n",
                index + 1,
                post.markdown.title,
                slug,
                post.markdown.date,
                count,
                post.markdown.description
            ));
        } else {
            report.push_str(&format!(
                "{}. {}\n   slug: {}\n   views: {}\n   note: post no longer exists in synced repository\n\n",
                index + 1,
                slug,
                slug,
                count
            ));
        }
    }

    report
}

fn send_email(config: &Config, body: &str) -> Result<(), Box<dyn Error>> {
    let subject = format!(
        "{} - {}",
        config.email_subject_prefix,
        Local::now().format("%Y-%m-%d")
    );
    let message = format!(
        "To: {}\nFrom: {}\nSubject: {}\nContent-Type: text/plain; charset=utf-8\n\n{}",
        config.email_to, config.email_from, subject, body
    );

    let mut child = Command::new(&config.sendmail_bin)
        .arg("-t")
        .arg("-i")
        .stdin(Stdio::piped())
        .spawn()?;

    let stdin = child
        .stdin
        .as_mut()
        .ok_or_else(|| io::Error::other("Failed to open sendmail stdin"))?;
    stdin.write_all(message.as_bytes())?;

    let status = child.wait()?;
    if !status.success() {
        return Err(io::Error::other(format!("sendmail exited with status {}", status)).into());
    }

    Ok(())
}
