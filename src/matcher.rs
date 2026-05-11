use crate::config::{Config, MatchMode};
use crate::history::History;
use crate::model::{Entry, MatchResult, SearchData};

pub fn filter_entries(
    entries: &[Entry],
    query: &str,
    history: &History,
    config: &Config,
) -> Vec<MatchResult> {
    let query = query.trim().to_lowercase();
    let tokens: Vec<&str> = query.split_whitespace().collect();
    let mut results = Vec::new();

    for entry in entries {
        let mut combined_score = 0_i64;
        let mut combined_indices = Vec::new();
        let mut matched = true;

        for token in &tokens {
            match score_entry(&entry.search, token, config) {
                Some((score, indices)) => {
                    combined_score += score;
                    combined_indices.extend(indices);
                }
                None => {
                    matched = false;
                    break;
                }
            }
        }

        if !matched {
            continue;
        }

        if tokens.is_empty() {
            combined_score = 10;
        }

        combined_score += history.score(&entry.id) as i64 * 50;
        combined_score += prefix_bonus(&entry.search.combined, &query);
        combined_score -= entry.label.len() as i64;

        results.push(MatchResult {
            entry: entry.clone(),
            score: combined_score,
            matched_indices: combined_indices,
        });
    }

    results.sort_by(|a, b| {
        b.score
            .cmp(&a.score)
            .then_with(|| a.entry.label.to_lowercase().cmp(&b.entry.label.to_lowercase()))
    });
    results
}

fn score_entry(search: &SearchData, token: &str, config: &Config) -> Option<(i64, Vec<usize>)> {
    let mut best: Option<(i64, Vec<usize>)> = None;

    for field in &config.fields {
        let haystack = search.field(*field);
        if haystack.is_empty() {
            continue;
        }

        let candidate = match config.match_mode {
            MatchMode::Exact => exact_score(haystack, token),
            MatchMode::Fzf => subsequence_score(haystack, token),
        };

        if let Some(candidate) = candidate {
            let should_replace = best
                .as_ref()
                .map(|current| candidate.0 > current.0)
                .unwrap_or(true);
            if should_replace {
                best = Some(candidate);
            }
        }
    }

    best
}

fn prefix_bonus(haystack: &str, query: &str) -> i64 {
    if query.is_empty() {
        return 0;
    }
    if haystack.starts_with(query) {
        120
    } else if haystack.contains(query) {
        50
    } else {
        0
    }
}

fn exact_score(haystack: &str, needle: &str) -> Option<(i64, Vec<usize>)> {
    let start = haystack.find(needle)?;
    let indices = (start..start + needle.len()).collect::<Vec<_>>();
    let mut score = 150 - start as i64;
    if start == 0 {
        score += 30;
    }
    Some((score, indices))
}

fn subsequence_score(haystack: &str, needle: &str) -> Option<(i64, Vec<usize>)> {
    if needle.is_empty() {
        return Some((0, Vec::new()));
    }

    let mut score = 0_i64;
    let mut indices = Vec::new();
    let mut needle_chars = needle.chars();
    let mut current = needle_chars.next()?;
    let mut last_match = None;

    for (idx, ch) in haystack.chars().enumerate() {
        if ch != current {
            continue;
        }

        indices.push(idx);
        score += 10;

        if let Some(prev) = last_match {
            if idx == prev + 1 {
                score += 15;
            } else {
                score -= (idx - prev) as i64;
            }
        } else if idx == 0 {
            score += 20;
        }

        last_match = Some(idx);

        if let Some(next) = needle_chars.next() {
            current = next;
        } else {
            score += 100;
            return Some((score, indices));
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::filter_entries;
    use crate::config::{Config, MatchField, MatchMode};
    use crate::history::History;
    use crate::model::{Entry, EntrySource, SearchData};

    fn entry(id: &str, label: &str, search_text: &str) -> Entry {
        Entry {
            id: id.to_string(),
            index: 0,
            label: label.to_string(),
            raw_text: label.to_string(),
            icon_name: None,
            icon: None,
            exec: None,
            path: None,
            terminal: false,
            working_dir: None,
            search: SearchData {
                filename: id.to_string(),
                name: label.to_lowercase(),
                exec: search_text.to_string(),
                combined: search_text.to_string(),
                ..SearchData::default()
            },
            source: EntrySource::Dmenu,
        }
    }

    #[test]
    fn prefix_match_beats_later_match() {
        let entries = vec![
            entry("a", "Firefox", "firefox browser"),
            entry("b", "LibreWolf", "browser firefox-compatible"),
        ];
        let history = History::default();
        let config = Config::default();
        let results = filter_entries(&entries, "fir", &history, &config);

        assert_eq!(results.first().map(|item| item.entry.id.as_str()), Some("a"));
    }

    #[test]
    fn history_affects_ordering() {
        let entries = vec![entry("a", "Alpha", "alpha"), entry("b", "Alpine", "alpine")];
        let mut history = History::default();
        history.bump("b");
        history.bump("b");
        let config = Config::default();

        let results = filter_entries(&entries, "al", &history, &config);
        assert_eq!(results.first().map(|item| item.entry.id.as_str()), Some("b"));
    }

    #[test]
    fn exact_mode_uses_selected_fields() {
        let mut config = Config::default();
        config.fields = vec![MatchField::Exec];
        config.match_mode = MatchMode::Exact;

        let entries = vec![
            entry("a", "Firefox", "firefox --new-window"),
            entry("b", "Terminal", "alacritty"),
        ];
        let history = History::default();
        let results = filter_entries(&entries, "new-win", &history, &config);
        assert_eq!(results.first().map(|item| item.entry.id.as_str()), Some("a"));
    }
}
