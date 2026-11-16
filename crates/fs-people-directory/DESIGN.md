# fs-people-directory (design)

The light consumer side of People (ADR 0085 section 3, ADR 0087, plan O2). Calendar, tasks and groups hold only a
`PersonRef` and ask this trait for display, suggestions, proposals, the asking person and an excerpt for a frozen
snapshot. No Blinc, no app, no store.

## Contents

| Part | What | Pattern |
| --- | --- | --- |
| `src/person_directory_v1/` | The LOCKED trait, copied byte for byte (hash `dbbc4b8d8c1c93bc6eb343eb87c121d99ad9eba3857eed24764d9a842219947a`, lock file `.scratch/sperren/person_directory_v1.sha256`). Never edited; a change is `person_directory_v2`. | Repository + Null Object |
| `fake/` | `FakeDirectory`: in memory, every method, change log, trash (`delete`, `unshare`, `restore`), `merge`, batches | Fake |
| `composite.rs` | `CompositeDirectory`: asks in order; `Unknown` and `Unavailable` pass on; `propose` to the first member with `person.propose` | Composite + Chain of Responsibility |
| `fake/sample.rs` | `sample()` and `sample_ref(n)`: twelve entries covering every state (table in the file), the shared test data of calendar adapter, tasks and groups | Object Mother |
| `doubles/` | `FailingDirectory` (one fixed `Error`; `each_variant()` reaches every variant) and `SlowDirectory` + `Gate` (calls wait at a latch the test opens; a dropped future never reaches the inner provider) | Test double, Decorator |
| `conformance/` | `run_conformance(dir)` (rules that need only the trait, data seeded through `propose`), `run_fixture_conformance(fixture)` (plus the state rules), `run_unsupported(dir)` | Template method, Strategy |
| `conformance/fixture.rs` | `ConformanceFixture`: the mutators (`add_confirmed`, `delete`, `unshare`, `restore`, `merge`, `rename`, `foreign_book`) a provider offers to put itself into the states the rules need; a missing mutator makes the rule `skipped` in the `Report`, never passed. `FakeFixture` is the fake's; `PeopleRepo` gets its own in P0b | Strategy |

## Rules worth knowing

- Who asks is a property of the provider (`asker`), never a parameter.
- `resolve` never fails for "not found": `Unknown`, `Deleted`, `NotShared`. Only `Unknown` lets the composite ask on.
- `changes`: `next` is always `Some`, `size == 0` is `Invalid`; a composite hands through the feed of its first
  reachable member (opaque cursors belong to one provider; a merged feed needs a cursor of its own).
- `excerpt`: `NoExcerpt` carries the reason; `Unknown` lets the composite ask on, `Deleted` and `NotShared` end it.
- Composite, mixed answers: `Unavailable` mixed with `Unknown` gives `Unavailable` (a chain cannot claim "unknown" while a member could not answer). `Unsupported` of a member (`changes`, `excerpt`) passes the question on, the same everywhere; if nobody can, the answer is `Unsupported`.
- Composite `resolve_many` asks each member once, with the entries that are still `Unknown`. `suggest` checks `limit` itself (so a chain that is all down still says `Invalid` for 51) and merges the lists by position: every first suggestion in member order, then every second one, duplicates keep their first place (a candidate has no rank of its own to compare across providers).
- Composite `changes` hands through the feed of its first member that can answer (an own cursor for a merged feed is a later step).
- Fake: `excerpt` and `resolve` follow a merge, `merge` and `rename` raise `rev`, `rename` reports `Changed`, new ids (proposals, batches) are UUIDv7 with random bits, `tel` addresses compare by their digits (E.164).
- The fake keeps its rows by creation time (a tick clock), so "oldest first" is deterministic.

## Who expects what (calendar adapter, not in this crate)

The adapter `appointments_v1::PersonDirectory` to `person_directory_v1::PersonDirectory` is ONE file in fs-calendars
(plan step C1, O3): it converts `PersonRef`, `ContactAddress` and `Utc` between the two shapes and holds a
`Box<dyn person_directory_v1::PersonDirectory>`. fs-calendars tests it against `FakeDirectory` from this crate;
this crate never depends on fs-calendars. Tasks (`assignee`) and groups (members, `PersonArt::Group` via
`GroupsDirectory`) depend on this crate the same way. People itself provides the real `PeopleDirectory`.

## Planned (not in P0)

- P0b (only AFTER the lift of fs-people-core and fs-people-server to fs-types v0.0.22, step L0: the lock holds v0.0.22 and v0.0.14 side by side, and the two are different types): `PeopleDirectory` over the current `PeopleRepo` (store v4): resolve by uid, suggest over search plus rank,
  propose as a card with `state: proposed`, excerpt from the card, changes over `PeopleFeed`. It lives in
  `fs-people-core` or `fs-people-server`, not here, so this crate stays light.
- P0c: `people.capnp` and `RemoteDirectory`.
- P0b also brings a `ConformanceFixture` over `PeopleRepo`, so the same rules run against the real provider.
- P0c: the composite's `resolve_many` bundling pays off with `RemoteDirectory` (one socket round trip per member).
