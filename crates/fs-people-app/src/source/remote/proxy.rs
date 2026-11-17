//! `RemotePeople`: `People` over the Cap'n Proto socket of the people server (ADR 0063).
//!
//! Pattern: Proxy — the same contract as the in-process core, answered by the server. A Cap'n Proto
//! client lives on one thread, so a worker thread owns the connection and the proxy hands it calls
//! through a channel; the window thread waits for the answer like for the in-process core.
//!
//! Provisional: only what the window needs (read, list, search, create, update, delete, and of the
//! roles: read one, read a person's, give and end). The rest of `RoleBook` answers "not possible".

use std::path::Path;
use std::sync::{Arc, PoisonError};
use std::thread;
use std::time::Duration;

use async_trait::async_trait;
use fs_interfaces::Capabilities;
use fs_objects::{ObjectId, StoredObject};
use fs_people_core::{
    Card, Error, ListFilter, People, Result, Revision, RoleAssignment, RoleBook, RoleDefinition,
};
use tokio::sync::{mpsc, oneshot};

use super::call::{Call, Reply, Revisions};
use super::worker::worker;

/// The people server behind a socket, as a provider of [`People`].
pub struct RemotePeople {
    calls: mpsc::UnboundedSender<Call>,
    revisions: Revisions,
    call_limit: Duration,
}

/// How long the start and one call may take (the same 10 s as `RestartTimes` of `fs-rpc`). A server
/// that never answers ends in the message `timeout`, never in a window that waits for ever.
pub const START_LIMIT: Duration = Duration::from_secs(10);
/// See [`START_LIMIT`].
pub const CALL_LIMIT: Duration = Duration::from_secs(10);

/// The problem of a limit that ran out: words with the code `timeout` and `limit-ms`.
fn timed_out(limit: Duration) -> Error {
    Error::Stored(format!(
        "timeout: the people server did not answer (limit-ms {})",
        limit.as_millis()
    ))
}

impl RemotePeople {
    /// Connect to the server on `socket`.
    ///
    /// # Errors
    /// The text for the person when nothing answers on the socket.
    pub fn connect(socket: &Path) -> std::result::Result<Self, String> {
        Self::connect_within(socket, START_LIMIT, CALL_LIMIT)
    }

    /// Like [`connect`](Self::connect) with the limits of the start and of each call given.
    ///
    /// # Errors
    /// The text for the person when nothing answers on the socket within `start`; it says `timeout`
    /// and `limit-ms`.
    pub fn connect_within(
        socket: &Path,
        start: Duration,
        call: Duration,
    ) -> std::result::Result<Self, String> {
        let (calls, inbox) = mpsc::unbounded_channel();
        let (ready, started) = std::sync::mpsc::channel();
        let shown = socket_name(socket);
        let socket = socket.to_path_buf();
        let revisions = Revisions::default();
        let shared = Arc::clone(&revisions);
        thread::Builder::new()
            .name("people-rpc".into())
            .spawn(move || worker(&socket, inbox, &ready, &shared))
            .map_err(|e| e.to_string())?;
        match started.recv_timeout(start) {
            Ok(done) => done?,
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                return Err(format!(
                    "timeout: no answer from {} (limit-ms {})",
                    shown,
                    start.as_millis()
                ));
            }
            Err(e) => return Err(e.to_string()),
        }
        Ok(Self {
            calls,
            revisions,
            call_limit: call,
        })
    }

    async fn ask<T>(&self, make: impl FnOnce(Reply<T>) -> Call) -> Result<T> {
        let (reply, answer) = oneshot::channel();
        self.calls.send(make(reply)).map_err(|_| gone())?;
        match tokio::time::timeout(self.call_limit, answer).await {
            Ok(got) => got.map_err(|_| gone())?,
            Err(_) => Err(timed_out(self.call_limit)),
        }
    }
}

fn socket_name(socket: &Path) -> String {
    socket.display().to_string()
}

fn gone() -> Error {
    Error::Stored("the people server went away".into())
}
#[async_trait]
impl People for RemotePeople {
    fn capabilities(&self) -> Capabilities {
        // The server is the judge: a refusal comes back as the problem it is.
        let mut report = Capabilities::new("fs-people (socket)");
        for id in [
            "person.read",
            "person.create",
            "person.update",
            "person.delete",
        ] {
            report.supported(id);
        }
        report
    }

    async fn get(&self, id: ObjectId) -> Result<Option<StoredObject>> {
        self.ask(|r| Call::Get(id, r)).await
    }

    async fn list(&self, filter: &ListFilter) -> Result<Vec<StoredObject>> {
        self.ask(|r| Call::List(filter.clone(), r)).await
    }

    async fn create(&self, card: Card) -> Result<StoredObject> {
        let text = serde_json::to_string(&card)?;
        self.ask(|r| Call::Create(text, r)).await
    }

    async fn update(&self, id: ObjectId, card: Card) -> Result<StoredObject> {
        let text = serde_json::to_string(&card)?;
        self.ask(|r| Call::Update(id, text, r)).await
    }

    /// Compare and write in one step at the server, so another window's change is a conflict. The
    /// revision sent is the server's own of the last read of this person (`expected`, computed from a
    /// rebuilt object, would not equal it).
    async fn update_checked(
        &self,
        id: ObjectId,
        _expected: Revision,
        card: Card,
    ) -> Result<StoredObject> {
        let text = serde_json::to_string(&card)?;
        let known = self
            .revisions
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .get(&id)
            .copied();
        // Never read here: there is nothing to compare with, so nothing is written blindly.
        let Some(revision) = known else {
            return Err(Error::Changed(id));
        };
        self.ask(|r| Call::UpdateIf(id, revision, text, r)).await
    }

    async fn delete(&self, id: ObjectId) -> Result<()> {
        self.ask(|r| Call::Delete(id, r)).await
    }

    /// The server's list with a term searches name, nickname, e-mail, phone and organisation.
    async fn search(&self, term: &str) -> Result<Vec<StoredObject>> {
        let filter = ListFilter {
            term: Some(term.to_string()),
            ..ListFilter::default()
        };
        self.ask(|r| Call::List(filter, r)).await
    }
}

#[async_trait]
impl RoleBook for RemotePeople {
    async fn roles_of(&self, person: ObjectId) -> Result<Vec<RoleAssignment>> {
        self.ask(|r| Call::RolesOf(person, r)).await
    }

    async fn role(&self, id: &str) -> Result<Option<RoleDefinition>> {
        let id = id.to_string();
        self.ask(|r| Call::Role(id, r)).await
    }

    async fn assign_role(
        &self,
        person: ObjectId,
        role: &str,
        scope: Option<&str>,
        details: serde_json::Map<String, serde_json::Value>,
    ) -> Result<RoleAssignment> {
        // The wire carries `details` for roles this program owns; none of the form's roles does.
        if !details.is_empty() {
            return Err(Error::Conflict(
                "role details are not sent over the socket yet".into(),
            ));
        }
        let (role, scope) = (role.to_string(), scope.map(str::to_string));
        self.ask(|r| Call::Assign(person, role, scope, r)).await
    }

    async fn unassign_role(&self, assignment: ObjectId) -> Result<()> {
        self.ask(|r| Call::Unassign(assignment, r)).await
    }
}
