//! **A person's whole game: the 2017 sample** (item 4336, `docs/RECGAME.md`
//! §7; DECISIONS 139 §3). `seifer008 - Toughest AI 4 v 1.rcx` (HeavenGames
//! file 2170, under the install's gitignored `external-recgames/`): 21,044
//! frames, a person at leader 3 against four computers on Great Sahara.
//! The original plays it whole (run900); run1650 retakes its opening with
//! the start dump (`--dump-all-start`, frames 0..36).
//!
//! Two rows. **The start built from the recording's header** — the header's
//! lobby read as a `GAMEINFO` block ([`header_lobby`]), first held to the
//! playback's own `GAMEINFO` field by field, then handed to
//! [`super::start::from_lobby`] and compared record by record with the
//! playback's start block as [`build_sim`] installs it. **The walk's word**
//! over the playback's draws is the second, minted once the first stands.

use crate::recgame::{OPTION_KEYS, RecGame};

/// A `GAMEINFO` block's keyed fields, and its `PLAYER` rows'.
pub type Lobby = (Vec<(String, String)>, Vec<Vec<(String, String)>>);
/// The same, borrowed, as a parsed dump holds them.
pub type LobbyRef<'a, 'b> = (&'a [(&'b str, &'b str)], &'a [Vec<(&'b str, &'b str)>]);

/// The recording's header as the `GAMEINFO` block the playback prints
/// (`GameLog::begin_game`'s `GAME INFO`): the version lines, the thirty
/// options in `OPTION_KEYS`' order, the seed, the checksum triple, the
/// flags, and one `PLAYER` row per active slot in slot order, each row's
/// keys the ones [`crate::recgame::from_lobby`] reads back. The slot's name
/// is not a keyed field and is left out.
pub fn header_lobby(rec: &RecGame) -> Lobby {
    let mut info: Vec<(String, String)> = Vec::new();
    let mut put = |k: &str, v: String| info.push((k.to_string(), v));
    let version = rec.version.clone();
    match version.split_once(' ') {
        Some((k, v)) => put(k, v.to_string()),
        None => put(&version, String::new()),
    }
    put("(int)", format!("version {}", rec.info_version));
    for (k, key) in OPTION_KEYS.iter().enumerate() {
        put(key, rec.options[k].to_string());
    }
    put("(int)seed", rec.seed.to_string());
    put("checksum_deep", rec.checksum[0].to_string());
    put("checksum_window_size", rec.checksum[1].to_string());
    put("checksum_failure_threshold", rec.checksum[2].to_string());
    put("flags", rec.flags.to_string());
    let players = rec
        .slots
        .iter()
        .filter(|s| s.active())
        .map(|s| {
            let mut row: Vec<(String, String)> = Vec::new();
            let mut put = |k: &str, v: String| row.push((k.to_string(), v));
            put("flags", s.flags.to_string());
            put("tribe", s.tribe.to_string());
            put("who", s.who.to_string());
            put("team", s.team.to_string());
            put("handicap", s.handicap.to_string());
            put("play", s.play.to_string());
            put("pauses", s.pauses.to_string());
            put("diff", s.diff.to_string());
            put("pop_cap_frame", s.pop_cap_frame.to_string());
            for (key, v) in [
                "synced_used_zoom_control",
                "synced_frames_zoomed_in",
                "synced_frames_zoomed_out",
                "synced_clicks",
                "synced_hotkeys",
                "synced_minimap_clicks",
                "synced_mainmap_clicks",
                "synced_cheated",
                "synced_control_groups_formed",
                "synced_control_groups_activated",
            ]
            .iter()
            .zip(s.synced)
            {
                put(key, v.to_string());
            }
            row
        })
        .collect();
    (info, players)
}

/// Every keyed field the two `GAMEINFO` blocks disagree on, `key: header
/// <value> playback <value>`, a `PLAYER` row's as `PLAYER <n> key`. A key absent on one
/// side prints `—` there.
pub fn compare_lobby(a: &Lobby, b: LobbyRef<'_, '_>) -> Vec<String> {
    let mut out = Vec::new();
    let side = |v: Option<&str>| v.map_or("—".to_string(), |v| v.trim().to_string());
    let mut cmp = |label: String, x: Option<&str>, y: Option<&str>| {
        if x.map(str::trim) != y.map(str::trim) {
            out.push(format!("{label}: header {} playback {}", side(x), side(y)));
        }
    };
    let keys: Vec<&str> =
        a.0.iter()
            .map(|(k, _)| k.as_str())
            .chain(b.0.iter().map(|(k, _)| *k))
            .fold(Vec::new(), |mut v, k| {
                if !v.contains(&k) {
                    v.push(k);
                }
                v
            });
    for k in keys {
        let x = a.0.iter().find(|(kk, _)| kk == k).map(|(_, v)| v.as_str());
        let y = b.0.iter().find(|(kk, _)| *kk == k).map(|(_, v)| *v);
        cmp(k.to_string(), x, y);
    }
    cmp(
        "PLAYER rows".into(),
        Some(a.1.len().to_string().as_str()),
        Some(b.1.len().to_string().as_str()),
    );
    for (n, (ra, rb)) in a.1.iter().zip(b.1).enumerate() {
        for (k, x) in ra {
            let y = rb.iter().find(|(kk, _)| kk == k).map(|(_, v)| *v);
            cmp(format!("PLAYER {n} {k}"), Some(x.as_str()), y);
        }
    }
    out
}

#[cfg(test)]
pub(in crate::diff) mod tests {
    use super::super::*;
    use super::*;
    use crate::testenv::install;

    /// The sample, under the install's `external-recgames/` (the one `.rcx`
    /// there, `recgame::tests::sample`'s).
    fn sample() -> Option<String> {
        let root = crate::testenv::install_root()?;
        let dir = std::path::Path::new(&root).join("external-recgames");
        std::fs::read_dir(dir).ok()?.find_map(|e| {
            let p = e.ok()?.path();
            (p.extension()? == "rcx").then(|| p.display().to_string())
        })
    }

    /// run1650's playback dump, in place under `$RON_GOLDEN_DIR` (default
    /// `~/ron-golden`), as run1646's is (`docs/RUNS.md` run1650).
    fn run1650() -> Option<std::path::PathBuf> {
        let home = std::env::var("HOME").ok()?;
        let dir = std::env::var("RON_GOLDEN_DIR").unwrap_or(format!("{home}/ron-golden"));
        let p = std::path::PathBuf::from(format!("{dir}/run1650/map-7/gamelog_p.txt"));
        p.is_file().then_some(p)
    }

    /// **The header's lobby against the playback's `GAMEINFO`** (item
    /// 4336): every keyed field the playback prints, header beside it.
    /// Pinned to the list of partings, so a reader of either side that
    /// moves turns it red.
    #[test]
    fn the_sample_s_header_is_the_playback_s_gameinfo() {
        let (Some(rcx), Some(path)) = (sample(), run1650()) else {
            eprintln!("skipping: no sample or no run1650 (RON_GOLDEN_DIR)");
            return;
        };
        let rec = crate::recgame::read(&rcx).expect("the sample parses");
        let lobby = header_lobby(&rec);
        let mut cap = crate::capture::indexed::IndexedCapture::open(&path).unwrap();
        let text = cap.read_replay_setup().unwrap();
        let log = Log::parse(&text);
        let init = log.replay_initial().expect("run1650's start");
        let parted = compare_lobby(&lobby, (&init.game_info, &init.players));
        for p in &parted {
            eprintln!("{p}");
        }
        assert_eq!(
            parted, SAMPLE_LOBBY_PARTED,
            "the header against run1650's GAMEINFO"
        );
    }

