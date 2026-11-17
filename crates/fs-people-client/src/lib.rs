//! `fs-people-client`: `People` and `RoleBook` over the socket of the people server.
//! Herkunft: fs-people@52438a65:crates/fs-people-app/src/source/remote/mod.rs
//!
//! Split by responsibility. Pattern: Proxy — `proxy.rs` is the proxy, `worker.rs` the thread that
//! owns the connection, `call.rs` the calls between them, `persons.rs`/`roles.rs` the requests,
//! `convert.rs` the conversions, `no_roles.rs` the empty role book. A client of the contract, no
//! GUI: any frontend (or the cli) can use it.
//!
//! Sorted in, not compiled (workflow geruest-heben).

#![deny(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::must_use_candidate, clippy::module_name_repetitions)]

mod call;
mod convert;
mod no_roles;
mod persons;
mod proxy;
mod roles;
mod worker;

pub use no_roles::NoRoles;
pub use proxy::RemotePeople;
