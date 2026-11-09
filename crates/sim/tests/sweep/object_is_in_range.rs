//! `ObjectData::is_in_range` — **not swept: it needs a packet** (item 1575).
//! Both of the batch's subagents stopped at the same fault: the wrapper
//! decodes its point and calls the body at `0x006486b0`, whose first read is
//! the global `objects` row table (an unmapped read at eip `0x006486d6`),
//! then the target's vtable, the `units` table, `worldc`'s terrain array and
//! `attack_dist`. A frame packet on a frame where a ranged unit attacks a
//! building (`tools/explore/frame_snapshot.py`, `tools/recomp/step4.py`) is
//! the rung; this module holds no test until one is taken.
