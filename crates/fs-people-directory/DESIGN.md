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
| `conformance/` | `run_conformance(dir)` and `run_unsupported(dir)`: every rule of the folder as a check, for any provider | Template method |

## Rules worth knowing

- Who asks is a property of the provider (`asker`), never a parameter.
- `resolve` never fails for "not found": `Unknown`, `Deleted`, `NotShared`. Only `Unknown` lets the composite ask on.
- `changes`: `next` is always `Some`, `size == 0` is `Invalid`; a composite hands through the feed of its first
  reachable member (opaque cursors belong to one provider; a merged feed needs a cursor of its own).
- `excerpt`: `NoExcerpt` carries the reason; `Unknown` lets the composite ask on, `Deleted` and `NotShared` end it.
- The fake keeps its rows by creation time (a tick clock), so "oldest first" is deterministic.

## Who expects what (calendar adapter, not in this crate)

The adapter `appointments_v1::PersonDirectory` to `person_directory_v1::PersonDirectory` is ONE file in fs-calendar
(plan step C1, O3): it converts `PersonRef`, `ContactAddress` and `Utc` between the two shapes and holds a
`Box<dyn person_directory_v1::PersonDirectory>`. fs-calendar tests it against `FakeDirectory` from this crate;
this crate never depends on fs-calendar. Tasks (`assignee`) and groups (members, `PersonArt::Group` via
`GroupsDirectory`) depend on this crate the same way. People itself provides the real `PeopleDirectory`.

## Planned (not in P0)

- P0b: `PeopleDirectory` over the current `PeopleRepo` (store v4): resolve by uid, suggest over search plus rank,
  propose as a card with `state: proposed`, excerpt from the card, changes over `PeopleFeed`. It lives in
  `fs-people-core` or `fs-people-server`, not here, so this crate stays light.
- P0c: `people.capnp` and `RemoteDirectory`.
- `sample()` with the twelve plan entries (person, linked and unlinked profile, AI, group, organisation, deleted,
  unshared, merged, proposed, two profiles with one address) for the consumers' tests.
