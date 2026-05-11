use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Clone, Parser, Default)]
#[command(name = "xuzzel", about = "A fuzzel-inspired X11 launcher written in Rust")]
pub struct Cli {
    #[arg(long)]
    pub dmenu: bool,

    #[arg(long)]
    pub dmenu0: bool,

    #[arg(long = "check-config")]
    pub check_config: bool,

    #[arg(long)]
    pub config: Option<PathBuf>,

    #[arg(long)]
    pub cache: Option<PathBuf>,

    #[arg(long = "show-paths")]
    pub show_paths: bool,

    #[arg(short = 'p', long)]
    pub prompt: Option<String>,

    #[arg(long = "prompt-only")]
    pub prompt_only: Option<String>,

    #[arg(long = "hide-prompt")]
    pub hide_prompt: bool,

    #[arg(long)]
    pub placeholder: Option<String>,

    #[arg(long = "message", alias = "mesg")]
    pub message: Option<String>,

    #[arg(long)]
    pub search: Option<String>,

    #[arg(long = "auto-select")]
    pub auto_select: bool,

    #[arg(long)]
    pub select: Option<String>,

    #[arg(long = "select-index")]
    pub select_index: Option<usize>,

    #[arg(short = 'l', long)]
    pub lines: Option<usize>,

    #[arg(short = 'T', long)]
    pub terminal: Option<String>,

    #[arg(long = "match-counter")]
    pub match_counter: bool,

    #[arg(long = "hide-before-typing")]
    pub hide_before_typing: bool,

    #[arg(long)]
    pub index: bool,

    #[arg(long, num_args = 0..=1, default_missing_value = "*")]
    pub password: Option<String>,

    #[arg(long = "minimal-lines")]
    pub minimal_lines: bool,

    #[arg(short = 'R', long = "no-run-if-empty")]
    pub no_run_if_empty: bool,

    #[arg(long = "only-match")]
    pub only_match: bool,

    #[arg(long = "with-nth")]
    pub with_nth: Option<String>,

    #[arg(long = "accept-nth")]
    pub accept_nth: Option<String>,

    #[arg(long = "match-nth")]
    pub match_nth: Option<String>,

    #[arg(long = "nth-delimiter", default_value = "\t")]
    pub nth_delimiter: String,
}
