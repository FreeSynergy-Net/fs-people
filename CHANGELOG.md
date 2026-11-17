# Changelog

## Unreleased

### English
- Sorted, NOT compiled (workflow `geruest-heben`, no build, no gates yet): logic moved out of the frontend `fs-people-app` into the program and modules. New crates `fs-people-client` (socket client of the contract, was `source/remote/`), `fs-people-view` (card as form / form onto card / card as sheet, was `source/draft.rs`, `apply.rs`, `sheet.rs`, plus the labels and the error reasons) and `fs-people-vcard` (vCard file exchange, was `transfer.rs`); the sensitive-marking check moved into `fs-people-core` (`card::marked`, was `source/marks.rs`). `fs-people-app` re-exports the old names. Table: `.scratch/katalog-plan/sortierung/people.md`. `Cargo.lock` is not updated; the first build will find what is left.

### Deutsch
- Sortiert, NICHT kompiliert (Workflow `geruest-heben`, kein Build, noch keine Gates): Logik aus dem Frontend `fs-people-app` ins Programm und in Module verschoben. Neue Crates `fs-people-client` (Socket-Client des Vertrags, vorher `source/remote/`), `fs-people-view` (Karte als Formular / Formular auf Karte / Karte als Blatt, vorher `source/draft.rs`, `apply.rs`, `sheet.rs`, dazu Beschriftungen und Fehlergründe) und `fs-people-vcard` (vCard-Dateiaustausch, vorher `transfer.rs`); die Prüfung der Sensibel-Markierung liegt jetzt in `fs-people-core` (`card::marked`, vorher `source/marks.rs`). `fs-people-app` führt die alten Namen weiter. Tabelle: `.scratch/katalog-plan/sortierung/people.md`. `Cargo.lock` ist nicht nachgezogen; der erste Build zeigt, was noch fehlt.

## 0.0.14

### English
- `--version`/`-V` for the release smoke test.
- Earlier versions: see the Git tags.

### Deutsch
- `--version`/`-V` für den Release-Smoke-Test.
- Frühere Versionen: siehe Git-Tags.
