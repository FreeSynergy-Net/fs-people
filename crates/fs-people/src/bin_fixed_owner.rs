//! The program with a FIXED owner, for the tests that run it as a process (feature `fake-owner`, never shipped).
//!
//! Pattern: Facade — the same call as `main.rs`, with the test source of the owner (`FixedOwner`, Strategy): the owner of
//! `fs_objects::testing`, the one the tests open their own stores with.

use fs_interfaces::owner::fake::FixedOwner;

fn main() -> std::process::ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    fs_people::run(&args, &FixedOwner(fs_objects::testing::owner()))
}
