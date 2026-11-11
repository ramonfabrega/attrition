# Item 4152, family 14 (`queued`): adjudication

- **Family**: 14, slug `queued`. Leaves: `scholar_militia` (+0x9f0); `barracks_queued` (+0xa10), `stable_queued` (+0xa14), `factory_queued` (+0xa18), `combat_queued` (+0xa1c), `dock_queued` (+0xa20), `air_queued` (+0xa24); all `int`.
- **Adjudicator**: Opus 5.5 (claude-opus-5-5), wave 3, 2026-11-11.
- **Inputs**: `queued-reading.md` (first), `queued-second.md` (blind second), the decompile export, the listing (`aa/exe.s`, the full `.text` dump) and `crates/sim`.
- **Asked to settle by name**: (a) the zero-floor predicate: does the guard skip a negative step only when the counter is exactly 0, so a step of -n from a positive value can go below zero; (b) the first reading's possible permanent +1 to `scholar_militia` from a recycled slot in `Unit::init (00612100)`.

Written per leaf as settled; the verdict table closes the file.

