# fs-people – Design

Pattern: Strangler Fig – the data package and the module `person` take over function by function; the old code stays reachable
in the branch `archiv/2026-11-27/pre-data-only` (commit `b3e6381b5f7a77ae1cf892bcccced85e5b47e7c7`). Deviation: the old trunk is deleted at once instead of running in parallel.

fs-people is a program without code: a blueprint (`scaffold.toml`) plus forms descriptions (`forms/`). The type Profile (JSContact card,
RFC 9553) belongs to the module `person` (fs-modules-person); frontends only show it. One person is one record (the real account; a written-down
contact is only a profile of it). The earlier long design (core, server, directory, window) is in the archive branch.
