//! `fs-people`: one binary for everything (ADR 0063). Pattern: Facade over the crates that stay
//! libraries.
//!
//! ```text
//! fs-people                          the window (feature `gui`, on by default)
//! fs-people [--store F] [--size WxH] [--snapshot F.png] [--import F.vcf] [--export F.vcf]
//! fs-people list | show | add | delete | find | export | import     over the socket
//! fs-people serve                    the server (Cap'n Proto, people.capnp)
//! fs-people --version                the version, without opening anything
//! ```

mod person_owner;
mod program;

pub use person_owner::person_owner;
pub use program::run;
