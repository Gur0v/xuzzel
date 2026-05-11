use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Default)]
pub struct History {
    counts: HashMap<String, u64>,
}

impl History {
    pub fn load(explicit_path: Option<&Path>) -> Self {
        let Some(path) = explicit_path.map(PathBuf::from).or_else(default_history_path) else {
            return Self::default();
        };

        let Ok(contents) = fs::read_to_string(path) else {
            return Self::default();
        };

        let mut counts = HashMap::new();
        for line in contents.lines() {
            let Some((id, count)) = line.split_once('|') else {
                continue;
            };
            if let Ok(count) = count.parse::<u64>() {
                counts.insert(id.to_string(), count);
            }
        }

        Self { counts }
    }

    pub fn bump(&mut self, id: &str) {
        *self.counts.entry(id.to_string()).or_insert(0) += 1;
    }

    pub fn score(&self, id: &str) -> u64 {
        self.counts.get(id).copied().unwrap_or(0)
    }

    pub fn save(&self, explicit_path: Option<&Path>) {
        let Some(path) = explicit_path.map(PathBuf::from).or_else(default_history_path) else {
            return;
        };

        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        let mut lines: Vec<_> = self.counts.iter().collect();
        lines.sort_by(|a, b| b.1.cmp(a.1).then_with(|| a.0.cmp(b.0)));

        let contents = lines
            .into_iter()
            .map(|(id, count)| format!("{id}|{count}"))
            .collect::<Vec<_>>()
            .join("\n");

        let _ = fs::write(path, contents);
    }
}

fn default_history_path() -> Option<PathBuf> {
    dirs::cache_dir().map(|base| base.join("xuzzel").join("history"))
}
