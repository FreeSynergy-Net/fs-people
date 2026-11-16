# fs-people-directory (design)

The light consumer side of People (ADR 0085 section 3, ADR 0087, plan O2). Calendar, tasks and groups hold only a
`PersonRef` and ask this trait for display, suggestions, proposals, the asking person and an excerpt for a frozen
snapshot. No Blinc, no app, no store.

## Contents

| Part | What | Pattern |
| --- | --- | --- |
| `src/person_directory_v1/` | The trait at its working fixpoint (hash `6c885cfe515a3d8c195a0ddcf43f19602d84e8342e29cca7d4297ae808037fe0`), copied byte for byte from the strategy's folder. Not edited here; the strategy moves the fixpoint (new hash), a freeze comes only on the owner's word. | Repository + Null Object |
| `fake/` | `FakeDirectory`: in memory, every method, change log, trash (`delete`, `unshare`, `restore`), `merge`, batches, consent records with the matching rule, purpose binding in `excerpt`, the AI rules, `view_as(asker, Rights)` for a second asker over the same data; `Defect` = one deliberate rule break per mutant | Fake, fault injection |
| `composite.rs` | `CompositeDirectory`: asks in order; `Unknown` and `Unavailable` pass on; `propose` to the first member with `person.propose` | Composite + Chain of Responsibility |
| `fake/sample.rs` | `sample()` and `sample_ref(n)`: twelve entries covering every state (table in the file), the shared test data of calendar adapter, tasks and groups | Object Mother |
| `doubles/` | `FailingDirectory` (one fixed `Error`; `each_variant()` reaches every variant) and `SlowDirectory` + `Gate` (calls wait at a latch the test opens; a dropped future never reaches the inner provider) | Test double, Decorator |
| `conformance/` | `run_conformance(dir)` (rules that need only the trait, data seeded through `propose`), `run_fixture_conformance(fixture)` (plus the state rules), `run_unsupported(dir)` | Template method, Strategy |
| `conformance/fixture.rs` | `ConformanceFixture`: the mutators (`add_confirmed`, `delete`, `unshare`, `restore`, `merge`, `rename`, `foreign_book`, `add_ai`, `set_birthday`, `describe`, `admin_view`, `member_view(who, rights)`) a provider offers to put itself into the states the rules need; a missing mutator makes the rule `skipped` in the `Report`, never passed. `FakeFixture` is the fake's; `PeopleRepo` gets its own in P0b | Strategy |

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

## Consents, purpose and the AI rule (delta of 16.11.)

- `excerpt(person, &ExcerptRequest)` carries fields, purpose, scope and recipient. A purpose with consent duty
  (`Purpose::needs_consent`) on a `Person` or `Profile` needs an active consent found by `check_consent`, else
  `NoExcerpt{NoConsent}`; `NoConsent` is final in the composite, only `Unknown` and `Unsupported` pass on.
- Matching rule (`ConsentQuery`): same purpose, `scope`/`recipient` of the record `None` or equal; merged references
  count as their target and their consents move along; a profile question also counts the person it describes, a
  profile's own consent never covers the person or another profile; a cut link stops covering.
- `record_consent` always writes an own record; `withdraw_consent` never deletes, is idempotent, and ranks an unknown or
  invisible id (`UnknownConsent`) before `Forbidden`; deleting a person withdraws their consents with `Erasure`.
- An asker that is an AI (the entry of the asker has `ai`): no sensitive fields, no address, no address stage in
  `display`, no address search; `find_by_address`, `propose` with an address, every `consent.*` method and `excerpt` with
  a purpose other than `AiAssistance` are `Forbidden` (the purpose check comes first).
- The AI mark sticks to the entry whatever its state (shared, unshared, deleted) and is looked up at the asker's id and
  at the end of its merge chain: an AI stays an AI after unshare, delete and merge.
- Retention: `retain_until` is the end of the third year after the WITHDRAWAL (`withdrawal.at`), not after the record;
  after it the provider deletes the record for good (fake: `purge_expired`, fixture: `purge_expired(today)`). The one who
  deletes a person records the `Erasure` withdrawals (`recorded_by`); the person only if nobody asks.
- Composite: each `consent.*` method goes to the first member that offers its capability. A member that offers the
  capability but is `Unavailable` is not skipped (like `propose`).
- TODO stage 2 (test needed): "first member with the capability" only works if that member knows ALL persons. With the
  consents at member A and the person at member B, `check_consent` says Some, `excerpt` says NoConsent (B checks its
  own store) and `record_consent` at A is Invalid. Decide: people-core is the consent member (knows everyone), OR the
  composite `excerpt` checks the consent through the consent member. Write the test with that decision.
- `tests/mutants.rs` runs 24 `Defect`s (`Defect` and `with_defect` are `#[doc(hidden)]`) against the rules and demands that the rule written for each one fails.

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
