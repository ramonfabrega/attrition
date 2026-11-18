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
mod tests {
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
        if run1654().is_some() {
            assert_eq!(
                (
                    w.records,
                    w.records_parted,
                    w.first_record.as_ref().map(|(f, c, _)| (*f, c.as_str()))
                ),
                SAMPLE_WINDOW_RECORDS,
                "run1654's window (RON_START_WALK_ROWS=1 prints the first parted record)"
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
    const SAMPLE_WINDOW_RECORDS: (usize, usize, Option<(i64, &str)>) = (18, 0, None);

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
    const SAMPLE_WALKED: (i64, i64) = (3019, 21044);

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
    const SAMPLE_LINES: &[(&str, i64)] = &[
        ("units", 92),
        ("builds", 0),
        ("walls", 21044),
        ("ammo", 3022),
        ("deaths", 3365),
        ("groups", 927),
        ("guys", 2920),
        ("leaders", 0),
        ("cities", 3176),
        ("items", 3976),
        ("goods", 1),
        ("world", 0),
    ];

    /// `(frames our walk holds no wall, shot or death, the first it holds
    /// one)`. Minted at 0: none on 3,594 of 21,044, the first shot on 2929,
    /// theirs' frame.
    const SAMPLE_OURS_HELD: (usize, Option<i64>) = (3594, Some(2929));
}