    /// The header's fields that are not the playback's: the version line
    /// alone (item 4336). The playback prints the running build's string,
    /// the header the recording build's; `(int) version` is 108265764 on
    /// both, and the options, seed, checksum triple, flags and the five
    /// `PLAYER` rows' every keyed field agree.
    const SAMPLE_LOBBY_PARTED: &[&str] =
        &["(Version:: header 00.2017.08.2100) playback 00.2024.06.2000)"];

    /// **The start built from the recording's header** (item 4336, row 1):
    /// [`from_lobby`] on [`header_lobby`] against [`build_sim`] on
    /// run1650's start dump (`WORLD=6`, the setup trace at
    /// `check_all_level` 14), compared record by record. Pins the first
    /// parted row and the agreeing count. `RON_START_DIFF=1` prints the
    /// steps, the notes and the partings.
    #[test]
    fn the_sample_s_start_built_from_its_header() {
        let (Some(inst), Some(rcx), Some(path)) = (install(), sample(), run1650()) else {
            eprintln!("skipping: no install, no sample or no run1650 (RON_GOLDEN_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let rec = crate::recgame::read(&rcx).expect("the sample parses");
        let (info, players) = header_lobby(&rec);
        let info: Vec<(&str, &str)> = info.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
        let players: Vec<Vec<(&str, &str)>> = players
            .iter()
            .map(|r| r.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect())
            .collect();
        let mut cap = crate::capture::indexed::IndexedCapture::open(&path).unwrap();
        let text = cap.read_replay_setup().unwrap();
        let log = Log::parse(&text);
        let init = log.replay_initial().expect("run1650's start");
        let n = player_count(&init);
        let theirs = build_sim(&loaded, &init, Tuning::RON);
        let ours = from_lobby(&loaded, &info, &players, Tuning::RON);
        let a = start_block(&ours.sim, n);
        let b = start_block(&theirs.sim, n);
        let parted = compare_start(&a, &b);
        let agree = b.len() - parted.iter().filter(|p| p.theirs.is_some()).count();
        if std::env::var_os("RON_START_DIFF").is_some() {
            for (s, built) in &ours.steps {
                eprintln!("step {s}: {}", if *built { "built" } else { "not built" });
            }
            for note in &ours.notes {
                eprintln!("note {note}");
            }
            let mut by_field: std::collections::BTreeMap<(String, &str), usize> =
                Default::default();
            for p in &parted {
                let kind = p.record.split(' ').next().unwrap_or("").to_string();
                *by_field.entry((kind, p.field)).or_default() += 1;
            }
            for ((k, f), c) in &by_field {
                eprintln!("parted {k} {f}: {c}");
            }
            let all = std::env::var("RON_START_DIFF").is_ok_and(|v| v == "all");
            for p in parted.iter().take(if all { usize::MAX } else { 40 }) {
                eprintln!("{p}");
            }
            eprintln!("agree {agree} of {}", b.len());
        }
        let first = parted.first().map(ToString::to_string);
        assert_eq!(
            (first.as_deref(), agree, b.len()),
            (
                SAMPLE_START_FIRST_PARTING,
                SAMPLE_START_AGREE.0,
                SAMPLE_START_AGREE.1
            ),
            "the sample's start built from its header (RON_START_DIFF=1 prints the value diff)"
        );
    }

    /// The first parted row of the sample's start, `record field: ours
    /// theirs`: none (item 4336). Minted at `BUILD 2/2000 pos: ours 10848
    /// 10080 theirs 5472 30816` — seats 2, 3 and 4 a cell round, the
    /// computers' team built in the permutation's order where
    /// `build_game` builds a seat's teammates at once after it
    /// ([`super::super::start::empire_order`]).
    const SAMPLE_START_FIRST_PARTING: Option<&str> = None;
    /// The sample's agreeing start fields, of the installed block's:
    /// 18,315 of 18,315 (item 4336; minted at 17,163, the 18 buildings'
    /// and 18 units' places and 1,116 cells' owners parted).
    const SAMPLE_START_AGREE: (usize, usize) = (18315, 18315);

    /// run1651's full playback, in place: the dump (`end:MISC`) and the
    /// trace of every frame 0..21044.
    fn run1651() -> Option<(String, std::path::PathBuf)> {
        let home = std::env::var("HOME").ok()?;
        let dir = std::env::var("RON_GOLDEN_DIR").unwrap_or(format!("{home}/ron-golden"));
        let d = std::path::PathBuf::from(format!("{dir}/run1651/map-7"));
        let dump = d.join("gamelog_p.txt");
        let trace = d.join("rontrace.log");
        (dump.is_file() && trace.is_file()).then(|| (dump.display().to_string(), trace))
    }

    /// run1654's window, in place: frames 84..101 of the playback at
    /// `end:UNITS=3,GUYS=4`, every unit and figure gaia's included — the
    /// widening of the word item 4347 moved off 98.
    fn run1654() -> Option<String> {
        let home = std::env::var("HOME").ok()?;
        let dir = std::env::var("RON_GOLDEN_DIR").unwrap_or(format!("{home}/ron-golden"));
        let p = std::path::PathBuf::from(format!("{dir}/run1654/map-7/gamelog_p.txt"));
        p.is_file().then(|| p.display().to_string())
    }

    /// run1674's window, in place: frames 3013..3021 of the playback at
    /// `end:UNITS=3,GUYS=4` (item 4380), the widening of the word on 3019.
    fn run1674() -> Option<String> {
        let home = std::env::var("HOME").ok()?;
        let dir = std::env::var("RON_GOLDEN_DIR").unwrap_or(format!("{home}/ron-golden"));
        let p = std::path::PathBuf::from(format!("{dir}/run1674/map-7/gamelog_p.txt"));
        p.is_file().then(|| p.display().to_string())
    }

    /// The start built from the sample's header (row 1), and the leaders
    /// it seats (`who` + 1 of the highest seat).
    fn sample_start(inst: &crate::Install, rec: &RecGame) -> (Built, usize) {
        let loaded = crate::load::load(inst).unwrap();
        let (info, players) = header_lobby(rec);
        let info: Vec<(&str, &str)> = info.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
        let players: Vec<Vec<(&str, &str)>> = players
            .iter()
            .map(|r| r.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect())
            .collect();
        let n = players
            .iter()
            .filter_map(|p| p.iter().find(|(k, _)| *k == "who"))
            .filter_map(|(_, v)| v.parse::<usize>().ok())
            .max()
            .map_or(1, |w| w + 1);
        let ours = from_lobby(&loaded, &info, &players, Tuning::RON);
        let (built, notes) = built_from_lobby(&loaded, ours);
        for note in &notes {
            eprintln!("note {note}");
        }
        (built, n)
    }

    /// **The walk's word** (item 4336, row 2): the start built from the
    /// header, walked under the turn pump on the recording's own packages
    /// (each acting as `info.player[play].who`), every frame's draws
    /// against run1651's trace of the original's playback. Pins `(word,
    /// frames walked)`: the word is the first frame whose draw sequence
    /// parts, or the walk's last when none does. `RON_SAMPLE_WALK=<frame>`
    /// stops the walk there.
    #[test]
    fn the_sample_walks_to_its_word() {
        let (Some(inst), Some(rcx), Some((dump, trace))) = (install(), sample(), run1651()) else {
            eprintln!("skipping: no install, no sample or no run1651 (RON_GOLDEN_DIR)");
            return;
        };
        let rec = crate::recgame::read(&rcx).expect("the sample parses");
        let (built, n) = sample_start(&inst, &rec);
        let tr = crate::trace::Trace::read(&trace)
            .expect("a finalized trace")
            .expect("a RONT header");
        let limit = std::env::var("RON_SAMPLE_WALK")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(i64::MAX);
        let mut stream = crate::input::Stream::pump(&rec);
        let mut captures = vec![("run1651", dump)];
        captures.extend(run1654().map(|p| ("run1654", p)));
        captures.extend(run1674().map(|p| ("run1674", p)));
        let (w, _) =
            super::super::start_walk::walk_with(built, n, &captures, &tr, limit, Some(&mut stream));
        eprintln!(
            "SAMPLE WALK: to {}; draws part (count) {:?}, (sequence) {:?}; records {} of which {} part, first {:?}",
            w.last, w.first_count, w.first_sequence, w.records, w.records_parted, w.first_record
        );
        if let Some((o, t)) = &w.sites
            && let Some((k, d)) = crate::diff::testkit::first_parting(o, t)
        {
            eprintln!(
                "the first parting frame {:?}, site {k}: {d}",
                w.first_sequence
            );
        }
        if limit != i64::MAX {
            return;
        }
        assert_eq!(
            (w.first_sequence.unwrap_or(w.last), w.last),
            SAMPLE_WALKED,
            "the sample's word against run1651's draws"
        );
        if run1654().is_some() && run1674().is_some() {
            assert_eq!(
                (
                    w.records,
                    w.records_parted,
                    w.first_record.as_ref().map(|(f, c, _)| (*f, c.as_str()))
                ),
                SAMPLE_WINDOW_RECORDS,
                "run1654's and run1674's windows (RON_START_WALK_ROWS=1 prints the first parted record)"
            );
        }
    }

    /// run1654's window against the walk (item 4347): `(records compared,
    /// records parted, the first parted record's frame and capture)`. 18
    /// of 18 agree (item 4351: 9 parted → 0). On 92 the person's scout
    /// `3/0` (`who` 3) held an `EXPLORE_TO` (`OrderIndex` 3) in theirs and
    /// a `MOVE_TO` (1) here: the package on recording frame 91 is
    /// `move_to` with `orders` 1 and `queued` 1, and a scout's
    /// `QUEUE_LAST` `MOVE_TO` is an explore
    /// ([`sim::Unit::explore_queued`]). run1651 prints no frame block, so
    /// its one record is the closing one.
    ///
    /// **run1674's window (item 4380), 3013..3020: 8 records, all 8
    /// part** (18, 0, None → 26, 8, first 3013). The one unit parted on
    /// every block is the person's SCOUT `3/0` (`TypeIndex` 69, land,
    /// `uid` 6) on its tenth `EXPLORE_TO` to (42216, 31320): on 3013 ours
    /// stands at (39311, 32904) with the move's `dest_x` 39912, theirs at
    /// (39801, 32538) with 40680. The two paths agree on their first 61
    /// nodes; ours holds three more, (41448, 32088), (40680, 32856),
    /// (39912, 32856), what is left of the grid detour it took on 2892
    /// round computer 1's capital after the refused straight line on 2891
    /// (`Unit::do_move+0xe84 < Unit::do_explore_to`, spent on both
    /// sides). On 3020 computer 1's MERCHANT `1/17` (`TypeIndex` 61) parts
    /// too: theirs collides with the scout at (39826, 32220), ours does not.
    ///
    /// Item 4442 (`find_wpath`'s scouting gate: the scout's `flags 5`
    /// explore plans at a person's prices, the 2892 detour gone): 8 parted
    /// → 0, all 26 agree.
    const SAMPLE_WINDOW_RECORDS: (usize, usize, Option<(i64, &str)>) = (26, 0, None);

    /// The sample's word and the frames walked, against run1651's draws:
    /// 3019 of 21,044 (item 4359: 2888 → 3019, +131, in two builds. On
    /// 2887 the person's units strike computer 4's capital `4/2000` and
    /// its city's bell rings, [`sim::Sim::city_struck_alarm`]: 2888 → 2908,
    /// a Citizen's garrison walk's `Unit::do_move+0xe84`. On 2901 `ai
    /// economic`'s head asks `was_city_raided`, which read never, and the
    /// capital's light blow had stamped `raid_stamp` 2887: 2908 → 3019,
    /// computer 4's `Make` step's nine `make_stuff` draws on 2908. On 3019
    /// theirs starts two units' steps (`Unit::move_step+0x823`) three
    /// frames after `Armies::emergency`, whose two armies are empty here.)
    /// Before that, 2888 (item 4351: 444 → 2888, +2444, the scout's
    /// shift-queued moves made explores, [`sim::Unit::explore_queued`]; the
    /// sequences part at draw 32 there). Before that, 444 (item 4347: 98 → 444, +346, the gaia figures' variant
    /// off the lobby's seed, [`sim::Sim::info_seed`]; minted at 98 by item
    /// 4336). On 98 herd 0's peacocks `8/0`..`8/3` wore pieces 60079,
    /// 60080, 60078, 60079 here against run1654's 60078, 60079, 60080,
    /// 60078, so `8/0`'s 99-frame idle ran out here and `8/1`'s in theirs:
    /// ours took the far arm (413 from the herd centre), theirs the near
    /// one's three draws.
    ///
    /// Item 4442 (`find_wpath`'s scouting gate; the person's Scout `3/0`
    /// re-plans its `flags 5` explore on 2892 at a person's prices): 3019
    /// → 3887, +868. Item 4463 (the Inca capital taken on 3887 hands its
    /// building's age, farm slots, flags and `visible` across, keeps the
    /// loser's city slots, and its caravan's route ends that frame): 3887
    /// → 3903, +16.
    const SAMPLE_WALKED: (i64, i64) = (3903, 21044);

    /// run1662: the sample played whole under the `RON_CHECKSUMS` hook
    /// (`docs/RUNS.md` run1662), the fifteen sums on every frame.
    const RUN1662_TRACE: &str = "rontrace-run1662.log";

    /// **The sample by state, parity's second game** (item 4359,
    /// `docs/RECGAME.md` §7; `docs/DECISIONS.md` entry 140 §2): the walk
    /// of [`the_sample_walks_to_its_word`] against run1662's hook, our
    /// twelve emitted lines' Adler on every frame beside the original's
    /// ([`super::super::checksum_frames`]' pairing: the hook's frame `f`
    /// against our state at `sim.frame == f + 1`).
    #[test]
    fn the_sample_s_lines_against_the_hook() {
        let (Some(inst), Some(rcx)) = (install(), sample()) else {
            eprintln!("skipping: no install or no sample");
            return;
        };
        let Some(trace) = super::super::checksum_frames::hook(RUN1662_TRACE) else {
            eprintln!("skipping: {RUN1662_TRACE} not filed (set RON_GAMELOG_DIR)");
            return;
        };
        let per = crate::trace::checksums::per_frame(&trace);
        let frames = (
            per.len(),
            per.keys().next().copied(),
            per.keys().last().copied(),
        );
        let theirs: [Option<i64>; 3] = std::array::from_fn(|j| {
            trace
                .iter()
                .find(|s| s.lines[2 + j] != 1)
                .map(|s| s.frame + 1)
        });
        eprintln!(
            "SAMPLE HOOK: {} sets on {} frames ({:?}..{:?}); theirs' first walls, ammo, deaths not empty {theirs:?}",
            trace.len(),
            frames.0,
            frames.1,
            frames.2
        );
        let rec = crate::recgame::read(&rcx).expect("the sample parses");
        let (built, _) = sample_start(&inst, &rec);
        let stream = crate::input::Stream::pump(&rec);
        let Some((walked, (empty, first_held))) =
            super::super::checksum_frames::first_partings_of(built, stream, &trace, SAMPLE_HOOK.1)
        else {
            eprintln!("skipping: no pieces (RON_INSTALL)");
            return;
        };
        for (line, f, ours, theirs, p, n) in &walked {
            eprintln!(
                "SAMPLE LINES {line}: first parts on {f} (ours {ours}, theirs {theirs}); agrees on {n}; \
                 inside 300: {} frames, {}",
                p.len(),
                super::super::checksum_frames::spans(p)
            );
        }
        eprintln!("SAMPLE HELD: ours none on {empty} frames, first held {first_held:?}");
        let seen: Vec<(&str, i64)> = walked.iter().map(|(l, f, ..)| (*l, *f)).collect();
        assert_eq!(
            (trace.len(), frames.0, theirs),
            (
                SAMPLE_HOOK.0,
                SAMPLE_HOOK.1 as usize + 1,
                SAMPLE_THEIRS_HELD
            ),
            "run1662's take"
        );
        assert_eq!(seen, SAMPLE_LINES, "the sample's lines against the hook");
        assert_eq!(
            (empty, first_held.map(|(f, _)| f)),
            SAMPLE_OURS_HELD,
            "the frames our walk holds no wall, shot or death"
        );
    }

    /// run1662's take: `(sets, the frames walked)`: 22,077 sets, one on
    /// every trace frame −1 to 21,043 and a second on the frames before an
    /// order's package. Minted at 0.
    const SAMPLE_HOOK: (usize, i64) = (22077, 21044);

    /// The first frame (ours, `f + 1` of the hook's) on which theirs'
    /// walls, ammo and deaths are not the Adler of no bytes (item 4359,
    /// minted at none): no wall in the whole game; the first shot in
    /// flight on 2929, computer 4's capital firing on the person's units
    /// once its Citizens are garrisoned by the bell; the first death
    /// object on 3365. The first walk on disk where two of the three
    /// record shapes are exercised (`docs/CHECKSUM.md` §8).
    const SAMPLE_THEIRS_HELD: [Option<i64>; 3] = [None, Some(2929), Some(3365)];

    /// Per emitted line, the first frame ours parts from the hook's (the
    /// walk's length where it never does), item 4359's tree (the bell and
    /// the raid stamp built). Minted empty. Builds, leaders and world part
    /// on the start and goods on 1: the sample's start is whole by the
    /// dump's records (18,315 of 18,315) and not by the sums. Units part
    /// on 92, the frame run1654's window agrees on whole, so the parted
    /// field is one no `UNITS` record prints. Deaths part where theirs
    /// first hold one. Ammo agrees from theirs' first shot on 2929 to 3021
    /// on the merged tree (2929 → 3022: the base's ammo records, merged
    /// into this lane), where our emitter refused a held shot before.
    /// Item 4394 (the frame-0 `seen2` share and the census's `ever_seen`
    /// mark): goods 1 → 4094 and world 0 → 1716; builds and leaders still
    /// part on the start. Item 4397 (`Setup::reinit_game_units`, every
    /// building's `z` off the whole start's ground): builds 0 → 76; and
    /// (the walk's oil left to `compute_all_territory`) leaders 0 → 241.
    /// Item 4405 (`walk_data@004708a0` emits a gather list last point
    /// first): builds 76 → 525; and (`unit_masks & 0x4000000`, the queued
    /// explore, on the word) units 92 → 532. Item 4412 (`strong`'s four
    /// building lines and its third parties at war are a human target's
    /// arm, `6bd02a`): leaders 241 → 2112; and (a detector building's
    /// `SubObjectData::flags & 0x40`, `Object::init`'s `DETECT`) builds 525
    /// → 3029. Item 4414 (a `THINK` walks `UnitOrder::walk_data@0047f2b0`'s
    /// flags byte): units 532 → 2112. Item 4419 (a dead building leaves its
    /// group on the next push): groups 927 → 961. Item 4422
    /// (`action_swarm_around`'s `form` −1, slot 195 on 961): groups 961 →
    /// 2888. Item 4424 (a trained captain's exit lights no disc; the
    /// gather re-seat lights its own): world 1716 → 2921. Item 4429
    /// (`process_attrition`'s check 6, an ally's ground both ways): units
    /// 2112 → 2700 and leaders 2112 → 2989. Item 4432 (`action_move_near`'s
    /// same-region arm: a refused slot re-placed round its captain's, the
    /// person's Ashigaru Spearmen `3/27` on 2699): units 2700 → 2888. Item
    /// 4440 (`GarrisonOrder::walk_data@00484690` ported, and the bell's
    /// one-building group pushed on its player's pool): units 2888 → 2892,
    /// groups 2888 → 3053. Item 4442 (`find_wpath`'s scouting gate: an
    /// explore order carrying `ACTION` scouts only under `unit_masks &
    /// 0x40100`; the person's Scout `3/0` re-plans on 2892 at a person's
    /// prices): units 2892 → 3878, builds 3029 → 3878, ammo 3022 → 3888,
    /// deaths 3365 → 3932, groups 3053 → 3935, guys 2920 → 3878, leaders
    /// 2989 → 3575, cities 3176 → 3888, items 3976 → 7544, goods 4094 →
    /// 4559, world 2921 → 3878. Item 4445 (an outside mine pays the Inca's
    /// wealth per miner): leaders 3575 → 3888; items 7544 → 5760, past
    /// 3878, where units, builds, guys and world already part. Item 4449
    /// (`set_new_location(…, 1)` copies a figure's point into `last`, the
    /// Inca Citizen `4/4` out of its building): units and guys 3878 → 3888.
    /// Item 4454 (a building at health 0 still counts its arrows down,
    /// `Build::process@0061edf0`): builds 3878 → 3888; items 5760 → 4805,
    /// a named fall past world's unchanged 3878 and the 3888 partings.
    /// Item 4456 (a unit out of a building paints its disc where it stood
    /// inside, then moves it): world 3878 -> 3888, every other line
    /// unchanged. Item 4463 (the capture of 3887 carried across: the
    /// building's `max_age`, farm slot, flags and `visible`, the city's
    /// `traded_with`, the loser's city slots, its caravan's route ended):
    /// units 3888 -> 3906, builds -> 3904, ammo -> 3920, groups 3935 ->
    /// 4004, guys -> 3906, cities -> 3891, world -> 3889, items 4805 ->
    /// 5474; goods 4559 -> 4458, a named fall past leaders' unchanged 3888.
    /// Item 4466 (the capture's members close before the old city; the
    /// line's `0x400000` and make-list city slot): leaders 3888 -> 3889.
    /// Item 4469 (an unassimilated city's cells win as −2): leaders 3889 ->
    /// 3925, cities 3891 -> 4201, world 3889 -> 3904.
    const SAMPLE_LINES: &[(&str, i64)] = &[
        ("units", 3906),
        ("builds", 3904),
        ("walls", 21044),
        ("ammo", 3920),
        ("deaths", 3932),
        ("groups", 4004),
        ("guys", 3906),
        ("leaders", 3925),
        ("cities", 4201),
        ("items", 5474),
        ("goods", 4458),
        ("world", 3904),
    ];

    /// `(frames our walk holds no wall, shot or death, the first it holds
    /// one)`. Minted at 0: none on 3,594 of 21,044, the first shot on 2929,
    /// theirs' frame. Item 4429 (an ally's ground bleeds nothing): 3594 →
    /// 4361 empty frames, past every parting; the first shot stays on 2929.
    /// Item 4432 (a refused group slot re-placed): 4361 → 5010, the first
    /// shot still on 2929. Item 4440 (the bell's group pushed, a garrison
    /// walked): 5010 -> 4141, every frame it loses past units' 2892 and
    /// groups' 3053; the first shot still on 2929. Item 4442 (the scouting
    /// gate): 4141 -> 3855, the first shot still on 2929. Item 4443 (on
    /// 4430 and 4444): 3855 -> 3974, measured on the merged tip; 4443's fix
    /// is the units line's image alone, so the frames moved with 4430's
    /// turrets or 4444's caps meeting 4442's gate, all past leaders' 3575;
    /// the first shot still on 2929. Item 4445 (an outside mine pays the
    /// Inca's wealth per miner): 3974 -> 5344, measured on the merged tree;
    /// the first shot still on 2929. Item 4454 (a building at health 0
    /// still counts its arrows down): 5344 -> 6697, measured on the merged
    /// tree (22 commits of the base beside it); the first shot still on
    /// 2929. Item 4463 (the capture of 3887 carried across): 6697 -> 5042,
    /// a named fall, the lost frames not read one by one (a capture that
    /// lands differently moves the war after it); the first shot still on
    /// 2929. Item 4466 (the capture's members close before the old city,
    /// the library's queue refunded): 5042 -> 6309, a named rise past every
    /// parting, the frames not read one by one; the first shot still on
    /// 2929. Item 4469 (an unassimilated city's cells win as −2, the
    /// captured Inca capital's 29 from 3889): 6309 -> 4473, a named fall,
    /// the frames not read one by one; the first shot still on 2929.
    const SAMPLE_OURS_HELD: (usize, Option<i64>) = (4473, Some(2929));

    /// run1678's record (item 4384, `docs/RUNS.md` run1678): the sample's
    /// playback over 0..100 under the checksum and ranges hooks, all twelve
    /// lines kept (`RNG_MASK` 0xfff). `RON_SAMPLE_RANGES`, else where the
    /// take left it.
    fn run1678() -> Option<crate::trace::ranges::Record> {
        ranges_record("RON_SAMPLE_RANGES", "run1678")
    }

    /// run1697's record (item 4412, `docs/RUNS.md` run1697): the same
    /// playback to 560 under the same hooks, for the parting on 241 and the
    /// two on 525 and 532. `RON_SAMPLE_RANGES_1697`, else where the take
    /// left it.
    fn run1697() -> Option<crate::trace::ranges::Record> {
        ranges_record("RON_SAMPLE_RANGES_1697", "run1697")
    }

    /// run1710's record (item 4414, `docs/RUNS.md` run1710): the same
    /// playback to 3,100 under the same hooks, so every first parting under
    /// the draw walk's 3019 is a diff on disk. `RON_SAMPLE_RANGES_1710`,
    /// else where the take left it.
    fn run1710() -> Option<crate::trace::ranges::Record> {
        ranges_record("RON_SAMPLE_RANGES_1710", "run1710")
    }

    /// run1724's record (item 4445, `docs/RUNS.md` run1724): the same
    /// playback to 4,130 under the same hooks, for leaders' 3575 and the
    /// four lines on 3878. `RON_SAMPLE_RANGES_1724`, else where the take
    /// left it.
    fn run1724() -> Option<crate::trace::ranges::Record> {
        ranges_record("RON_SAMPLE_RANGES_1724", "run1724")
    }

    fn ranges_record(var: &str, run: &str) -> Option<crate::trace::ranges::Record> {
        let path = match std::env::var(var) {
            Ok(p) => std::path::PathBuf::from(p),
            Err(_) => {
                let home = std::env::var("HOME").unwrap_or_default();
                let dir = std::path::PathBuf::from(format!("{home}/ron-golden/{run}/map-7"));
                crate::testenv::experiment(&dir, "rontrace.ranges")?
            }
        };
        match crate::trace::ranges::Record::open(&path) {
            Ok(r) => Some(r),
            Err(e) => {
                eprintln!(
                    "skipping: no sample ranges record at {} ({e})",
                    path.display()
                );
                None
            }
        }
    }

    /// **The sample's lines by range, from the start** (item 4384,
    /// `docs/RECGAME.md` §7; DECISIONS 142 §4): the walk of
    /// [`the_sample_s_lines_against_the_hook`] over run1678's frames, each
    /// of the twelve lines held range by range against the record's set
    /// (the hook's frame `f − 1` against our `sim.frame == f`). Pins, per
    /// line that parts in the window, `(line, its first parted frame, the
    /// record's ranges, ours, ranges parted, the first parting as `range
    /// record field: name ours|theirs`)`. `RON_SAMPLE_RANGES_ALL=1` prints
    /// every frame's partings.
    #[test]
    fn the_sample_s_ranges_name_each_first_parting() {
        let Some(rec) = run1678() else { return };
        let Some((frames, got)) = walk_ranges(rec, SAMPLE_RANGES_WINDOW) else {
            return;
        };
        let pinned: Vec<SampleRange> = SAMPLE_RANGES
            .iter()
            .map(|&(l, f, t, o, p, x)| (l, f, t, o, p, x.to_string()))
            .collect();
        assert_eq!(
            frames,
            SAMPLE_RANGES_WINDOW as usize + 1,
            "run1678's sets, one a frame"
        );
        assert_eq!(got, pinned, "SAMPLE_RANGES");
    }

    /// **The sample's lines by range, to 560** (item 4412): the same walk
    /// over run1697's record, pinned per line by [`SAMPLE_RANGES_1697`].
    /// `RON_SAMPLE_RANGES_DETAIL` and `RON_SAMPLE_RANGES_ALL` print as
    /// above.
    #[test]
    fn the_sample_s_ranges_to_560_name_each_first_parting() {
        let Some(rec) = run1697() else { return };
        let Some((frames, got)) = walk_ranges(rec, SAMPLE_RANGES_1697_WINDOW) else {
            return;
        };
        let pinned: Vec<SampleRange> = SAMPLE_RANGES_1697
            .iter()
            .map(|&(l, f, t, o, p, x)| (l, f, t, o, p, x.to_string()))
            .collect();
        assert_eq!(
            frames,
            SAMPLE_RANGES_1697_WINDOW as usize + 1,
            "run1697's sets, one a frame"
        );
        assert_eq!(got, pinned, "SAMPLE_RANGES_1697");
    }

    /// **The sample's lines by range, to 3,100** (item 4414): the same walk
    /// over run1710's record, pinned per line by [`SAMPLE_RANGES_1710`].
    /// `RON_SAMPLE_RANGES_DETAIL` and `RON_SAMPLE_RANGES_ALL` print as
    /// above.
    #[test]
    fn the_sample_s_ranges_to_3100_name_each_first_parting() {
        let Some(rec) = run1710() else { return };
        let Some((frames, got)) = walk_ranges(rec, SAMPLE_RANGES_1710_WINDOW) else {
            return;
        };
        let pinned: Vec<SampleRange> = SAMPLE_RANGES_1710
            .iter()
            .map(|&(l, f, t, o, p, x)| (l, f, t, o, p, x.to_string()))
            .collect();
        assert_eq!(
            frames,
            SAMPLE_RANGES_1710_WINDOW as usize + 1,
            "run1710's sets, one a frame"
        );
        assert_eq!(got, pinned, "SAMPLE_RANGES_1710");
    }

    /// **The sample's lines by range, to 4,130** (item 4445): the same walk
    /// over run1724's record, pinned per line by [`SAMPLE_RANGES_1724`].
    /// `RON_SAMPLE_RANGES_DETAIL` and `RON_SAMPLE_RANGES_ALL` print as
    /// above.
    #[test]
    fn the_sample_s_ranges_to_4130_name_each_first_parting() {
        let Some(rec) = run1724() else { return };
        let Some((frames, got)) = walk_ranges(rec, SAMPLE_RANGES_1724_WINDOW) else {
            return;
        };
        let pinned: Vec<SampleRange> = SAMPLE_RANGES_1724
            .iter()
            .map(|&(l, f, t, o, p, x)| (l, f, t, o, p, x.to_string()))
            .collect();
        assert_eq!(
            frames,
            SAMPLE_RANGES_1724_WINDOW as usize + 1,
            "run1724's sets, one a frame"
        );
        assert_eq!(got, pinned, "SAMPLE_RANGES_1724");
    }

    /// The walk the ranges tests share: our sample from state 0 to
    /// `window`, each frame's twelve lines against the record's set.
    /// `(sets compared, each parting line's first parting)`, or `None` when
    /// the install, the sample or the pieces are absent.
    fn walk_ranges(
        mut rec: crate::trace::ranges::Record,
        window: i64,
    ) -> Option<(usize, Vec<SampleRange>)> {
        use super::super::checksum_lines::packets::{Pieces, STATE_LINES, compare, our_lines};
        let (Some(inst), Some(rcx)) = (install(), sample()) else {
            eprintln!("skipping: no install or no sample");
            return None;
        };
        let Some(pieces) = Pieces::read() else {
            eprintln!("skipping: no pieces (RON_INSTALL)");
            return None;
        };
        let all = std::env::var("RON_SAMPLE_RANGES_ALL").is_ok();
        let detail: Option<i64> = std::env::var("RON_SAMPLE_RANGES_DETAIL")
            .ok()
            .and_then(|v| v.parse().ok());
        let rec_game = crate::recgame::read(&rcx).expect("the sample parses");
        let (mut built, _) = sample_start(&inst, &rec_game);
        let mut stream = crate::input::Stream::pump(&rec_game);
        let mut seen: Vec<Option<FirstSeen>> = vec![None; 12];
        let mut frames = 0usize;
        while built.sim.frame <= window {
            let f = built.sim.frame;
            if let Some(theirs) = super::super::checksum_frames::record::stream_at(&mut rec, f) {
                frames += 1;
                let ours = our_lines(&built.sim, &pieces);
                if detail == Some(f) {
                    print_partings(&ours, &theirs);
                }
                let outcome = compare(&ours, &theirs);
                for (k, (name, o)) in STATE_LINES.iter().zip(&outcome).enumerate() {
                    let Some(d) = &o.first else { continue };
                    let text = super::super::checksum_frames::first_text(name, d);
                    if all {
                        eprintln!(
                            "SAMPLE RANGES frame {f} {name}: theirs {} ours {} parted {}; first {text}",
                            o.theirs, o.ours, o.parted
                        );
                    }
                    if seen[k].is_none() {
                        seen[k] = Some((f, o.theirs, o.ours, o.parted, text));
                    }
                }
            }
            stream.apply(f as i32, &mut built);
            crate::diff::harness::debug_watch(&built, f);
            crate::diff::harness::debug_builds(&built, f);
            built.tick();
        }
        let got: Vec<SampleRange> = STATE_LINES
            .iter()
            .zip(seen)
            .filter_map(|(name, s)| s.map(|(f, t, o, p, x)| (*name, f, t, o, p, x)))
            .collect();
        for row in &got {
            eprintln!("SAMPLE RANGES first {row:?}");
        }
        Some((frames, got))
    }

    /// Every parted range of every line on one frame, decoded where the
    /// line has a decoder (`RON_SAMPLE_RANGES_DETAIL=<frame>`).
    pub(in crate::diff) fn print_partings(
        ours: &[sim::checksum::Line],
        theirs: &std::collections::BTreeMap<String, super::super::checksum_lines::StreamRows>,
    ) {
        use super::super::checksum_lines::packets::STATE_LINES;
        let empty = Default::default();
        for (name, o) in STATE_LINES.iter().zip(ours) {
            let t: &super::super::checksum_lines::StreamRows = theirs.get(*name).unwrap_or(&empty);
            for i in 0..o.ranges.len().max(t.len()) {
                let (a, b) = (o.ranges.get(i), t.get(i));
                if a.map(|r| &r.bytes) == b.map(|r| &r.2) {
                    continue;
                }
                let (rec, field) = a.map_or((-1, "-"), |r| (r.record, r.field));
                // A leader's decrypted block is one range a word: name the
                // word by its place in the record's run of them (item 4445).
                let field = match (*name, field) {
                    ("leaders", "encrypt") => {
                        let k = i - o
                            .ranges
                            .iter()
                            .position(|r| r.record == rec && r.field == "encrypt")
                            .unwrap_or(i);
                        let six = sim::checksum::leaders::ENCRYPT_SIX;
                        let leaf = if k < 54 {
                            format!("{}[{}]", six[k % 9], k / 9)
                        } else {
                            format!("word {k}")
                        };
                        format!("encrypt {leaf}")
                    }
                    _ => field.to_string(),
                };
                let field = field.as_str();
                let fields = match (a, b) {
                    // A shifted range decodes theirs' bytes by our field's
                    // name; at another length that is raw bytes alone.
                    (Some(a), Some(b)) if a.bytes.len() != b.2.len() => {
                        format!(
                            "len {}|{}: {:02x?}|{:02x?}",
                            a.bytes.len(),
                            b.2.len(),
                            a.bytes,
                            b.2
                        )
                    }
                    (Some(a), Some(b)) => {
                        let x = super::super::checksum_lines::range_fields(name, field, &a.bytes);
                        let y = super::super::checksum_lines::range_fields(name, field, &b.2);
                        match (x, y) {
                            (Some(x), Some(y)) => x
                                .iter()
                                .zip(&y)
                                .filter(|(p, q)| p.1 != q.1)
                                .map(|(p, q)| format!("{} {}|{}", p.0, p.1, q.1))
                                .collect::<Vec<_>>()
                                .join(", "),
                            _ => {
                                let d: Vec<String> = a
                                    .bytes
                                    .iter()
                                    .zip(&b.2)
                                    .enumerate()
                                    .filter(|(_, (p, q))| p != q)
                                    // Probe (item 4472): every byte.
                                    .take(if std::env::var("RON_SAMPLE_RANGES_BYTES").is_ok() {
                                        usize::MAX
                                    } else {
                                        16
                                    })
                                    .map(|(k, (p, q))| format!("+{k:#x} {p:02x}|{q:02x}"))
                                    .collect();
                                format!("len {}|{}: {}", a.bytes.len(), b.2.len(), d.join(", "))
                            }
                        }
                    }
                    _ => "absent on one side".to_string(),
                };
                eprintln!("PARTED {name} {i} {rec} {field}: {fields}");
            }
        }
    }

    type SampleRange = (&'static str, i64, usize, usize, usize, String);

    /// A line's first parting: `(frame, the record's ranges, ours, parted, the text)`.
    type FirstSeen = (i64, usize, usize, usize, String);

    /// run1678's last frame, ours: the record holds trace frames −1 to 99.
    const SAMPLE_RANGES_WINDOW: i64 = 100;

    /// Per line that parts on run1678's window, its first parting by range
    /// (item 4384; minted empty, then at the first act's five rows). A
    /// leaders range starts at `LeaderData +0x8`.
    ///
    /// Item 4388, act 1 (`Leader::init`'s mask in the start permutation's
    /// order, read by `Wall::activate`): builds 21 → 2 parted on state 0,
    /// the first `10 2000 ever_seen_completed` 23|3 → `146 2003 z` 142|145;
    /// leaders' `ally_mask` 0x17|3, 0x17|2, 0x17|7 on leaders 0 to 2 agree.
    /// Act 2 (`init_teams`' 0x80 arm): `chat_status[3]` 0|1 on leaders 0,
    /// 1, 2 and 4, and the person's `[0..2]`, `[4]` 0|1, agree. Act 3 (an
    /// unseated slot is `get_player`'s record 0, the person's team): the
    /// person's `diplos[5..7]` 0|2, `treaties[5..7]` 0|1, `aggression[5..7]`
    /// 1|0 agree. Leaders 6 → 1 parted, the first `1 0 chat_status[3]` →
    /// `328 2 oil_patches` (20, 5)|(5, 20). State 0: 28 → 4 parted ranges.
    ///
    /// Item 4394, act 1 (`Game::run:722`'s `GameDaemon::update_all_seen`
    /// before the first frame, and its frame-0 tail sharing each team's
    /// `seen2`): world `3626 seen2` at fog (24, 75) and (25, 76..80), leader
    /// 2's Woodcutter `2/2001`'s disc past its city's radius, ours 0x04,
    /// theirs 0x17 → agree; world whole over the window. With
    /// `plan_strategy`'s `new_rares` arm marking `ever_seen`, goods' `145
    /// 29 ever_seen` on frame 1, ours 0x04, theirs 0x17 → agree; goods
    /// whole over the window. State 0: 4 → 3 parted ranges.
    ///
    /// Item 4397, act 1 (`Game::init:629`'s `Setup::reinit_game_units`,
    /// every live building's `z` sampled again once the start stands):
    /// builds `146 2003 z` 142|145 and `785 2002 z` 316|330 agree (Farms
    /// placed before `0/2006`'s and `3/2005`'s terraforms moved their
    /// tiles). Builds whole on state 0, its first parting 0 → 76, `758
    /// 2000 gather[k]` 32640|31872. State 0: 3 → 1 parted range. Act 2
    /// (the start's walk leaves the oil to `compute_all_territory`, after
    /// the city's `update_seen(0)`): leader 2's `oil_patches` (20, 4, 5,
    /// 15)|(5, 4, 20, 15) agree, its capital's disc revealing goods 5 and 4
    /// first. Leaders whole over the window; state 0: 1 → 0 parted ranges.
    ///
    /// Item 4405, act 1 (`PtrLinkListAbstract<GatherPoint>::walk_data@
    /// 004708a0` walks the list from `head_node->prev` by `prev`, last point
    /// first): the person's `city_gather` Farm points on `3/2000` (recording
    /// frame 75) go out reversed, builds `758 2000 gather[k]` 32640|31872,
    /// `761` 33600|34560, `767` 34560|33600, `770` 31872|32640 agree on 76.
    /// Builds whole over the window. Act 2 (`unit_masks & 0x4000000`,
    /// `add_move_facing_order`'s queued explore, carried since item 4351 as
    /// `Unit::explore_queued`, on the word): units `439 0 unit_masks`
    /// 8|0x4000008 on 92, the person's scout `3/0`, agree. All twelve lines
    /// whole over the window.
    const SAMPLE_RANGES: &[(&str, i64, usize, usize, usize, &str)] = &[];

    /// run1697's last frame, ours: the record holds trace frames −1 to 559.
    const SAMPLE_RANGES_1697_WINDOW: i64 = 560;

    /// Per line that parts on run1697's window, its first parting by range
    /// (item 4412). Minted at three rows: units 532, builds 525 and leaders
    /// 241, `528 4 who..ally_mask` `+0x230` 0x20|0, leader 4's `strong[2]`
    /// (`+0x238`) against its teammate 2, ours 32, theirs 0. Act 1
    /// (`Leader::diplomacy`'s `6bd02a`: the four building lines and the
    /// third parties at war count only toward a human target): the leaders
    /// row agrees; leaders whole over the window. Act 2 (`Object::init@
    /// 00647750:39`'s `flags |= 0x40` for a `DETECT` type, a building's as
    /// a unit's): builds `974 2008 flags` 1|0x41, the person's Lookout
    /// `3/2008` (`TypeIndex` 107), agrees; builds whole over the window.
    /// Item 4414, act 1 (`ThinkOrder`'s vslot `+0x8` is the base's
    /// `UnitOrder::walk_data@0047f2b0`, the flags byte): the person's
    /// Citizen `3/6` (`TypeIndex` 50, land), a `MOVE_TO` then a `THINK`, its
    /// range `905` absent|`04` agrees, ours 1,556 → 1,557 ranges of 1,557.
    /// All twelve lines whole over the window.
    const SAMPLE_RANGES_1697: &[(&str, i64, usize, usize, usize, &str)] = &[];

    /// run1710's last frame, ours: the record holds trace frames −1 to 3099.
    const SAMPLE_RANGES_1710_WINDOW: i64 = 3100;

    /// Per line that parts on run1710's window, its first parting by range
    /// (item 4414). Minted at seven rows, each on the frame the sum parts
    /// (`SAMPLE_LINES`): units, builds, ammo, groups, guys, leaders, world.
    /// Units: item 4429 2112 -> 2700 (computer 2's Citizen `2/1` on its
    /// ally 0's ground bled the peace period on 2111 in ours alone); item
    /// 4432 2700 -> 2888 (the person's Ashigaru Spearmen `3/27` re-placed
    /// round its captain's); item 4440 2888 -> 2892 (computer 4's ten
    /// Citizens under the bell walk `GarrisonOrder::walk_data@00484690`).
    /// Groups: items 4419 and 4422 927 -> 2888; item 4440 2888 -> 3053 (the
    /// bell's `[4/2000]` pushed into player 4's slot 0 on 2887). Leaders:
    /// item 4429 2112 -> 2989. World: item 4424 1716 -> 2921.
    ///
    /// Item 4442 (`find_wpath@00688fc0`'s scouting gate, `order.flags & 4
    /// == 0 || unit_masks & 0x40100`): the person's Scout `3/0` (`TypeIndex`
    /// 69, land) re-plans its queued explore (`flags 5`) to (40680, 31320)
    /// on 2892 at a person's prices, seen ground the cheaper: `1410 0
    /// path.size..increment` `path.length` ours 68 -> 65, theirs 65, and
    /// `path[61..64]` ours (41448, 32088), (40680, 32856), (39912, 32856),
    /// (39144, 32856) -> theirs' (39912, 32856), (39144, 32856), (38376,
    /// 32088), (38376, 31320). Every row went with it: builds 3029 (`1815
    /// 2003` `up` 5|7), ammo 3022, groups 3053 (`341 257` `ox` 7392|20448),
    /// guys 2920 (`379 0` `angle`), leaders 2989 (`457 3` `oil_patches`
    /// 0x0c|0x0d) and world 2921 (`6412 2449` `block.body`) agree. All twelve
    /// lines whole over the window.
    const SAMPLE_RANGES_1710: &[(&str, i64, usize, usize, usize, &str)] = &[];

    /// run1724's last frame, ours: the record holds trace frames −1 to 4129.
    const SAMPLE_RANGES_1724_WINDOW: i64 = 4130;

    /// Per line that parts on run1724's window, its first parting by range
    /// (item 4445). Minted at nine rows with leaders on 3575: computer 2's
    /// wealth (`ENCRYPT_SIX` index 2), `356 2 encrypt leftover[2]` ours
    /// 2160, theirs 2400, `resources[2]` and `income[2]` ours 160, theirs
    /// 320, its first miner at an outside mine. Step 5 now pays the Inca's
    /// wealth per miner (`BuildTypeData::calc_gather@00639e40:849`–`856`,
    /// reached from `Leader::calc_gather@006ceee0:211`–`222`): leaders
    /// 3575 -> 3888, every other row unchanged. Item 4449: units and guys
    /// 3878 -> 3888, `set_new_location(…, 1)`'s `last` copy
    /// (`Guy::set_new_location@005d86f0:66`–`70`) on the Inca Citizen `4/4`
    /// out of its building: `2844 4 guy` `last_x/last_y/last_z` ours
    /// 10152, 10104, 332 -> theirs' 10488, 10968, 304. Item 4454: builds
    /// 3878 -> 3888, `Build::process@0061edf0` reaching `do_attack` at
    /// health 0: computer 4's `2672 2000` `recharging` ours 8 -> theirs' 7
    /// (it held at 8 from 3877 on; theirs went 7, 6, 5). Item 4456: world
    /// 3878 -> 3888, `Unit::come_out@00617c10:514`'s `add_to_world` painting
    /// the Inca Citizen `4/4`'s disc at its inside point (10152, 10104)
    /// before `:518`'s `move_unit` to its spot (10488, 10968):
    /// `4474 793 block.present` ours 0 -> theirs' 1, an empty block on
    /// world cell (13, 13) (`docs/COLLISION.md` §25). Item 4463: both sides
    /// take the Inca capital `4/2000` on 3887 (`check_capture` true on
    /// both), and the hand-over parted: builds' `2661 2010 max_age` ours 0
    /// -> theirs' 1 (`Build::swap_team@00628210:44`–`48` keeps the old
    /// building's), `flags` 39 -> 55 and `visible` 0 -> 8 on the copy
    /// (`Wall::swap_team@00640c00`), the farms' `+0x78` 34..38 -> 9, 10,
    /// 11, 18, 21 (`swap_team:75`–`81`), and the loser's next city's `city`
    /// 0 -> 1 (`City::close@00737550`, from `capture_city:1157`, leaves the slot); cities'
    /// `46 9 traded_with[4]` 0 -> 2 (`City::capture@00736c40:77`–`84`) and
    /// city 8's caravan link and `trade_val` 96 -> 0; units' and guys' 3888
    /// were the caravan `12`, its walk popped by `City::close@00737550:202`'s
    /// `repath` and its route ended in the kill (`unit_masks` 0x4020a ->
    /// 0x4000a). Units and guys 3888 -> 3906, builds -> 3904, ammo -> 3920,
    /// groups 3935 -> 4004, cities -> 3891, world -> 3889; leaders' 3888
    /// stands: player 4's `flags` `+0x2` 08|48. Item 4466 (the capture's
    /// members close before the old city, `capture_city@00733380`'s loop
    /// skipping the centre; `0x400000` and the make list's city slot on the
    /// line): leaders 3888 -> 3889, player 3's `reg_terr[1]` 77|48; every
    /// other line unchanged. Item 4469 (an unassimilated city's claim wins
    /// as −2, its bonuses from `race`'s arrays,
    /// `compute_reg_territory@006b0bb0:425`–`:483`): the captured Inca
    /// capital's 29 cells `154 130 cell` `who` 03 -> theirs' fe on 3889,
    /// `reg_terr[1]` 77 -> 48; leaders 3889 -> 3925, world 3889 -> 3904,
    /// cities 3891 -> whole to 4130 (`45 9 city_flags` 99|89 was
    /// `0x1000`, the pass's near-border mark from a cell it gave player 3).
    const SAMPLE_RANGES_1724: &[(&str, i64, usize, usize, usize, &str)] = &[
        (
            "units",
            3906,
            3644,
            3648,
            73,
            "2038 6 collide_frame..play: dest_angle -541917184|-1260584960, orders_x 32568|32376, orders_y 7032|7608",
        ),
        (
            "builds",
            3904,
            3064,
            3000,
            338,
            "2710 2011 gather_down..infiltrate2: gather_max 0|5",
        ),
        (
            "ammo",
            3920,
            3,
            3,
            1,
            "1 0 rolling..start_roll_angle: +0x7 of 99|99: 439e0000|b99f0000",
        ),
        (
            "deaths",
            3932,
            16,
            12,
            4,
            "12 -1 00936bff: +0x0 of 0|4: |01000000",
        ),
        ("groups", 4004, 628, 628, 1, "166 130 id..march: facing 0|1"),
        (
            "guys",
            3906,
            1012,
            1012,
            1,
            "990 12 guy: end_time 30|120, cur_anim 0|1",
        ),
        (
            "leaders",
            3925,
            667,
            667,
            2,
            "134 1 who..ally_mask: +0xa54 of 26914|26914: 02000800|01000800",
        ),
        (
            "world",
            3904,
            7795,
            7795,
            1,
            "3624 -1 mask: +0x6201 of 115200|115200: e138e138|f138f138",
        ),
    ];
}
