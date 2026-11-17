# Changelog

## Unreleased (collects towards 0.0.15)

### English
- L0, lift to the V1+ libraries, one source per library: `fs-types` v0.0.1 (own repository, ADR 0093; `fs_people_core::values` now points to it), `fs-interfaces` v0.0.27, `fs-rpc` v0.0.2 (the RPC runtime left `fs-interfaces`: `fs_interfaces::rpc::X` is `fs_rpc::X`, `CatalogServer` is `fs_rpc::catalog::CatalogServer`), `fs-objects` v0.0.6 (one pin, no alias), `fs-components` v0.0.27. `.fs-lock-sources.toml` with `strict = true` and `only_from_enabled = true`: the lock check is red for a library in two sources.
- The program icon for the AppImage (`packaging/net.freesynergy.People.png` and `.desktop`) is on main now.
- Not in this release step: the `people-delta` branch (person directory v1, consents) waits for the answers to questions 40-44.

- L1, core faults: (B1) a source without roles shows the "Roles" section with the code `unsupported` and the capability, a failing one shows the failure (was: the section silently vanished); (B4) the vCard export is always 4.0, also for a card that came in as 3.0 (found by the new test: the export wrote `VERSION:3.0`), the stored original stays byte for byte; (B5) no fall back to a plain `update`: a source without a compare-and-set update locks editing with `people/no-checked-update` and writes nothing, two windows saving from one revision: exactly one wins; (B6) the import report of the window says new / known / failed / unclear (the unclear list was dropped before); (B8) test over the socket and a store file: a turned down proposal does not come back after a restart, taking a link back counts as a no (README corrected); roles `manager` and `owner` (ADR 0095, flat, defined by fs-groups) in `known.json`; `supervision::is_supervised` answers `Some(false)` while supervision is not built (`SUPERVISION_NOT_BUILT`), with a compile-time assertion and the red test `supervision_gate`.
- `fs-components` v0.0.28 (menu model, context menu, person picker, file chooser come with it).

- L2: the window's calls to the server have limits (start and each call 10 s); a stuck server ends in the message `timeout` with `limit-ms`. Not yet: a work thread (needs an asynchronous source in the workbench) and the question on closing with an unsaved draft (needs a close hook in `fs-components-app`); both are noted as gaps in DESIGN.

- L3 (main view, B7 for the visible texts): the window has the navigation on the left (All contacts, Groups: a page that says plainly that groups live in FreeSynergy Groups), the list and the detail next to it. All words of the window, the labels of the detail sheet and the problems of e-mail and phone are German and English (`texts/{de,en}.mf2`, ADR 0057, language of the session); the raw name "vCard" is no longer listed as "Not shown here", the sections read "More on the card" / "Kept as it came". `--screen all|groups` for a picture of a screen. Not yet: the status line after an import and the CLI texts are English only; no dark picture (the picture mode ignores `FS_COLOR_SCHEME`, card on the board); no header menu, context menu or shortcuts (L8).

### Deutsch
- L3 (Hauptansicht, B7 für die sichtbaren Texte): das Fenster hat links die Navigation (Alle Kontakte, Gruppen: eine Seite, die offen sagt, dass Gruppen in FreeSynergy Groups liegen), daneben Liste und Detail. Alle Worte des Fensters, die Beschriftungen des Detailblatts und die Hinweise zu E-Mail und Telefon gibt es deutsch und englisch (`texts/{de,en}.mf2`, ADR 0057, Sprache der Sitzung); der Rohbegriff "vCard" steht nicht mehr als "Not shown here", die Abschnitte heißen "Weiteres auf der Karte" / "So übernommen, wie es kam". `--screen all|groups` für ein Bild einer Seite. Noch nicht: die Statuszeile nach einem Import und die Texte der Befehlszeile sind nur englisch; kein dunkles Bild (der Bildmodus ignoriert `FS_COLOR_SCHEME`, Karte auf dem Board); kein Kopfmenü, Kontextmenü, keine Kürzel (L8).
- L2: die Aufrufe des Fensters an den Server haben Fristen (Start und je Aufruf 10 s); ein hängender Server endet in der Meldung `timeout` mit `limit-ms`. Noch nicht: Arbeits-Thread (braucht eine asynchrone Quelle im Workbench) und die Rückfrage beim Schließen mit ungespeichertem Entwurf (braucht einen Schließ-Haken in `fs-components-app`); beides steht als Lücke in DESIGN.
- L1, Kernfehler: (B1) eine Quelle ohne Rollen zeigt den Abschnitt "Rollen" mit dem Code `unsupported` und der Fähigkeit, eine fehlerhafte zeigt den Fehler (vorher verschwand der Abschnitt still); (B4) der vCard-Export ist immer 4.0, auch für eine als 3.0 gekommene Karte (der neue Test fand: der Export schrieb `VERSION:3.0`), das gespeicherte Original bleibt Byte für Byte; (B5) kein Rückfall auf ein einfaches `update`: eine Quelle ohne atomares Prüfen-und-Schreiben sperrt das Bearbeiten (`people/no-checked-update`) und schreibt nichts, zwei Fenster mit gleichem Stand: genau eines gewinnt; (B6) der Importbericht des Fensters nennt neu / bekannt / fehlerhaft / unklar (die Liste "unklar" fiel bisher weg); (B8) Test über Socket und Speicherdatei: ein verworfener Vorschlag kommt nach einem Neustart nicht wieder, ein zurückgenommener Link zählt als Nein (README berichtigt); Rollen `manager` und `owner` (ADR 0095, flach, von fs-groups definiert) in `known.json`; `supervision::is_supervised` antwortet `Some(false)`, solange Betreuung nicht eingebaut ist (`SUPERVISION_NOT_BUILT`), mit Übersetzungszeit-Prüfung und dem roten Test `supervision_gate`.
- `fs-components` v0.0.28 (Menümodell, Kontextmenü, Personenauswahl, Datei-Wähler kommen damit).

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
