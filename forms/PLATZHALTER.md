# Placeholders

| What | Why | Next step |
|---|---|---|
| P-F2, P-F3 need the blocks `detail-page` and `tile-grid` in the host | drawn only with fs-components 4a923bbc or newer (C2 report) | host with that catalog |
| P-F2..P-F4 are not named in `scaffold.toml` | the blueprint schema has ONE `[frontend]` block | extend fs-blueprint to a list of frontends |
| Cards have no command (copy, mail, open) | `open` and `actions` of single-record and tiles are not served yet (card to C4) | module step |
| Format 4 vs fs-tasks format 3; range lower bound 0.0.1 vs package 0.0.3 | descriptions copied unchanged | decide: keep format 4 or convert; raise bound to `>=0.0.3` |
| Functions of the old code (vCard, groups, roles, proposals, sensitive mark) | code removed | see CHANGELOG 0.0.15 |
