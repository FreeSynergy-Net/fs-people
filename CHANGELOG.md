# Changelog

## Unreleased (collects towards 0.0.15)

### English
- L0, lift to the V1+ libraries, one source per library: `fs-types` v0.0.1 (own repository, ADR 0093; `fs_people_core::values` now points to it), `fs-interfaces` v0.0.27, `fs-rpc` v0.0.2 (the RPC runtime left `fs-interfaces`: `fs_interfaces::rpc::X` is `fs_rpc::X`, `CatalogServer` is `fs_rpc::catalog::CatalogServer`), `fs-objects` v0.0.6 (one pin, no alias), `fs-components` v0.0.27. `.fs-lock-sources.toml` with `strict = true` and `only_from_enabled = true`: the lock check is red for a library in two sources.
- The program icon for the AppImage (`packaging/net.freesynergy.People.png` and `.desktop`) is on main now.
- Not in this release step: the `people-delta` branch (person directory v1, consents) waits for the answers to questions 40-44.

### Deutsch
- L0, Anhebung auf die V1+-Bibliotheken, eine Quelle je Bibliothek: `fs-types` v0.0.1 (eigenes Repository, ADR 0093; `fs_people_core::values` zeigt dorthin), `fs-interfaces` v0.0.27, `fs-rpc` v0.0.2 (die RPC-Laufzeit ist aus `fs-interfaces` ausgezogen: `fs_interfaces::rpc::X` ist jetzt `fs_rpc::X`), `fs-objects` v0.0.6 (ein Pin, kein Alias), `fs-components` v0.0.27. `.fs-lock-sources.toml` mit `strict = true`: eine Bibliothek in zwei Quellen macht die Prüfung rot.
- Das Programm-Symbol für das AppImage (`packaging/net.freesynergy.People.png` und `.desktop`) liegt jetzt auf main.
- Nicht in diesem Schritt: der Zweig `people-delta` (Personenverzeichnis v1, Einwilligungen) wartet auf die Antworten zu den Fragen 40-44.

## 0.0.14

### English
- `--version`/`-V` for the release smoke test.
- Earlier versions: see the Git tags.

### Deutsch
- `--version`/`-V` für den Release-Smoke-Test.
- Frühere Versionen: siehe Git-Tags.
