use crate::model::{Entry, EntrySource, SearchData};
use std::collections::HashSet;
use std::env;
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use which::which;

#[derive(Clone, Debug)]
pub struct DmenuOptions {
    pub delimiter: char,
    pub show_paths: bool,
    pub with_nth: Option<String>,
    pub match_nth: Option<String>,
    pub nth_delimiter: char,
}

pub fn load_desktop_entries(show_paths: bool) -> Vec<Entry> {
    let mut entries = Vec::new();
    let mut seen = HashSet::new();

    for dir in application_dirs() {
        collect_from_dir(&dir, &mut entries, &mut seen, show_paths);
    }

    entries.sort_by(|a, b| a.label.to_lowercase().cmp(&b.label.to_lowercase()));
    entries
}

pub fn load_dmenu_entries(options: &DmenuOptions) -> io::Result<Vec<Entry>> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let chunks: Box<dyn Iterator<Item = &str>> = if options.delimiter == '\0' {
        Box::new(input.split('\0'))
    } else {
        Box::new(input.lines())
    };

    Ok(chunks
        .enumerate()
        .filter_map(|(idx, raw_line)| {
            let raw_text = raw_line.trim_end_matches('\n').trim_end_matches('\r').to_string();
            if raw_text.is_empty() {
                return None;
            }

            let display_text =
                apply_nth_transform(&raw_text, options.with_nth.as_deref(), options.nth_delimiter)
                    .unwrap_or_else(|| raw_text.clone());
            let match_text =
                apply_nth_transform(&raw_text, options.match_nth.as_deref(), options.nth_delimiter)
                    .unwrap_or_else(|| display_text.clone());
            let combined = if options.show_paths {
                format!("{} stdin", match_text.to_lowercase())
            } else {
                match_text.to_lowercase()
            };

            Some(Entry {
                id: format!("stdin:{idx}"),
                index: idx,
                label: display_text,
                raw_text,
                icon_name: None,
                icon: None,
                exec: None,
                path: None,
                terminal: false,
                working_dir: None,
                search: SearchData {
                    name: match_text.to_lowercase(),
                    combined,
                    ..SearchData::default()
                },
                source: EntrySource::Dmenu,
            })
        })
        .collect())
}

pub fn apply_nth_transform(input: &str, spec: Option<&str>, delimiter: char) -> Option<String> {
    let spec = spec?.trim();
    if spec.is_empty() || spec == "0" {
        return None;
    }

    let columns: Vec<&str> = input.split(delimiter).collect();

    if let Ok(index) = spec.parse::<usize>() {
        if index == 0 {
            return None;
        }
        return columns.get(index - 1).map(|value| value.to_string());
    }

    Some(expand_column_format(spec, &columns))
}

fn expand_column_format(spec: &str, columns: &[&str]) -> String {
    let mut output = String::new();
    let mut chars = spec.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch != '{' {
            output.push(ch);
            continue;
        }

        let mut token = String::new();
        while let Some(next) = chars.next() {
            if next == '}' {
                break;
            }
            token.push(next);
        }

        if let Some(range_pos) = token.find("..") {
            let start = token[..range_pos].trim().parse::<usize>().ok().unwrap_or(1);
            let end_raw = token[range_pos + 2..].trim();
            let end = if end_raw.is_empty() {
                columns.len()
            } else {
                end_raw.parse::<usize>().ok().unwrap_or(columns.len())
            };

            let joined = columns
                .iter()
                .enumerate()
                .filter_map(|(idx, value)| {
                    let one_based = idx + 1;
                    (one_based >= start && one_based <= end).then_some(*value)
                })
                .collect::<Vec<_>>()
                .join(" ");
            output.push_str(&joined);
        } else if let Ok(index) = token.trim().parse::<usize>() {
            if let Some(value) = columns.get(index.saturating_sub(1)) {
                output.push_str(value);
            }
        } else {
            output.push('{');
            output.push_str(&token);
            output.push('}');
        }
    }

    output
}

fn collect_from_dir(
    dir: &Path,
    entries: &mut Vec<Entry>,
    seen: &mut HashSet<String>,
    show_paths: bool,
) {
    let Ok(read_dir) = fs::read_dir(dir) else {
        return;
    };

    for item in read_dir.flatten() {
        let path = item.path();
        if path.is_dir() {
            collect_from_dir(&path, entries, seen, show_paths);
            continue;
        }

        if path.extension().and_then(|ext| ext.to_str()) != Some("desktop") {
            continue;
        }

        let Some(entry) = parse_desktop_file(&path, show_paths) else {
            continue;
        };

        if seen.insert(entry.id.clone()) {
            entries.push(entry);
        }
    }
}

