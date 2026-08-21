# Orders — the seven first readings (2026-08-20/21)

The reading notes `docs/ORDERS.md` was synthesised from, one per sub-area,
kept here because the blind second reading and its adjudication had not yet
run when the session ended — the adjudicator will want the first readers'
own words beside the document and the blind reports:

- `R1-orderlist.md` — the order list, `Unit::process`/`work`/`do_job`, idle.
- `R2-move.md` — `MoveOrder`, `do_move`, `find_path`, the pathfinder seam.
- `R3-build-garrison.md` — build/repair/garrison orders, `come_out`, `think_peasant`.
- `R4-gather.md` — the gather registration, `do_gather`, wood/ore, the farm.
- `R5-setup.md` — the start of a game, the id scheme, frame numbering.
- `R6-attack-group.md` — the attack order and `fight`, the group orders, the command stream.
- `R7-spot.md` — `find_nearby_spot`, `invalid_loc`, `adjacent_to`, collision's interface.

All seven were `lean` subagents on Fable 5 reading the decompile export; two
(R1, R6) dropped on API connection errors mid-run — R6 was resumed from its
transcript and finished, R1 was relaunched fresh after a second drop. Like
the audit files, these are reading notes: short expressions are quoted where
a claim turns on one, nothing is transcribed, nothing here is a source to
implement from.
