# attrition

A deterministic, ground-up reimplementation of **Rise of Nations** (Big Huge
Games, 2003) in Rust — simulation first, art last.

Named for the mechanic no open-source RTS has ever implemented: units bleeding
health inside hostile national borders.

> **Status: Phase 0, pre-data.** The only thing here is the fixed-point math
> the simulation will stand on. Real work begins once the game's data layer is
> extracted.

## What this is

There is no open-source Rise of Nations engine. Not on osgameclones, not in
awesome-game-remakes, not on Wikipedia's engine-recreation list. No public
decompile, no format wiki. The field is empty.

This aims to fill it, following the arc that OpenTTD and Beyond All Reason both
completed: reimplement the engine against the original's own data files, reach
parity, then progressively replace the original assets until the game stands on
its own.

Two things make Rise of Nations an unusually good target:

- **Its design is already readable.** `rules.xml`, `unitrules.xml`, and
  `buildingrules.xml` ship as plain text — nation powers, attrition rates, pop
  caps, per-unit attack/HP/cost/speed. Twenty years of tuned balance, sitting
  in files you can open in a text editor.
- **Its art is not the point.** Nobody is nostalgic for 2003 low-poly RTS
  units. The appeal is the systems — territory, attrition, eight ages in forty
  minutes — and those survive a total art replacement intact.

## Requirements

You need your own copy of **Rise of Nations: Extended Edition** (Steam app
`287450`). No game assets are distributed here and none ever will be — the
tools read from your installed copy.

The original 2004 Mac port (Gold Edition, MacSoft) is PowerPC-era and will not
run on any current Mac. Extended Edition is Windows-only, but you do not need
Windows to *extract* its data — see below.

## Getting the game files on macOS

Extended Edition has no Mac build, but its files can be pulled down natively
with no Wine, VM, or Windows machine involved:

```sh
scripts/fetch-depot.sh
```

This uses SteamCMD with a forced Windows platform type to download the depot
without running it. Phase 1 needs the files, not a running game.

Actually *playing* it on a Mac (to generate controlled recorded games) is a
separate problem — CrossOver or Whisky, later.

## Layout

```
crates/fixed/     deterministic Q16.16 fixed-point math
docs/DECISIONS.md architectural decisions and their rationale
docs/FORMATS.md   file-format reverse-engineering log
```

Crates appear here when they have real code, not in anticipation of it.

## Development

```sh
cargo test          # 13 tests, all in crates/fixed
cargo clippy --all-targets
cargo fmt
```

The toolchain is pinned to stable in `rust-toolchain.toml`.

See `CLAUDE.md` for the full thesis, hard constraints, and phase plan.

## License

MIT OR Apache-2.0. Rise of Nations is a trademark of Microsoft; this project is
unaffiliated and contains none of its assets.
