//! The program `fs-people`: all of it is in the library (`fs_people::run`).
//!
//! Pattern: Facade — one call, with the source of the local owner (`LocalOwner`, Strategy): the person "me" of the module
//! `person`, asked from its service. Without that person a store is not opened and the reason is said on stderr; no owner is
//! made up.

fn main() -> std::process::ExitCode {
    let running = std::env::current_exe().unwrap_or_default();
    let owner = fs_people::person_owner(&running);
    let args: Vec<String> = std::env::args().skip(1).collect();
    fs_people::run(&args, &*owner)
}
