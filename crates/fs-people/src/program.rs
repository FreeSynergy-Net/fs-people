//! The program: the words after its name, dispatched to the window, the commands or the server.
//!
//! Pattern: Facade over the crates that stay libraries (ADR 0063).

use std::process::ExitCode;

use fs_interfaces::owner::LocalOwner;
use fs_people_server::ServeError;

/// Run the program with the words after its name; `owner` is the source of the owner of the store (the person "me" of the module
/// person in the shipped program).
pub fn run(args: &[String], owner: &dyn LocalOwner) -> ExitCode {
    match args.first().map(String::as_str) {
        Some("--version" | "-V") => {
            println!("fs-people {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        Some("serve") => serve(&args[1..], owner),
        None | Some("gui") => window(args.get(1..).unwrap_or_default(), owner),
        // `fs-people --socket PATH serve`: the option may stand before the word.
        Some("--socket") if args.get(2).map(String::as_str) == Some("serve") => {
            let mut words = vec!["--socket".to_string(), args[1].clone()];
            words.extend(args[3..].iter().cloned());
            serve(&words, owner)
        }
        Some(_) if has_command(args) => exit(fs_people_cli::command_line(args)),
        Some(_) => window(args, owner),
    }
}

/// Whether the words hold a command (`list`, `show`, ...) rather than only window options.
/// `--socket PATH` belongs to both; every other option with a value is the window's.
fn has_command(words: &[String]) -> bool {
    let mut it = words.iter();
    while let Some(word) = it.next() {
        if word == "--help" || word == "-h" {
            return false;
        }
        if word.starts_with("--") {
            if word != "--local" {
                it.next();
            }
        } else {
            return true;
        }
    }
    false
}

fn exit(code: i32) -> ExitCode {
    u8::try_from(code).map_or(ExitCode::FAILURE, ExitCode::from)
}

fn serve(words: &[String], owner: &dyn LocalOwner) -> ExitCode {
    match fs_people_server::serve(words.iter().map(Into::into), owner) {
        Ok(()) => ExitCode::SUCCESS,
        Err(ServeError::Usage(m)) => {
            eprintln!("fs-people: {m}");
            ExitCode::from(2)
        }
        Err(ServeError::Failed(m)) => {
            eprintln!("fs-people: {m}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(feature = "gui")]
fn window(words: &[String], owner: &dyn LocalOwner) -> ExitCode {
    match fs_people_app::gui::run(words, owner) {
        Ok(()) => ExitCode::SUCCESS,
        Err(m) => {
            eprintln!("fs-people: {m}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(not(feature = "gui"))]
fn window(_words: &[String], _owner: &dyn LocalOwner) -> ExitCode {
    eprintln!(
        "fs-people: this build has no window (built with --no-default-features); use a command, e.g. `fs-people list`"
    );
    ExitCode::FAILURE
}