fn parse_desktop_file(path: &Path, show_paths: bool) -> Option<Entry> {
    let contents = fs::read_to_string(path).ok()?;
    let mut in_desktop_entry = false;
    let mut name = None;
    let mut generic_name = None;
    let mut comment = None;
    let mut keywords = Vec::new();
    let mut categories = Vec::new();
    let mut icon_name = None;
    let mut exec = None;
    let mut hidden = false;
    let mut no_display = false;
    let mut terminal = false;
    let mut try_exec = None;
    let mut working_dir = None;

    for raw_line in contents.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if line.starts_with('[') && line.ends_with(']') {
            in_desktop_entry = line == "[Desktop Entry]";
            continue;
        }

        if !in_desktop_entry {
            continue;
        }

        let Some((key, value)) = line.split_once('=') else {
            continue;
        };

        match key.trim() {
            "Name" if name.is_none() => name = Some(value.trim().to_string()),
            "GenericName" if generic_name.is_none() => generic_name = Some(value.trim().to_string()),
            "Comment" if comment.is_none() => comment = Some(value.trim().to_string()),
            "Icon" if icon_name.is_none() => icon_name = Some(value.trim().to_string()),
            "Exec" if exec.is_none() => exec = Some(value.trim().to_string()),
            "TryExec" if try_exec.is_none() => try_exec = Some(value.trim().to_string()),
            "Path" if working_dir.is_none() => working_dir = Some(value.trim().to_string()),
            "NoDisplay" => no_display = matches!(value.trim(), "true" | "1"),
            "Hidden" => hidden = matches!(value.trim(), "true" | "1"),
            "Terminal" => terminal = matches!(value.trim(), "true" | "1"),
            "Keywords" if keywords.is_empty() => {
                keywords = parse_list(value);
            }
            "Categories" if categories.is_empty() => {
                categories = parse_list(value);
            }
            _ => {}
        }
    }

    if hidden || no_display {
        return None;
    }

    let label = name?;
    let exec = exec?;
    if let Some(binary) = try_exec.as_deref() {
        if !binary_in_path(binary) {
            return None;
        }
    }

    let id = desktop_id(path);
    let path_text = path.to_string_lossy().to_string();
    let search = build_search_data(
        &id,
        &label,
        generic_name.as_deref(),
        &exec,
        &keywords,
        &categories,
        comment.as_deref(),
        show_paths.then_some(path_text.as_str()),
    );

    Some(Entry {
        id,
        index: 0,
        label,
        raw_text: String::new(),
        icon_name,
        icon: None,
        exec: Some(exec),
        path: Some(path_text),
        terminal,
        working_dir,
        search,
        source: EntrySource::Desktop,
    })
}

fn parse_list(value: &str) -> Vec<String> {
    value
        .split(';')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn desktop_id(path: &Path) -> String {
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .map(|stem| stem.to_string())
        .unwrap_or_else(|| path.to_string_lossy().to_string())
}

fn application_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    if let Ok(home) = env::var("HOME") {
        dirs.push(Path::new(&home).join(".local/share/applications"));
    }

    if let Ok(data_home) = env::var("XDG_DATA_HOME") {
        dirs.push(Path::new(&data_home).join("applications"));
    }

    if let Ok(data_dirs) = env::var("XDG_DATA_DIRS") {
        for dir in data_dirs.split(':').filter(|part| !part.is_empty()) {
            dirs.push(Path::new(dir).join("applications"));
        }
    } else {
        dirs.push(PathBuf::from("/usr/local/share/applications"));
        dirs.push(PathBuf::from("/usr/share/applications"));
    }

    dirs
}

fn build_search_data(
    filename: &str,
    label: &str,
    generic_name: Option<&str>,
    exec: &str,
    keywords: &[String],
    categories: &[String],
    comment: Option<&str>,
    path: Option<&str>,
) -> SearchData {
    let filename = filename.to_lowercase();
    let name = label.to_lowercase();
    let generic = generic_name.unwrap_or("").to_lowercase();
    let exec = exec.to_lowercase();
    let keywords = keywords
        .iter()
        .map(|item| item.to_lowercase())
        .collect::<Vec<_>>()
        .join(" ");
    let categories = categories
        .iter()
        .map(|item| item.to_lowercase())
        .collect::<Vec<_>>()
        .join(" ");
    let comment = comment.unwrap_or("").to_lowercase();

    let mut parts = vec![name.clone()];
    if !filename.is_empty() {
        parts.push(filename.clone());
    }
    if !generic.is_empty() {
        parts.push(generic.clone());
    }
    if !exec.is_empty() {
        parts.push(exec.clone());
    }
    if !keywords.is_empty() {
        parts.push(keywords.clone());
    }
    if !categories.is_empty() {
        parts.push(categories.clone());
    }
    if !comment.is_empty() {
        parts.push(comment.clone());
    }
    if let Some(path) = path {
        parts.push(path.to_lowercase());
    }

    SearchData {
        filename,
        name,
        generic,
        exec,
        keywords,
        categories,
        comment,
        combined: parts.join(" "),
    }
}

fn binary_in_path(program: &str) -> bool {
    if program.is_empty() {
        return false;
    }

    if program.contains('/') {
        return Path::new(program).exists();
    }

    which(program).is_ok()
}

#[cfg(test)]
mod tests {
    use super::{apply_nth_transform, binary_in_path, build_search_data};

    #[test]
    fn search_text_includes_metadata() {
        let keywords = vec!["browser".to_string(), "web".to_string()];
        let categories = vec!["Network".to_string()];
        let text = build_search_data(
            "firefox",
            "Firefox",
            Some("Web Browser"),
            "firefox %u",
            &keywords,
            &categories,
            Some("Browse the web"),
            Some("/usr/share/applications/firefox.desktop"),
        );

        assert!(text.combined.contains("firefox"));
        assert!(text.generic.contains("web browser"));
        assert!(text.comment.contains("browse the web"));
        assert!(text.keywords.contains("browser web"));
        assert!(text.categories.contains("network"));
        assert!(text.combined.contains("firefox.desktop"));
    }

    #[test]
    fn path_lookup_handles_missing_binary() {
        assert!(!binary_in_path("definitely-not-a-real-binary-for-xuzzel"));
    }

    #[test]
    fn nth_transform_supports_single_column_and_ranges() {
        assert_eq!(
            apply_nth_transform("1\tTwo\tThree", Some("2"), '\t').as_deref(),
            Some("Two")
        );
        assert_eq!(
            apply_nth_transform("1\tTwo\tThree", Some("{2} / {1..2}"), '\t').as_deref(),
            Some("Two / 1 Two")
        );
    }
}
