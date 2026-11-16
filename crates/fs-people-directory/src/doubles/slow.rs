//! `SlowDirectory`: holds every call at a gate until the test opens it; dropping the future meanwhile is a
//! cancellation (a time limit in the consumer). No timer and no runtime: the test decides when time passes.
//!
//! Pattern: Decorator over `PersonDirectory` (delays, then delegates) with a hand-made latch.

use crate::person_directory_v1::{
    Candidate, Capabilities, ContactAddress, DirectoryChange, Excerpt, ExcerptField, ObjectId,
    Page, PersonDirectory, PersonRef, PersonView, Place, ProposeEntry, Reference, Resolved, Result,
    Suggest,
};
use async_trait::async_trait;
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::task::{Context, Poll, Waker};

#[derive(Default)]
struct GateState {
    open: bool,
    wakers: Vec<Waker>,
}

/// A latch shared between the test and the provider; clones share the state.
#[derive(Clone, Default)]
pub struct Gate(Arc<Mutex<GateState>>);

impl Gate {
    /// A closed gate.
    #[must_use]
    pub fn closed() -> Self {
        Self::default()
    }

    /// Let every waiting and every later call through.
    pub fn open(&self) {
        let mut g = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        g.open = true;
        for w in g.wakers.drain(..) {
            w.wake();
        }
    }

    fn wait(&self) -> Wait {
        Wait(self.clone())
    }
}

struct Wait(Gate);

impl Future for Wait {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        let mut g = self.0.0.lock().unwrap_or_else(PoisonError::into_inner);
        if g.open {
            Poll::Ready(())
        } else {
            g.wakers.push(cx.waker().clone());
            Poll::Pending
        }
    }
}

/// Wraps a provider; every call waits at the gate first.
pub struct SlowDirectory<D> {
    inner: D,
    gate: Gate,
    passed: AtomicUsize,
}

impl<D: PersonDirectory> SlowDirectory<D> {
    /// Hold the calls of `inner` at `gate`.
    #[must_use]
    pub fn new(inner: D, gate: Gate) -> Self {
        Self {
            inner,
            gate,
            passed: AtomicUsize::new(0),
        }
    }

    /// How many calls got through the gate to the inner provider (a cancelled call never does).
    #[must_use]
    pub fn passed(&self) -> usize {
        self.passed.load(Ordering::SeqCst)
    }

    async fn hold(&self) {
        self.gate.wait().await;
        self.passed.fetch_add(1, Ordering::SeqCst);
    }
}

#[async_trait]
impl<D: PersonDirectory> PersonDirectory for SlowDirectory<D> {
    fn capabilities(&self) -> Capabilities {
        self.inner.capabilities()
    }
    async fn resolve(&self, person: &PersonRef) -> Result<Resolved> {
        self.hold().await;
        self.inner.resolve(person).await
    }
    async fn asker(&self) -> Result<Option<PersonRef>> {
        self.hold().await;
        self.inner.asker().await
    }
    async fn resolve_many(&self, people: &[PersonRef]) -> Result<Vec<Resolved>> {
        self.hold().await;
        self.inner.resolve_many(people).await
    }
    async fn changes(&self, since: Option<&Place>, size: usize) -> Result<Page<DirectoryChange>> {
        self.hold().await;
        self.inner.changes(since, size).await
    }
    async fn find_by_address(&self, address: &ContactAddress) -> Result<Vec<PersonView>> {
        self.hold().await;
        self.inner.find_by_address(address).await
    }
    async fn suggest(&self, query: &Suggest) -> Result<Vec<Candidate>> {
        self.hold().await;
        self.inner.suggest(query).await
    }
    async fn open_batch(&self, origin: &Reference) -> Result<ObjectId> {
        self.hold().await;
        self.inner.open_batch(origin).await
    }
    async fn propose(&self, entry: &ProposeEntry, batch: Option<ObjectId>) -> Result<PersonRef> {
        self.hold().await;
        self.inner.propose(entry, batch).await
    }
    async fn excerpt(&self, person: &PersonRef, fields: &[ExcerptField]) -> Result<Excerpt> {
        self.hold().await;
        self.inner.excerpt(person, fields).await
    }
}
