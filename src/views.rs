use std::{
    collections::HashMap,
    fs,
    io::{self, BufRead, BufReader},
    path::PathBuf,
};

pub struct ViewCounterStore {
    file_path: PathBuf,
    counts: HashMap<String, u64>,
}

impl ViewCounterStore {
    pub fn load(file_path: PathBuf) -> io::Result<Self> {
        let mut counts = HashMap::new();

        if file_path.exists() {
            let file = fs::File::open(&file_path)?;
            let reader = BufReader::new(file);

            for line_result in reader.lines() {
                let line = match line_result {
                    Ok(line) => line,
                    Err(_) => continue,
                };

                let Some((slug, count_raw)) = line.split_once('\t') else {
                    continue;
                };

                let Ok(count) = count_raw.parse::<u64>() else {
                    continue;
                };

                counts.insert(slug.to_string(), count);
            }
        }

        Ok(Self { file_path, counts })
    }

    pub fn increment(&mut self, slug: &str) -> io::Result<u64> {
        let count = self.counts.entry(slug.to_string()).or_insert(0);
        *count += 1;
        let updated_count = *count;
        self.persist()?;
        Ok(updated_count)
    }

    pub fn get(&self, slug: &str) -> u64 {
        self.counts.get(slug).copied().unwrap_or(0)
    }

    fn persist(&self) -> io::Result<()> {
        if let Some(parent) = self.file_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let tmp_path = self.file_path.with_extension("tmp");
        let mut lines: Vec<(&String, &u64)> = self.counts.iter().collect();
        lines.sort_by(|a, b| a.0.cmp(b.0));

        let data = lines
            .into_iter()
            .map(|(slug, count)| format!("{slug}\t{count}"))
            .collect::<Vec<String>>()
            .join("\n");

        fs::write(&tmp_path, data)?;
        fs::rename(tmp_path, &self.file_path)?;
        Ok(())
    }
}
