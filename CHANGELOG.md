# Changelog

## 0.0.15

### English
- Data package (workflow workflow-pflege, card C3): fs-people holds no code any more. Removed: `crates/` (fs-people-core, -server, -cli, -app, -directory, fs-people), `Cargo.*`, `.cargo/`, `clippy.toml`, empty placeholder folders, the two `[[exception]]` entries and all own types of `scaffold.toml`. State before: branch `archiv/2026-11-27/pre-data-only` (`b3e6381b5f7a77ae1cf892bcccced85e5b47e7c7`).
- Functions that left (cards): vCard import/export (old core); JSContact card store with unknown properties kept; create/edit/delete/search; groups and several roles per person (ai, user, member, customer, supplier, lead, prospect); relations on the card; same-person proposals (match rules, dialog); sensitive mark and release at export; local socket server (contract `people`); command line; the provisional window; the 200+ tests.
- `scaffold.toml`: program 0.0.15, module `person` 0.0.3 (RPM requires `fs-module-person >= 0.0.3`), type `Profile` on the module, `[frontend]` = `forms/P-F1-liste.toml`. `forms/`: four descriptions P-F1..P-F4 (copied from fs-forms 0841aeb, format 4). CI callers pinned to fs-release `8033f1d1cdc4cd7da83222ea41b5a0d9ea37b6fa`, `release.yml` `kind: data`. Written with the help of Claude (Anthropic).

### Deutsch
- Daten-Paket (Workflow workflow-pflege, Karte C3): fs-people enthält keinen Code mehr. Entfernt: `crates/`, `Cargo.*`, `.cargo/`, `clippy.toml`, leere Platzhalter-Ordner, die zwei `[[exception]]`-Einträge und alle eigenen Typen in `scaffold.toml`. Stand davor: Branch `archiv/2026-11-27/pre-data-only` (`b3e6381b5f7a77ae1cf892bcccced85e5b47e7c7`).
- Entfallene Funktionen (Karten): vCard-Import/-Export; JSContact-Kartenspeicher mit erhaltenen unbekannten Eigenschaften; Anlegen/Bearbeiten/Löschen/Suchen; Gruppen und mehrere Rollen je Person; Beziehungen auf der Karte; Gleiche-Person-Vorschläge (Regeln, Dialog); Sensibel-Markierung und Freigabe beim Export; lokaler Socket-Server (Vertrag `people`); Kommandozeile; das vorläufige Fenster; die über 200 Tests.
- `scaffold.toml`: Programm 0.0.15, Modul `person` 0.0.3 (RPM verlangt `fs-module-person >= 0.0.3`), Typ `Profile` am Modul, `[frontend]` = `forms/P-F1-liste.toml`. `forms/`: vier Beschreibungen P-F1..P-F4 (aus fs-forms 0841aeb, Format 4). CI-Aufrufer auf fs-release `8033f1d1cdc4cd7da83222ea41b5a0d9ea37b6fa` gepinnt, `release.yml` `kind: data`. Mit Hilfe von Claude (Anthropic) geschrieben.

## Unreleased

### English
- CI callers (`check.yml`, `release.yml`) pinned to fs-release v0.0.17 (was v0.0.15). No code change.

### Deutsch
- CI-Aufrufer (`check.yml`, `release.yml`) auf fs-release v0.0.17 gehoben (vorher v0.0.15). Keine Code-Änderung.

## 0.0.14

### English
- `--version`/`-V` for the release smoke test.
- Earlier versions: see the Git tags.

### Deutsch
- `--version`/`-V` für den Release-Smoke-Test.
- Frühere Versionen: siehe Git-Tags.
