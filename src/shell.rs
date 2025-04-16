use miette::{Context, IntoDiagnostic, Result};
use reedline::{DefaultHinter, DefaultPrompt, Reedline, FileBackedHistory};
use std::cell::RefCell;
use std::collections::{BTreeSet, HashMap};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::rc::Rc;
use serde::{Serialize, Deserialize};
use serde_json;

#[derive(Debug, Serialize, Deserialize)]
struct ShellState {
    last_exit_code: i32,
    aliases: HashMap<String, String>,
}

impl Default for ShellState {
    fn default() -> Self {
        ShellState {
            last_exit_code: 0,
            aliases: HashMap::new(),
        }
    }
}

fn get_executables_in_path() -> Vec<String> {
    let mut entries = BTreeSet::new();

    if let Some(paths) = env::var_os("PATH") {
        for path in env::split_paths(&paths) {
            if let Ok(read_dir) = fs::read_dir(path) {
                for entry in read_dir.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                            entries.insert(name.to_string());
                        }
                    }
                }
            }
        }
    }

    entries.into_iter().collect()
}

pub fn run_shell(config_dir: PathBuf) -> Result<()> {
    let prompt = DefaultPrompt::default();
    let aliases_path = config_dir.join("aliases.json");
    let aliases = if aliases_path.exists() {
        let data = fs::read_to_string(&aliases_path).unwrap_or_default();
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        HashMap::new()
    };

    let shell_state = Rc::new(RefCell::new(ShellState {
        last_exit_code: 0,
        aliases,
    }));

    if !std::path::Path::exists(&config_dir) {
        let _ = std::fs::create_dir_all(config_dir.as_path());
    }

    let hinter = Box::new(DefaultHinter::default());
    let commands = get_executables_in_path()
        .into_iter()
        .chain(
            vec!["cd", "alias", "exit", "$?"]
                .into_iter()
                .map(String::from),
        )
        .collect::<Vec<_>>();
    let history_path = config_dir.join("history.rushh");
    println!("{:#?}", history_path);

    // Just for debugging :)
    // Using the old school "printing"

    // We will exit. If you are on macOS, it throws error "IO error: read-only file system" because of System Integrity Protection I guess.
    // std::process::exit(0);
    let mut line_editor = Reedline::create()
        .with_hinter(hinter)
        .with_tab_completion(commands);

    let history = Box::new(
        FileBackedHistory::with_file(1000, history_path.clone())
            .into_diagnostic()
            .wrap_err("Could not load history file")?,
    );

    line_editor = line_editor.with_history(history);

    loop {
        let sig = line_editor.read_line(&prompt).into_diagnostic()?;
        match sig {
            reedline::Signal::Success(input) => {
                if input.trim() == "exit" {
                    break;
                }

                if input.trim() == "$?" {
                    println!("{}", shell_state.borrow().last_exit_code);
                    continue;
                }

                if let Some(alias_def) = input.trim().strip_prefix("alias ") {
                    if let Some((name, value)) = alias_def.split_once('=') {
                        let name = name.trim().to_string();
                        let value = value.trim().trim_matches('"').to_string();
                        shell_state.borrow_mut().aliases.insert(name, value);

                        // Save aliases to file
                        let aliases_json = serde_json::to_string_pretty(&shell_state.borrow().aliases)
                            .into_diagnostic()?;
                        fs::write(&aliases_path, aliases_json).into_diagnostic()?;

                        continue;
                    }
                }

                let mut actual_input = input.trim().to_string();
                if let Some(alias_cmd) = shell_state
                    .borrow()
                    .aliases
                    .get(actual_input.split_whitespace().next().unwrap_or(""))
                {
                    let args = actual_input
                        .split_whitespace()
                        .skip(1)
                        .collect::<Vec<&str>>()
                        .join(" ");
                    actual_input = format!("{} {}", alias_cmd, args);
                }

                let exit_code = match run_command(&actual_input) {
                    Ok(_) => 0,
                    Err(e) => {
                        eprintln!("{:?}", e);
                        1
                    }
                };
                shell_state.borrow_mut().last_exit_code = exit_code;
            }
            reedline::Signal::CtrlC => {
                eprintln!("^C");
                shell_state.borrow_mut().last_exit_code = 96;
                continue;
            }
            reedline::Signal::CtrlD => {
                println!("Exiting Rush.");
                break;
            }
        }
    }
    Ok(())
}

fn run_command(input: &str) -> Result<()> {
    let mut parts = input.trim().split_whitespace();
    let command = parts
        .next()
        .ok_or_else(|| miette::miette!("Empty command"))?;
    let args: Vec<&str> = parts.collect();

    if command == "cd" {
        let new_dir = args
            .get(0)
            .ok_or_else(|| miette::miette!("cd: missing operand"))?;
        std::env::set_current_dir(new_dir)
            .into_diagnostic()
            .wrap_err_with(|| format!("cd: no such file or directory: {}", new_dir))?;
        return Ok(());
    }

    Command::new(command)
        .args(&args)
        .spawn()
        .into_diagnostic()
        .wrap_err_with(|| format!("Failed to execute: {}", input))?
        .wait()
        .into_diagnostic()?;

    Ok(())
}

trait RushTabCompletionExt {
    fn with_tab_completion(self, commands: Vec<String>) -> Self;
}

impl RushTabCompletionExt for Reedline {
    fn with_tab_completion(self, commands: Vec<String>) -> Self {
        use reedline::DefaultCompleter;
        self.with_completer(Box::new(DefaultCompleter::new_with_wordlen(commands, 1)))
    }
}
