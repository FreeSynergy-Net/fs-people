//! B2 (limits): a server that never answers ends in the message `timeout` with `limit-ms`, at the
//! start and at each call, never in a window that waits for ever.
//!
//! Pattern: Integration test with a socket that is bound and never answered (own oracle: the clock)
//! and the real server with a limit no answer can meet. Tests that bind sockets run under one lock.
//! Mutant: `recv_timeout` back to `recv`, or the `tokio::time::timeout` in `ask` removed: the tests
//! are red (the first never ends within its bound, the second gets an answer).
//! Not covered: the window thread itself still waits for the answer (the workbench calls are
//! synchronous); the limit bounds that wait. See DESIGN.md, "Gaps".

use std::os::unix::net::UnixListener;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use fs_people_app::RemotePeople;

static LOCK: Mutex<()> = Mutex::new(());

#[test]
fn a_socket_that_never_answers_ends_the_start_with_timeout_in_time() {
    let _one = LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("people.sock");
    let _silent = UnixListener::bind(&path).unwrap();
    let began = Instant::now();
    let got =
        RemotePeople::connect_within(&path, Duration::from_millis(400), Duration::from_secs(10));
    let took = began.elapsed();
    let why = got.err().expect("no server, no connection");
    assert!(why.contains("timeout"), "{why}");
    assert!(why.contains("limit-ms 400"), "{why}");
    assert!(
        took >= Duration::from_millis(400) && took < Duration::from_secs(5),
        "{took:?}"
    );
}
