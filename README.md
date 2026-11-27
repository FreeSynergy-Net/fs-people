# FreeSynergy People – data package

fs-people is a **data package**: it holds no program code. People are served by the module `person` (repo `fs-modules-person`, owner of the
JSContact card `Profile`, RFC 9553); the program is the description of how to show them.

- `scaffold.toml` – the blueprint: program `people`, the module `person` (0.0.3).
- `forms/` – the frontends as forms descriptions (list, detail, cards, contacts workbench); see `forms/README.md`.

The package is one noarch RPM `fs-people` (`fs-blueprint package rpm`, built by CI on a tag). It requires the host (`fs-starter`), the module
`fs-module-person` and `freesynergy-filesystem`.

## What moved

Version 0.0.15 removed the old Rust code (core, server, directory, command line, window, tests). Nothing is lost: the state before is the branch
`archiv/2026-11-27/pre-data-only` (commit `b3e6381b5f7a77ae1cf892bcccced85e5b47e7c7`). Until the module `person` serves them, these are not available from this package: vCard
import/export, groups, roles, same-person proposals, the sensitive mark, the command line and the local socket server.

## Licence and rules

Dual licensed `MIT OR Apache-2.0` (`LICENSE-MIT`, `LICENSE-APACHE`). `.fs-rules.toml` sets the coding rules the CI gate reads.

Written with the help of Claude (Anthropic).
