mod cli;
mod config;
mod desktop;
mod history;
mod icons;
mod matcher;
mod model;
mod runner;
mod x11;

use anyhow::Result;
use clap::Parser;
use cli::Cli;
use config::Config;
use desktop::{apply_nth_transform, load_desktop_entries, load_dmenu_entries, DmenuOptions};
use history::History;
use icons::attach_icons;
use matcher::filter_entries;
use model::MatchResult;
use runner::activate;
use x11::{UiAction, X11Ui};

fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut config = Config::load(cli.config.as_deref());
    apply_cli_overrides(&mut config, &cli);

    if cli.check_config {
        return Ok(());
    }

    let mut history = History::load(cli.cache.as_deref());
    let show_paths = cli.show_paths || config.show_paths;
    let dmenu_options = DmenuOptions {
        delimiter: if cli.dmenu0 { '\0' } else { '\n' },
        show_paths,
        with_nth: cli.with_nth.clone(),
        match_nth: cli.match_nth.clone(),
        nth_delimiter: cli
            .nth_delimiter
            .chars()
            .next()
            .unwrap_or('\t'),
    };
    let mut entries = if cli.dmenu {
        if cli.prompt_only.is_some() {
            Vec::new()
        } else {
            match load_dmenu_entries(&dmenu_options) {
                Ok(items) => items,
                Err(err) => {
                    eprintln!("failed to read stdin: {err}");
                    std::process::exit(1);
                }
            }
        }
    } else {
        load_desktop_entries(show_paths)
    };

    if !cli.dmenu && config.icons_enabled {
        let icon_size = ((config.line_height.unwrap_or(22) as f32) * config.image_size_ratio)
            .round()
            .max(16.0) as u32;
        attach_icons(&mut entries, &config.icon_theme, icon_size);
    }

    if cli.dmenu && cli.no_run_if_empty && entries.is_empty() {
        return Ok(());
    }

    if entries.is_empty() && !(cli.dmenu && cli.prompt_only.is_some()) {
        eprintln!("xuzzel: no entries found");
        std::process::exit(1);
    }

    let initial_rows = if cli.dmenu && cli.prompt_only.is_some() {
        0
    } else if cli.dmenu && (cli.minimal_lines || config.minimal_lines) {
        config.lines.min(entries.len())
    } else {
        config.lines
    };
    let ui = match X11Ui::open(&config, initial_rows) {
        Ok(ui) => ui,
        Err(err) => {
            eprintln!("xuzzel: {err}");
            std::process::exit(1);
        }
    };

    let mut input = cli.search.clone().unwrap_or_default();
    let mut selected = cli.select_index.unwrap_or(0);
    let mut scroll_offset = 0_usize;

    loop {
        let matches = filter_entries(&entries, &input, &history, &config);
        let total_match_count = matches.len();
        let show_list = !(config.hide_before_typing && input.trim().is_empty());
        let page_size = config.lines.max(1);

        if total_match_count == 0 {
            selected = 0;
            scroll_offset = 0;
        } else {
            selected = selected.min(total_match_count.saturating_sub(1));
            if selected < scroll_offset {
                scroll_offset = selected;
            }
            if selected >= scroll_offset + page_size {
                scroll_offset = selected + 1 - page_size;
            }
            let max_scroll = total_match_count.saturating_sub(page_size);
            scroll_offset = scroll_offset.min(max_scroll);
        }

        let visible: Vec<_> = if cli.dmenu && cli.prompt_only.is_some() {
            Vec::new()
        } else if show_list {
            matches
                .into_iter()
                .skip(scroll_offset)
                .take(page_size)
                .collect()
        } else {
            Vec::new()
        };

        if let Some(select) = cli.select.as_deref() {
            if let Some(index) = visible
                .iter()
                .position(|item| item.entry.label.to_lowercase().contains(&select.to_lowercase()))
            {
                selected = scroll_offset + index;
            }
        }

        if config.auto_select && visible.len() == 1 && !input.trim().is_empty() {
            if activate(&visible[0].entry, &config.terminal_command).is_ok() {
                history.bump(&visible[0].entry.id);
                history.save(cli.cache.as_deref());
            }
            break;
        }

        let shown_input = if let Some(mask) = cli.password.as_deref() {
            mask.repeat(input.chars().count())
        } else {
            input.clone()
        };

        if visible.is_empty() {
            ui.draw(&config, &shown_input, total_match_count, &[], 0);
        } else {
            let local_selected = selected.saturating_sub(scroll_offset);
            ui.draw(
                &config,
                &shown_input,
                total_match_count,
                &visible,
                local_selected.min(visible.len().saturating_sub(1)),
            );
        }

        let local_selected = if visible.is_empty() {
            0
        } else {
            selected.saturating_sub(scroll_offset)
        };

        match ui.next_action(&config, local_selected, visible.len(), &mut input) {
            UiAction::Cancel => break,
            UiAction::Continue => {}
            UiAction::JumpToEdge(to_start) => {
                if total_match_count > 0 {
                    selected = if to_start {
                        0
                    } else {
                        total_match_count - 1
                    };
                }
            }
            UiAction::MoveSelection(delta) => {
                if total_match_count > 0 {
                    if delta < 0 {
                        selected = selected.saturating_sub(delta.unsigned_abs());
                    } else {
                        selected = (selected + delta as usize).min(total_match_count - 1);
                    }
                }
            }
            UiAction::Page(delta) => {
                if total_match_count > 0 {
                    if delta < 0 {
                        selected = selected.saturating_sub(page_size);
                    } else {
                        selected = (selected + page_size).min(total_match_count - 1);
                    }
                }
            }
            UiAction::SubmitSelected => {
                if let Some(item) = visible.get(local_selected) {
                    if cli.dmenu {
                        emit_dmenu_output(&cli, item);
                    } else if activate(&item.entry, &config.terminal_command).is_ok() {
                        history.bump(&item.entry.id);
                        history.save(cli.cache.as_deref());
                    }
                    break;
                } else if cli.dmenu && !cli.only_match && !input.trim().is_empty() {
                    println!("{}", input.trim());
                    break;
                }
            }
            UiAction::SubmitAt(index) => {
                if let Some(item) = visible.get(index) {
                    if cli.dmenu {
                        emit_dmenu_output(&cli, item);
                    } else if activate(&item.entry, &config.terminal_command).is_ok() {
                        history.bump(&item.entry.id);
                        history.save(cli.cache.as_deref());
                    }
                    break;
                } else if cli.dmenu && !cli.only_match && !input.trim().is_empty() {
                    println!("{}", input.trim());
                    break;
                }
            }
        }
    }

    Ok(())
}

