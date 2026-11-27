# fs-people: forms descriptions of the frontends (not yet imported)

A program is a blueprint plus forms descriptions (ADR 0096). The frontends of fs-people are **descriptions**, never a frontend
crate: names of components, slots, fields and MF2 text keys, no code. The files are the **import form**. They show different data
of the same module `person` (type `Profile`, RFC 9553 card; one person = one record). Copied from fs-forms main 0841aeb (`forms/people/`).

| File | Frontend id | Block | Pages |
|---|---|---|---|
| `P-F1-liste.toml` | `people-list` (named in `scaffold.toml`) | rows 1.0.0 | table: name and short fields |
| `P-F2-einzel.toml` | `people-detail` | `detail-page`, single-record 1.2.0 | all contact fields of ONE person |
| `P-F3-karten.toml` | `people-cards` | `tile-grid`, tiles 1.1.0 | one tile per person |
| `P-F4-kontakte.toml` | `people-contacts` | rows 1.0.0 | contacts workbench (list and sheet) |

Format: layout format 4 (`schema = 4`), range spelling `person@>=0.0.1, <0.1.0`. fs-tasks uses format 3 (see `forms/PLATZHALTER.md`).
