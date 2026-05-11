use crate::model::{Entry, EntrySource};
use anyhow::{anyhow, Result};
use std::process::{Command, Stdio};

pub fn activate(entry: &Entry, terminal_command: &str) -> Result<()> {
    match entry.source {
        EntrySource::Dmenu => {
            println!("{}", entry.label);
            Ok(())
        }
        EntrySource::Desktop => launch_desktop(entry, terminal_command),
    }
}

fn launch_desktop(entry: &Entry, terminal_command: &str) -> Result<()> {
    let exec = entry.exec.as_deref().unwrap_or("");
    let argv = sanitize_exec(exec)?;
    if argv.is_empty() {
        return Err(anyhow!("desktop entry command resolved to an empty argv"));
    }

    if entry.terminal {
        let mut child = terminal_wrap_command(terminal_command, &argv)?;
        apply_child_setup(&mut child, entry).spawn()?;
    } else {
        let mut child = Command::new(&argv[0]);
        child.args(&argv[1..]);
        apply_child_setup(&mut child, entry).spawn()?;
    }

    Ok(())
}

fn apply_child_setup<'a>(command: &'a mut Command, entry: &Entry) -> &'a mut Command {
    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    if let Some(dir) = entry.working_dir.as_deref() {
        command.current_dir(dir);
    }

    command
}

fn terminal_wrap_command(terminal_command: &str, argv: &[String]) -> Result<Command> {
    let command_line = argv.join(" ");
    let terminal_argv = sanitize_exec(terminal_command)?;

    if terminal_argv.is_empty() {
        let mut child = Command::new("xterm");
        child.arg("-e").arg(command_line);
        return Ok(child);
    }

    let mut replaced = false;
    let expanded = terminal_argv
        .into_iter()
        .map(|token| {
            if token.contains("{cmd}") {
                replaced = true;
                token.replace("{cmd}", &command_line)
            } else {
                token
            }
        })
        .collect::<Vec<_>>();

    let mut child = Command::new(&expanded[0]);
    child.args(&expanded[1..]);

    if !replaced {
        child.args(argv);
    }

    Ok(child)
}

fn sanitize_exec(exec: &str) -> Result<Vec<String>> {
    let argv = shlex::split(exec).ok_or_else(|| anyhow!("failed to parse command line"))?;
    Ok(argv
        .into_iter()
        .filter_map(|token| sanitize_token(&token))
        .collect())
}

fn sanitize_token(token: &str) -> Option<String> {
    if token.starts_with('%') {
        return None;
    }

    let mut cleaned = String::new();
    let mut chars = token.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '%' {
            let _ = chars.next();
            continue;
        }
        cleaned.push(ch);
    }

    if cleaned.is_empty() {
        None
    } else {
        Some(cleaned)
    }
}

#[cfg(test)]
mod tests {
    use super::{sanitize_exec, terminal_wrap_command};

    #[test]
    fn strips_desktop_exec_placeholders() {
        let argv = sanitize_exec("env BAMF_DESKTOP_FILE_HINT=%k firefox %u --new-window").unwrap();
        assert_eq!(argv, vec!["env", "BAMF_DESKTOP_FILE_HINT=", "firefox", "--new-window"]);
    }

    #[test]
    fn preserves_quoted_arguments() {
        let argv = sanitize_exec("sh -c 'printf hello world'").unwrap();
        assert_eq!(argv, vec!["sh", "-c", "printf hello world"]);
    }

    #[test]
    fn terminal_command_replaces_placeholder() {
        let command = terminal_wrap_command(
            "foot -a {cmd} -T {cmd} {cmd}",
            &["htop".to_string()],
        )
        .unwrap();
        let debug = format!("{command:?}");
        assert!(debug.contains("foot"));
        assert!(debug.contains("htop"));
    }
}