fn apply_cli_overrides(config: &mut Config, cli: &Cli) {
    if let Some(prompt) = cli.prompt.as_deref() {
        config.prompt = prompt.to_string();
    }
    if let Some(prompt) = cli.prompt_only.as_deref() {
        config.prompt = prompt.to_string();
    }
    if cli.hide_prompt {
        config.hide_prompt = true;
    }
    if let Some(placeholder) = cli.placeholder.as_deref() {
        config.placeholder = placeholder.to_string();
    }
    if let Some(message) = cli.message.as_deref() {
        config.message = message.to_string();
    }
    if let Some(lines) = cli.lines {
        config.lines = lines;
    }
    if let Some(terminal) = cli.terminal.as_deref() {
        config.terminal_command = terminal.to_string();
    }
    if cli.auto_select {
        config.auto_select = true;
    }
    if cli.match_counter {
        config.match_counter = true;
    }
    if cli.hide_before_typing {
        config.hide_before_typing = true;
    }
    if let Some(password_character) = cli.password.as_deref() {
        config.password_character = password_character.to_string();
    }
}

fn emit_dmenu_output(cli: &Cli, item: &MatchResult) {
    if cli.index {
        println!("{}", item.entry.index);
        return;
    }

    if let Some(spec) = cli.accept_nth.as_deref() {
        let nth_delimiter = cli.nth_delimiter.chars().next().unwrap_or('\t');
        if let Some(output) = apply_nth_transform(&item.entry.raw_text, Some(spec), nth_delimiter) {
            println!("{output}");
            return;
        }
    }

    println!("{}", item.entry.raw_text);
}
