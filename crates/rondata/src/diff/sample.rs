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
    use crate::diff::held_out::{Battery, SPENT};
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
    /// → 3903, +16. Item 4475 (a region walked to its end takes `0x20`,
    /// `compute_reg_territory@006b0bb0:280`–`282`, and the captured Inca
    /// camp re-walks on 3904 with theirs' 244 shuffle draws): 3903 → 3931,
    /// +28. Item 4488 (a packed Merchant dies with `Unit::close`'s death
    /// draw: `local_1c` needs `unit_masks & 0x80000` clear and
    /// `!is_packing`, `Unit::close@0060ee50:170`–`:194`; computer 1's
    /// Merchant `1/18`, `TypeIndex` 61, land, killed walking on 3931, its
    /// draw theirs' `Unit::close+0xcb6`): 3931 → 4182, +251. Item 4492
    /// (an attacked city's site keeps clear of the enemy,
    /// `produce_building@006e1400` `006e2484`–`006e2575`: computer 1's
    /// Barracks spiral refuses cells (49, 40) and (52, 42) beside the
    /// person's units, and the jitter on (57, 39) spends theirs' four
    /// `Leader::produce_building+0x1805` against two): 4182 → 4223, +41.
    /// Item 4494 (no hits left, no shot: `Build::do_attack@006228f0`
    /// returns at `hits_left() == 0`, `622983`..`6229be`, after the
    /// countdown; computer 1's capital `1/2000`, health 0 of 1200 on 4223,
    /// fired its 16-frame cycle's round at an enemy unit here, its
    /// `Object::fire_ammo+0x3d5` and `Ammo::init+0xcd9` draws where theirs
    /// ran four `Guy::inc_time` animations): 4223 -> 4328, +105. Item 4496
    /// (the computer's own alliance to the top scorer, `Leader::diplomacy@
    /// 006bc950` `6c0926`–`6c0980`: a target not my mutual ally, the top
    /// scorer, with four or more leaders outscoring me, draws before the
    /// human test; on 4328 computer 4, score 592, against the person `3`,
    /// 1172, the top, with 950, 638, 851 and 1172 above it: theirs'
    /// `Leader::diplomacy+0x4035` after the race coin; the coin 16145 odd,
    /// refused by computer 4's three allies; again on 4520, computer 1, and
    /// 4840, computer 4): 4328 -> 5083, +755. Item 4498 (a player's Mine
    /// or Woodcutter's Camp snaps through `snap_center@00636190`'s spiral
    /// before `blocked_site`, `00636304`–`00636328`, the group's `who`
    /// passed by `Group::action_build@00707510`: the person's Mine ordered
    /// at (29695, 10216) on recording frame 5084, refused here `NoMountain`
    /// with range 12 at 1267 past the 1152 reach, stands at (29568, 10368)
    /// with a 205-tile list, theirs' 820 `Build::find_gather_tiles+0x10a`
    /// after the farm rolls on 5083): 5083 -> 5255, +172. Item 4500 (a
    /// scholar leaving a building takes `set_anim(CHAR_DEFAULT, 1, 1)`,
    /// `Unit::come_out@00617c10:548`, `6187a1`..`6187b9`: on 5255 the
    /// person takes computer 1's last Village `1/2007`, computer 1 is
    /// defeated, and the captured University `1/2014` ejects its Scholars
    /// `1/21`, `1/22`, `1/23` (`TypeIndex` 52, land) one a frame, theirs'
    /// `Unit::come_out+0xbae` on 5255, 5256 and 5257): 5255 -> 5273, +18.
    /// On 5273 theirs starts a unit's step (`Unit::move_step+0x823`,
    /// through `Unit::set_anim+0x56`) at game draw 6 where ours goes on to
    /// the `Guy::inc_time` wraps. Item 4521 (that capture's defeat: the
    /// old Village is left to the capture's own close, not killed by
    /// `defeat_by@006d1c80`'s walk, which skips an active city centre, so
    /// its cells stay blocked under `3/2029`; the defeated leader takes
    /// `0x40` and `leader_flags2 & 2`, and no gather or stats pass behind
    /// `process_all@006ed2a0`'s `& 2` gate): 5273 -> 5460, +187. Item 4529
    /// (`Leader::gain_tech@006dcb60`'s `ALLY_LOS` arm: Mathematics ORs
    /// every seated ally, the defeated computer 1 too, into the stored
    /// `ally_mask` the fog and danger tests read): 5460 -> 6015, +555.
    /// Item 4535 (a garrisoned worker whose owner has the militia counts a
    /// third of the militia line's attack, `count_inside@0064dda0`: the
    /// Village `0/2000` reloads in 15 on 5999, not 30): 6015 -> 6114, +99.
    /// Item 4549 (`Group::action_alarm@0070ec30`'s bell takes a gather
    /// building's gatherer chain however far, its building arm on the
    /// whole cell chain: on 6113 computer 2's bell sends `2/11` and `2/14`
    /// from the Woodcutter's Camp `2/2001`, and theirs' `Unit::do_move+0xe84`
    /// on 6114 is `2/11`'s walk to garrison): 6114 -> 7021, +907. On 7021
    /// theirs runs four `Guy::move+0x19f` animations at draw 8 where ours
    /// runs `Guy::inc_time+0x271`.
    const SAMPLE_WALKED: (i64, i64) = (7021, 21044);

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
    /// 3925, cities 3891 -> 4201, world 3889 -> 3904. Item 4475 (a region
    /// walked to its end takes `0x20`, the captured camp's re-walk on
    /// 3904): units 3906 -> 3921, builds 3904 -> 3981, ammo 3920 -> 3936,
    /// guys 3906 -> 3932, leaders 3925 -> 3945, world 3904 -> 3943; groups
    /// 4004 -> 3935, items 5474 -> 5235, goods 4458 -> 4171, named falls
    /// past units' new 3921. Item 4479 (a `REPAIR` order's flags and
    /// target, `TargetOrder::walk_data@0047f220`, unit `3/11` on 3921):
    /// units 3921 -> 3932. Item 4488 (a packed Merchant's death takes its
    /// draw and leaves its `DEATH_OBJS` records, computer 1's `1/18` on
    /// 3931): units 3932 -> 4183, builds 3981 -> 4026, ammo 3936 -> 4192,
    /// deaths 3932 -> 4657, groups 3935 -> 4188, guys 3932 -> 4183, leaders
    /// 3945 -> 4183, cities 4201 -> 4326, items 5235 -> 5434, goods 4171 ->
    /// 4436, world 3943 -> 4183. Item 4489 (`get_building_cities@006e06f0`
    /// counts an assimilated city's library only, `race == who`: the
    /// person's Library `3/2005` runs one slot while its second library
    /// stands in the Inca city taken on 3887): builds 4026 -> 4183, onto
    /// the draw word; deaths 4657 -> 4635, items 5434 -> 5097, named falls
    /// past builds' new 4183; goods 4436 -> 4449. Item 4492 (an attacked
    /// city's site keeps clear of the enemy; draws 4182 -> 4223): units
    /// 4183 -> 4224, builds 4183 -> 4224, ammo 4192 -> 4224, groups 4188 ->
    /// 4244, guys 4183 -> 4226, leaders 4183 -> 4232, cities 4326 -> 4401,
    /// items 5097 -> 5760, goods 4449 -> 4559, world 4183 -> 4229; deaths
    /// 4635 -> 4625, a named fall past the new lowest 4224. Item 4494 (no
    /// hits left, no shot: `Build::do_attack@006228f0` returns at
    /// `hits_left() == 0`, `6229be`; computer 1's capital `1/2000` at
    /// health 0 on 4223; draws 4223 -> 4328): units 4224 -> 4248, builds
    /// 4224 -> 4321, ammo 4224 -> 4347, deaths 4625 -> 4643, groups 4244 ->
    /// 4333, guys 4226 -> 4330, leaders 4232 -> 4321, world 4229 -> 4334;
    /// cities and goods unchanged; items 5760 -> 5666, a named fall past the
    /// new lowest 4248. Item 4496 (the computer's own alliance to the top
    /// scorer draws at `6c0980` before the human test,
    /// `Sim::top_scorer_alliance`; computer 4 on 4328, four above it, the person
    /// `3` the top; draws 4328 -> 5083): ammo 4347 -> 4462, deaths 4643 ->
    /// 5135, groups 4333 -> 5085, guys 4330 -> 5085, cities 4401 -> 5126,
    /// goods 4559 -> 5403, world 4334 -> 5085; units 4248, builds 4321 and
    /// leaders 4321 unchanged; items 5666 -> 5403, a named fall past the
    /// unchanged lowest 4248. Item 4498 (a player's Mine snaps through
    /// `snap_center@00636190`'s spiral before `blocked_site`: the person's
    /// Mine `3/2028`, ordered on recording frame 5084, built a tile south
    /// with 205 tiles; draws 5083 -> 5255): deaths 5135 -> 5428, groups
    /// 5085 -> 5282, guys 5085 -> 5256, cities 5126 -> 5326, world 5085 ->
    /// 5256; units 4248, builds 4321, ammo 4462 and leaders 4321
    /// unchanged; items 5403 -> 5286 and goods 5403 -> 5384, named falls
    /// past the unchanged lowest 4248. Item 4500 (a scholar rises from its
    /// host with the forced idle roll, `Unit::come_out@00617c10:548`;
    /// draws 5255 -> 5273): deaths 5428 -> 5429, groups 5282 -> 5311, guys
    /// 5256 -> 5273, items 5286 -> 12663, goods 5384 -> 5392; units 4248,
    /// builds 4321, ammo 4462, leaders 4321, cities 5326 and world 5256
    /// unchanged. Item 4505 (a gatherer pushed onto a second chain keeps
    /// the first's link to it, `Build::add_gatherer@0062f640`; computer
    /// 1's Citizen `1/11` on 4247): units 4248 -> 5273, onto the draw
    /// word; every other line unchanged. Item 4513 (an age is cheaper for
    /// a leader behind a non-allied rival, `TypeData::get_cost@00664090`'s
    /// age-behind arm; the person's Classical Age at Library `3/2005` on
    /// 4320, `cost[0]` ours 200, theirs 180): builds 4321 -> 5293, leaders
    /// 4321 -> 5256; items 12663 -> 8734, a named fall past the draw word
    /// 5273; every other line unchanged, ammo 4462 the lowest. Item 4515
    /// (a building's round punctures by its piece,
    /// `Ammo::do_damage@00678060`'s `verify_ammo_flags(+0x8, 1)`; gaia's
    /// `0/2016` arrow at the person's `3/0`, rolled and missed, lands on
    /// 4461 and reads `flags 1` on 4462 where ours closed it): ammo 4462
    /// -> 5299; every other line unchanged, leaders and world 5256 the
    /// lowest. Item 4521 (computer 1's last Village `1/2007` taken on 5256:
    /// the defeat's walk leaves the old centre to the capture, the 2x2
    /// cells 2623/2624/2683/2684 stay `blocked` 9 under `3/2029`; the
    /// defeated leader's `leader_flags` 0x01 -> 0x41 and `leader_flags2` 0
    /// -> 2; no gather, wall or unit stats for it on 5257): units 5273 ->
    /// 5461, builds 5293 -> 5487, ammo 5299 -> 5969, deaths 5429 -> 6190,
    /// groups 5311 -> 5461, guys 5273 -> 5461, leaders 5256 -> 5326, cities
    /// 5326 -> 5801, items 8734 -> 10910, goods 5392 -> 6395, world 5256 ->
    /// 5401; leaders 5326 the lowest. Item 4525 (step 7's clear and step
    /// 10's count of `invaders` gate on the slot's `leader_flags & 1`,
    /// which the defeat keeps: leader 1's `invaders[3]` on 5326 ours 5,
    /// held from 5256, theirs 0): leaders 5326 -> 5403; every other line
    /// unchanged, world 5401 the lowest. Item 4526 (`build_is_seen` and
    /// the fog test mask by the stored `ally_mask`, `LeaderData +0x6929`,
    /// which drops a defeated ally: run1772's danger rebuild of 5400,
    /// `3629 0 danger` `+0x858` ours 20, theirs 8): world 5401 -> 5462;
    /// items 10910 -> 10724, a named fall past units, groups and guys on
    /// 5461 (the danger read alone gives 12663; the fog test's attack
    /// callers bring it to 10724; no range record reaches it, DANGER §8);
    /// leaders 5403 the lowest. Item 4529 (`Leader::new_rare@006d9e70`
    /// gates its recipients on `leader_flags & 1`, which the defeated
    /// computer 1 keeps: run1772's `190 1 new_rares` ours 0x0b, theirs
    /// 0x0c on 5403): leaders 5403 -> 5427, every other line unchanged;
    /// leaders 5427 the lowest. Item 4529, second (`gain_tech@006dcb60`'s
    /// `ALLY_LOS` arm: computer 2 gains Mathematics on 5427 and ORs the
    /// seated, defeated computer 1 back into its stored `ally_mask`,
    /// `271 2 who..ally_mask` ours 0x15, theirs 0x17): leaders 5427 ->
    /// 5486, units 5461 -> 6000, builds 5487 -> 6000, ammo 5969 -> 6000,
    /// groups 5461 -> 6022, guys 5461 -> 6017, world 5462 -> 6022, cities
    /// 5801 -> 6151; two named falls past units and draws on 6000 and
    /// 6015: items 10724 -> 7940 and goods 6395 -> 6281 (good 52's
    /// `ever_seen`, ours leader 2's bit, theirs none). Item 4533
    /// (`capture_city@00733380`'s spoils skip Knowledge: the person's
    /// capture on 5485 pays Metal, `500 3 encrypt bucket[3]` ours 0xb4,
    /// theirs 0x69, `509 3 bucket[4]` ours 0xa2, theirs 0xed): leaders
    /// 5486 -> 5891, every other line unchanged. Item 4533, second (a
    /// leader's `SimpleArray<int>` doubles from ten: the person's
    /// `oil_patches` 21 elements, `462 3` size ours 30, theirs 40): leaders
    /// 5891 -> 6005, past units, builds and ammo, which are the lowest on
    /// 6000. Item 4535 (`count_inside@0064dda0`'s `GARRISON_ARROWS` arm
    /// counts a worker whose owner has the militia as a third of the
    /// militia line's attack: the Village `0/2000`'s five Citizens on
    /// 5999, `12 2000 recharging` ours 30, theirs 15, `attack_ox` 28|34):
    /// units 6000 -> 6108, builds 6000 -> 6114, ammo 6000 -> 6156, groups
    /// 6022 -> 6153, guys 6017 -> 6115, leaders 6005 -> 6119, world 6022
    /// -> 6116; items 7940 -> 9433 and goods 6281 -> 6407, named rises
    /// past the draw word; units 6108 the lowest. Item 4543
    /// (`UnitData::get_attrition@00608fd0`'s aged-up arm reads each
    /// leader's `ages`, `*(+0x6eb8) +0xdc ^ 0x62766`, which ours never
    /// fed: the person's Bushi `3/37` on computer 0's ground on 6108,
    /// leader 0's `ages` 2 against the person's 1, `3107 37 attrition`
    /// ours 48, theirs 24): units 6108 -> 6114, deaths 6190 -> 6186;
    /// items 9433 -> 7372 and goods 6407 -> 6542, past the draw word;
    /// units, builds and draws 6114 the lowest. Item 4549
    /// (`Group::action_alarm@0070ec30`'s bell walks each circle cell's
    /// whole chain, and a gather building there hands over its gatherer
    /// chain however far: computer 2's bell on 6113 takes `2/11` and
    /// `2/14` at the Woodcutter's Camp `2/2001`, 21 tiles out, `1665 11
    /// orders_x` ours 9576, theirs 6168): units 6114 -> 6669, builds 6114
    /// -> 6162, ammo 6156 -> 7023, deaths 6186 -> 7085, groups 6153 ->
    /// 6928, guys 6115, leaders 6119, cities 6151 and world 6116 -> 7021,
    /// items 7372 -> 7544, goods 6542 -> 7423; builds 6162 the lowest.
    /// Item 4552 (`Wall::check_ever_seen@0063ce70` filters an unstarted
    /// building with the owner's stored `+0x6929`: the Temple `2/2025`
    /// placed on 6161, `2515 2025 ever_seen` ours 7, theirs 5, seat 2's
    /// byte 5): builds 6162 -> 7021; units 6669 the lowest. Item 4555
    /// (`Unit::check_build_order@00603470`'s kill is the front's:
    /// `repath` strips the stepped-over `MOVE_TO`, and the person's
    /// Citizen `3/55`, `[BUILD_AT 2041 (finished), MOVE_TO, BUILD_AT
    /// 2042]` on 6668, keeps one order on the Stable `3/2042`, `3574 55
    /// orders.count` ours 2, theirs 1): units 6669 -> 6928; every other
    /// line unchanged; units and groups 6928 the lowest. Item 4557
    /// (`Unit::come_out@00617c10`'s enemy arm: the person's Barracks
    /// `3/2041` has its point (4433, 30955) on player 2's `2/2010`, and
    /// the three Bushi it trains on 6927 attack-move to the point itself,
    /// `QUEUE_NEW`, no action bit; `263 197 id..march` `ox` ours 4440,
    /// theirs 4433): units 6928 -> 7021, groups 6928 -> 7097; the draw
    /// word 7021 the lowest, units, builds, guys, leaders, cities and world
    /// tied there.
    const SAMPLE_LINES: &[(&str, i64)] = &[
        ("units", 7021),
        ("builds", 7021),
        ("walls", 21044),
        ("ammo", 7023),
        ("deaths", 7085),
        ("groups", 7097),
        ("guys", 7021),
        ("leaders", 7021),
        ("cities", 7021),
        ("items", 7544),
        ("goods", 7423),
        ("world", 7021),
    ];

    /// The headline's other games, each whole on the twelve lines over
    /// its own length and held there by its own test (DECISIONS 142 §1,
    /// 144 §1): the minted game, the four live windows the client
    /// recorded, and the demand's game. The sample is the seventh.
    const PARITY_OTHERS_WHOLE: usize = 6;

    /// Whether the sample is whole: its lowest first parting at or past
    /// [`SAMPLE_HOOK`]'s frames.
    fn sample_whole() -> bool {
        SAMPLE_LINES
            .iter()
            .map(|(_, f)| *f)
            .min()
            .expect("twelve lines")
            >= SAMPLE_HOOK.1
    }

    /// `(whole, of)` held out: each spent battery's rows whole now, by
    /// `held_out::whole_now` (its `now` column, never its found), of its
    /// declared rows.
    fn held_out_count(spent: &[&Battery]) -> (usize, usize) {
        spent.iter().fold((0, 0), |(w, of), b| {
            (w + b.whole_now(), of + b.scores.len())
        })
    }

    /// `(whole, of)` for the headline over `spent` (DECISIONS 150 §1): the
    /// six, the sample, and [`held_out_count`].
    fn parity_count(spent: &[&Battery]) -> (usize, usize) {
        let (w, of) = held_out_count(spent);
        (
            PARITY_OTHERS_WHOLE + usize::from(sample_whole()) + w,
            PARITY_OTHERS_WHOLE + 1 + of,
        )
    }

    /// **The fold counts each spent battery's whole rows, and nothing
    /// else** (item 4528, DECISIONS 150 §1). Off the pins: the six, the
    /// sample, and each spent battery's rows whole now, which its own
    /// pinned line ([`Battery::line`]) states and `line_reads_as_pinned`
    /// checks against its table. A battery off [`SPENT`] adds nothing, and
    /// dropping one from the list drops exactly its rows.
    #[test]
    fn the_fold_counts_the_spent_batteries_whole_rows() {
        let base = (
            PARITY_OTHERS_WHOLE + usize::from(sample_whole()),
            PARITY_OTHERS_WHOLE + 1,
        );
        let pinned: usize = SPENT.iter().map(|b| b.line.0).sum();
        let rows: usize = SPENT.iter().map(|b| b.scores.len()).sum();
        assert_eq!(
            parity_count(SPENT),
            (base.0 + pinned, base.1 + rows),
            "the headline folds each spent battery's whole rows"
        );
        assert_eq!(parity_count(&[]), base, "no battery spent, none folds");
        let (last, rest) = SPENT.split_last().expect("a battery spent");
        let (w, of) = parity_count(SPENT);
        assert_eq!(
            parity_count(rest),
            (w - last.whole_now(), of - last.scores.len()),
            "{} off the list folds none of its rows",
            last.label
        );
        // Each battery on the list is spent: a row of it parted as found,
        // which is what a lane reads a parting off (entry 145 §2).
        for b in SPENT {
            assert!(
                b.scores.iter().flatten().any(|p| !p.found.first.is_empty()),
                "{} is on SPENT with no row that parted as found",
                b.label
            );
        }
    }

    /// The queue's `Parity:` line, off the pins: the games whole of the
    /// declared set, the sample's lowest first parting with its line
    /// ([`SAMPLE_LINES`], of [`SAMPLE_HOOK`]'s frames), and the draw
    /// walk's word beside it ([`SAMPLE_WALKED`]). The word is the walk's
    /// instrument and the first parting is the score: four landings moved
    /// the word 4328 -> 5273 while units stood on 4248, and only the
    /// prose said so (DECISIONS 148 §5). Since item 4528 the count folds
    /// each spent battery's rows whole now ([`SPENT`], DECISIONS 150 §1),
    /// and the held-out half beside it says how many of the count are
    /// theirs.
    fn parity_line() -> String {
        let frames = SAMPLE_HOOK.1;
        let (line, first) = SAMPLE_LINES
            .iter()
            .min_by_key(|(_, f)| *f)
            .expect("twelve lines");
        let (whole, of) = parity_count(SPENT);
        let (held, held_of) = held_out_count(SPENT);
        let sample = if *first >= frames {
            "the sample whole".to_string()
        } else {
            format!(
                "the sample parts first on {line} {first} of {frames} \u{b7} draws {}",
                SAMPLE_WALKED.0
            )
        };
        format!("Parity: {whole} of {of} whole \u{b7} held out {held} of {held_of} \u{b7} {sample}")
    }

    /// **The handoff's parity line is the pinned sample** (the pass of
    /// 2026-11-20, second; DECISIONS 147 §5). The headline's count stood
    /// in prose for six records, and a booking named 3888 where
    /// [`SAMPLE_LINES`] pinned world on 3878 (entry 146 §4). A worker
    /// re-pins the lines; the driver writes this line at the booking.
    #[test]
    fn the_handoff_s_parity_line_is_the_pinned_sample() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../docs/QUEUE.md");
        let q = std::fs::read_to_string(path).expect("docs/QUEUE.md");
        let want = parity_line();
        let line = q.lines().find(|l| l.starts_with("Parity:"));
        assert_eq!(
            line.map(str::trim_end),
            Some(want.as_str()),
            "docs/QUEUE.md's handoff carries `{want}` (`diff::sample::SAMPLE_LINES`), \
             above the `Parity held out:` line"
        );
    }

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
    /// the frames not read one by one; the first shot still on 2929. Item
    /// 4475 (a region walked to its end takes `0x20`, the captured camp's
    /// re-walk on 3904): 4473 -> 5271, a named rise past every parting,
    /// the frames not read one by one; the first shot still on 2929. Item
    /// 4488 (a packed Merchant's death takes its draw and leaves its
    /// records, `1/18` on 3931): 5271 -> 4767, a named fall, the frames not
    /// read one by one (the war after 3931 runs on another draw sequence);
    /// the first shot still on 2929. Item 4489 (an unassimilated city's
    /// library adds no research slot): 4767 -> 6275, a named rise past
    /// every parting, the frames not read one by one; the first shot still
    /// on 2929. Item 4492 (an attacked city's site keeps clear of the
    /// enemy): 6275 -> 4172, a named fall, the frames not read one by one;
    /// the first shot still on 2929. Item 4494 (no hits left, no shot: a
    /// building at health 0 fires nothing, `Build::do_attack@006228f0`
    /// `6229be`): 4172 -> 4946, a named rise past every parting, the frames
    /// not read one by one; the first shot still on 2929. Item 4496 (the
    /// computer's own alliance to the top scorer draws at `6c0980`, three
    /// times to 5083): 4946 -> 4920, a named fall, the frames not read one
    /// by one; the first shot still on 2929. Item 4498 (a player's Mine
    /// snaps through `snap_center`'s spiral, the person's `3/2028` built on
    /// 5083 where ours refused it): 4920 -> 3475, a named fall, the frames
    /// not read one by one (the war past 5083 runs with the person's mine
    /// standing and its 820 draws spent); the first shot still on 2929.
    /// Item 4500 (the three Scholars rise from the captured University
    /// with theirs' forced idle rolls on 5255..5257): 3475 -> 3747, +272;
    /// the first shot still on 2929. Item 4513 (an age is cheaper for a
    /// leader behind a non-allied rival; the person's Classical Age at 180
    /// food on 4320): 3747 -> 4772, a named rise past the draw word 5273,
    /// the frames not read one by one; the first shot still on 2929. Item
    /// 4515 (a building's round punctures by its piece, so a building's
    /// `BuildingArrow` that strikes nothing stays 200 frames as a spent
    /// arrow): 4772 -> 4723, -49, a named fall, the frames not read one by
    /// one; the first shot still on 2929. Item 4521 (the defeat on 5256
    /// keeps the captured Village's cells and stops the defeated leader's
    /// gather and stats; the draw word 5273 -> 5460): 4723 -> 3624, -1099,
    /// a named fall, the frames not read one by one; the first shot still
    /// on 2929. Item 4526 (`build_is_seen` and the fog test mask by the
    /// stored `ally_mask`, which drops the defeated computer 1): 3624 ->
    /// 3729, +105, a named rise past the draw word 5460, the frames not
    /// read one by one; the first shot still on 2929.
    /// Item 4529 (`gain_tech@006dcb60`'s `ALLY_LOS` arm puts the defeated
    /// computer 1 back in computer 2's stored `ally_mask` on 5427; units
    /// whole to 6000, the draws to 6015): 3729 -> 3206, a named fall past
    /// the draw word, the frames not read one by one; the first shot
    /// still on 2929. Item 4533 (the capture's spoils skip Knowledge; a
    /// leader's array doubles from ten): 3206 -> 3215, +9, a named rise
    /// past the draw word 6015, the frames not read one by one; the first
    /// shot still on 2929. Item 4535 (a garrisoned worker counts as
    /// militia in a building's arrows): 3215 -> 3289, +74, a named rise
    /// past the draw word 6114, the frames not read one by one; the first
    /// shot still on 2929. Item 4543 (attrition's aged-up arm reads both
    /// leaders' `ages`): 3289 -> 3176, -113, a named fall past the draw
    /// word 6114, the frames not read one by one; the first shot still on
    /// 2929. Item 4549 (the bell takes a gather building's gatherer
    /// chain): 3176 -> 3315, +139, a named rise past the draw word 7021,
    /// the frames not read one by one; the first shot still on 2929.
    /// Item 4542 (`get_cost`'s locked-team wonder term) moved it 3176 ->
    /// 3182 on its own branch; on the tree merged with 4549 it reads 3315,
    /// both builds in. Item 4545 (the caravan limit counts a mutual ally's
    /// cities): 3315 -> 3382 on the tree merged with 4549, +67, a named
    /// rise past the draw word 7021; with 4542 in too, 3315 -> 3184,
    /// -131, a named fall past the draw word 7021, the frames not read one
    /// by one; the first shot still on 2929. Item 4555 (`check_build_order`'s
    /// kill is the front's): 3184 -> 3244, +60, a named rise past the draw
    /// word 7021, the frames not read one by one; the first shot still on
    /// 2929.
    /// Item 4553 (`find_target`'s ally arm reads the leader's
    /// `strong[8]`/`weak[8]`): on its own branch 3315 -> 3314, -1; on the
    /// tree merged with 4545, 3184 -> 3174, -10, a named fall past the
    /// draw word 7021, the frames not read one by one; the first shot still
    /// on 2929. On the tree merged with 4555 too, 3244 -> 3174, -70, a
    /// named fall past the draw word 7021; the first shot still on 2929.
    /// Item 4557 (`come_out`'s enemy arm sends the Bushi out of
    /// `3/2041` to the point on `2/2010` itself): 3244 -> 3174, -70, a
    /// named fall past the draw word 7021, on its own branch; on the tree
    /// merged with 4553, 3174 -> 3175, +1, a named rise past the draw word
    /// 7021, the frames not read one by one; the first shot still on 2929.
    /// Item 4547 (piece 736's bays, the Slingers in art style 2): 3184 ->
    /// 4792 on its own branch, +1608 (with the three rows keyed off,
    /// `tools/mutate.py`, 3184 passes); on the tree merged with 4553 and
    /// 4555, 3174 -> 3903, +729, a named rise past the draw word 7021, the
    /// frames not read one by one; the first shot still on 2929. On the
    /// tree merged with both (the driver's merge of 4547 onto 4557):
    /// 3174, measured; the first shot still on 2929.
    const SAMPLE_OURS_HELD: (usize, Option<i64>) = (3174, Some(2929));

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

    /// run1726's record (item 4505, `docs/RUNS.md` run1726): the same
    /// playback to 4,500 under the same hooks, keeping units, builds, ammo
    /// and leaders alone (`RNG_MASK` 0x8b), the four lines that parted
    /// below the draw word past run1724's end (DECISIONS 148 §5).
    /// `RON_SAMPLE_RANGES_1726`, else where the take left it.
    fn run1726() -> Option<crate::trace::ranges::Record> {
        ranges_record("RON_SAMPLE_RANGES_1726", "run1726")
    }

    /// run1771's record (item 4521, `docs/RUNS.md` run1771): the same
    /// playback to 5,350 under the same hooks, keeping units, builds, ammo,
    /// guys, leaders and world alone (`RNG_MASK` 0x8cb), the six lines that
    /// part below 5300 on the hook's sums (DECISIONS 148 §4, §5).
    /// `RON_SAMPLE_RANGES_1771`, else where the take left it.
    fn run1771() -> Option<crate::trace::ranges::Record> {
        ranges_record("RON_SAMPLE_RANGES_1771", "run1771")
    }

    /// run1772's record (item 4526, `docs/RUNS.md` run1772): the same
    /// playback to 6,400 under the same hooks, keeping the ten lines that
    /// part below 6400 on the hook's sums (`RNG_MASK` 0xdfb: all but walls
    /// and items; DECISIONS 149 §2). `RON_SAMPLE_RANGES_1772`, else where
    /// the take left it.
    fn run1772() -> Option<crate::trace::ranges::Record> {
        ranges_record("RON_SAMPLE_RANGES_1772", "run1772")
    }

    /// run1786's record (item 4555, `docs/RUNS.md` run1786): the same
    /// playback to 8,000 under the same hooks, keeping the eleven lines
    /// that part below 8000 on the hook's sums (`RNG_MASK` 0xffb: all but
    /// walls; DECISIONS 148 §5). `RON_SAMPLE_RANGES_1786`, else where the
    /// take left it.
    fn run1786() -> Option<crate::trace::ranges::Record> {
        ranges_record("RON_SAMPLE_RANGES_1786", "run1786")
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

    /// **The sample's kept lines by range, to 4,500** (item 4505): the
    /// same walk over run1726's record, the four lines of its mask alone,
    /// pinned per line by [`SAMPLE_RANGES_1726`].
    /// `RON_SAMPLE_RANGES_DETAIL` and `RON_SAMPLE_RANGES_ALL` print as
    /// above.
    #[test]
    fn the_sample_s_ranges_to_4500_name_each_first_parting() {
        let Some(rec) = run1726() else { return };
        let Some((frames, got)) =
            walk_kept_ranges(rec, SAMPLE_RANGES_1726_WINDOW, SAMPLE_RANGES_1726_KEPT)
        else {
            return;
        };
        let pinned: Vec<SampleRange> = SAMPLE_RANGES_1726
            .iter()
            .map(|&(l, f, t, o, p, x)| (l, f, t, o, p, x.to_string()))
            .collect();
        assert_eq!(
            frames,
            SAMPLE_RANGES_1726_WINDOW as usize + 1,
            "run1726's sets, one a frame"
        );
        assert_eq!(got, pinned, "SAMPLE_RANGES_1726");
    }

    /// **The sample's kept lines by range, to 5,350** (item 4521): the
    /// same walk over run1771's record, the six lines of its mask alone,
    /// pinned per line by [`SAMPLE_RANGES_1771`].
    /// `RON_SAMPLE_RANGES_DETAIL` and `RON_SAMPLE_RANGES_ALL` print as
    /// above.
    #[test]
    fn the_sample_s_ranges_to_5350_name_each_first_parting() {
        let Some(rec) = run1771() else { return };
        let Some((frames, got)) =
            walk_kept_ranges(rec, SAMPLE_RANGES_1771_WINDOW, SAMPLE_RANGES_1771_KEPT)
        else {
            return;
        };
        let pinned: Vec<SampleRange> = SAMPLE_RANGES_1771
            .iter()
            .map(|&(l, f, t, o, p, x)| (l, f, t, o, p, x.to_string()))
            .collect();
        assert_eq!(
            frames,
            SAMPLE_RANGES_1771_WINDOW as usize + 1,
            "run1771's sets, one a frame"
        );
        assert_eq!(got, pinned, "SAMPLE_RANGES_1771");
    }

    /// **The sample's kept lines by range, to 6,400** (item 4526): the
    /// same walk over run1772's record, the ten lines of its mask alone,
    /// pinned per line by [`SAMPLE_RANGES_1772`].
    /// `RON_SAMPLE_RANGES_DETAIL` and `RON_SAMPLE_RANGES_ALL` print as
    /// above.
    #[test]
    fn the_sample_s_ranges_to_6400_name_each_first_parting() {
        let Some(rec) = run1772() else { return };
        let Some((frames, got)) =
            walk_kept_ranges(rec, SAMPLE_RANGES_1772_WINDOW, SAMPLE_RANGES_1772_KEPT)
        else {
            return;
        };
        let pinned: Vec<SampleRange> = SAMPLE_RANGES_1772
            .iter()
            .map(|&(l, f, t, o, p, x)| (l, f, t, o, p, x.to_string()))
            .collect();
        assert_eq!(
            frames,
            SAMPLE_RANGES_1772_WINDOW as usize + 1,
            "run1772's sets, one a frame"
        );
        assert_eq!(got, pinned, "SAMPLE_RANGES_1772");
    }

    /// The same walk over run1786's record, the eleven lines of its mask
    /// alone, pinned per line by [`SAMPLE_RANGES_1786`].
    /// `RON_SAMPLE_RANGES_DETAIL` and `RON_SAMPLE_RANGES_ALL` print as
    /// above.
    #[test]
    fn the_sample_s_ranges_to_8000_name_each_first_parting() {
        let Some(rec) = run1786() else { return };
        let Some((frames, got)) =
            walk_kept_ranges(rec, SAMPLE_RANGES_1786_WINDOW, SAMPLE_RANGES_1786_KEPT)
        else {
            return;
        };
        let pinned: Vec<SampleRange> = SAMPLE_RANGES_1786
            .iter()
            .map(|&(l, f, t, o, p, x)| (l, f, t, o, p, x.to_string()))
            .collect();
        assert_eq!(
            frames,
            SAMPLE_RANGES_1786_WINDOW as usize + 1,
            "run1786's sets, one a frame"
        );
        assert_eq!(got, pinned, "SAMPLE_RANGES_1786");
    }

    /// The walk the ranges tests share: our sample from state 0 to
    /// `window`, each frame's twelve lines against the record's set.
    /// `(sets compared, each parting line's first parting)`, or `None` when
    /// the install, the sample or the pieces are absent.
    fn walk_ranges(
        rec: crate::trace::ranges::Record,
        window: i64,
    ) -> Option<(usize, Vec<SampleRange>)> {
        walk_kept_ranges(rec, window, 0xfff)
    }

    /// [`walk_ranges`] over a take that keeps the lines of `kept` alone
    /// (the hook's `RNG_MASK`, one bit a line in `STATE_LINES`' order):
    /// a line the take dropped is not compared (DECISIONS 148 §4).
    fn walk_kept_ranges(
        mut rec: crate::trace::ranges::Record,
        window: i64,
        kept: u16,
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
                    if kept & (1 << k) == 0 {
                        continue;
                    }
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
                                    .take(16)
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
    /// Item 4475 (a region walked to its end takes `flags |= 0x20`,
    /// `compute_reg_territory@006b0bb0:280`–`282`, so the captured Inca
    /// camp `3/2011` re-walks on 3904 through `Build::process`'s `0x10`
    /// arm): builds' `2710 2011 gather_max` ours 0 -> theirs' 5 and
    /// `2715 2011 gather_from.length` 0 -> 61, world `3624 -1 mask` 61
    /// tiles' `0x1000` clear -> set on 3904; builds 3904 -> 3981, world
    /// 3904 -> 3943, units 3906 -> 3921, guys 3906 -> 3932, ammo 3920 ->
    /// 3936, leaders 3925 -> 3945, groups 4004 -> 3935 (after units' and
    /// guys' new partings); deaths' 3932 stands. Item 4479 (a `REPAIR`
    /// walks `TargetOrder::walk_data@0047f220`, the flags and the target):
    /// units' `2190 11` unit `3/11`'s second order, `order.flags` 04|04 and
    /// `order.target` (2010, 3, 0x31) on both sides, absent in ours before
    /// -> agree; units 3921 -> 3932, onto guys' parting; every other line
    /// unchanged. Item 4488 (`Unit::close@0060ee50:170`–`:194`'s
    /// `local_1c` exempts only an unpacked Merchant that is not packing):
    /// computer 1's packed Merchant `1/18` killed walking on 3931 leaves
    /// deaths' `12..15 -1` on 3932, its figure and crew member, `cur_anim`
    /// 18 at (31910, 19682) and (31742, 19575), absent in ours before ->
    /// agree; every line but builds whole to 4130, builds 3981 -> 4026.
    /// Item 4489 (`LeaderData::get_building_cities@006e06f0` takes a city
    /// only with `race == who`): the person's Library `3/2005` queues
    /// `TypeIndex` 559 behind 552 on 4025, and its owner's second library
    /// `3/2015` stands in city `3/2010`, the Inca capital's taken on 3887,
    /// race 4 (assimilation timer 3887); `2810 2005 queue[k]` `job_counter`
    /// ours 100 -> theirs' 0 on 4026, one slot run, not two. Whole to 4130:
    /// one row -> none.
    const SAMPLE_RANGES_1724: &[(&str, i64, usize, usize, usize, &str)] = &[];

    /// run1726's last frame, ours: the record holds trace frames −1 to 4499.
    const SAMPLE_RANGES_1726_WINDOW: i64 = 4500;

    /// run1726's mask: units, builds, ammo and leaders (`RNG_MASK=0x8bu`).
    const SAMPLE_RANGES_1726_KEPT: u16 = 0x8b;

    /// Per kept line that parts on run1726's window, its first parting by
    /// range (item 4505). Minted at four rows on the sums' frames: units
    /// 4248, `1077 11 collide_frame..play` `gather_down` ours 12, theirs 8,
    /// computer 1's Citizen `1/11` (`TypeIndex` 50, land, uid #109),
    /// pushed onto `#2009`'s chain on 4247 while `#2001`'s still held it
    /// behind `1/13`: this crate kept one list a building and read its
    /// link off `#2001`'s, where the original's one link a unit had taken
    /// `#2009`'s next, `1/8`. Built (`Sim::gather_chain`): units whole to
    /// 4500, one row -> none. Item 4513: builds 4321, `2559 2005 queue[k]`
    /// `cost[0]` ours 200, theirs 180, and leaders 4321, `473 3 encrypt`
    /// `bucket[0]` ours 0x213 (531), theirs 0x227 (551), are one price: the
    /// person (still Ancient) queues the Classical Age behind tech 558 at
    /// Library `3/2005` on 4320 while a computer holds it, and the
    /// original's `get_cost` takes `TECH_AGE_BEHIND_DISCOUNT` off; the 20
    /// food stays in theirs' bucket. Built (`Sim::ages_behind`): both whole
    /// to 4500, three rows -> one. Item 4515: ammo 4462, `0 -1 0067ab7e`
    /// absent in ours, is gaia's `0/2016` `BuildingArrow` stuck in the
    /// ground (`flags 1`, `cur_time` 0, `ex`/`ey`/`ez` (19677, 42075, 183),
    /// `total_time` 23 off the rolled arm), which ours closed because a
    /// building's round never punctured here. Built (the building arm's
    /// `punctures` off its piece): ammo whole to 4500, one row -> none;
    /// every kept line whole on run1726's window.
    const SAMPLE_RANGES_1726: &[(&str, i64, usize, usize, usize, &str)] = &[];

    /// run1771's last frame, ours: the record holds trace frames −1 to 5349.
    const SAMPLE_RANGES_1771_WINDOW: i64 = 5350;

    /// run1771's mask: units, builds, ammo, guys, leaders and world
    /// (`RNG_MASK=0x8cbu`).
    const SAMPLE_RANGES_1771_KEPT: u16 = 0x8cb;

    /// Per kept line that parts on run1771's window, its first parting by
    /// range (item 4521). Minted at six rows: leaders and world 5256, units
    /// and guys 5273, builds 5293, ammo 5299. Leaders 5256, `137 1 flags`
    /// `leader_flags` ours 0x01, theirs 0x41, `leader_flags2` ours 0,
    /// theirs 2: computer 1, defeated by the person (`defeated_by` -1 -> 3,
    /// both sides) as its last Village `1/2007` is taken, takes
    /// `Leader::defeat@006ecb00`'s `0x40` and `flags2 | 2` in theirs. World
    /// 5256, `2647 2623 cell` `flags` 0|0x80, `blocked`/`bad`/`solid`
    /// 0/1/0|9/0x10/9 on the four cells under the Village, held in theirs
    /// since before 5255: ours' defeat killed the old centre while the
    /// capture still held it, and `defeat_by@006d1c80`'s walk skips an
    /// active city centre (`+0x8 & 0x20`). Built: the two flag bits, the
    /// skip, and `process_all@006ed2a0`'s `& 2` gate over `gather` and both
    /// stats passes (leaders 5257, `137 1 flags` `0x8000000` ours clear,
    /// theirs raised, and the stock): six rows -> one. Item 4525: leaders
    /// 5326, `138 1 who..ally_mask` `+0x294` (`LeaderData +0x29c`,
    /// `invaders[3]`) ours 5, theirs 0: the defeated leader 1's count,
    /// which ours froze at its defeat and `plan_strategy@006b9620` clears
    /// and recounts under `leader_flags & 1`. Built: one row -> none;
    /// every kept line whole on run1771's window.
    const SAMPLE_RANGES_1771: &[(&str, i64, usize, usize, usize, &str)] = &[];

    /// run1772's last frame, ours: the record holds trace frames −1 to 6399.
    const SAMPLE_RANGES_1772_WINDOW: i64 = 6400;

    /// run1772's mask: every line but walls and items (`RNG_MASK=0xdfbu`).
    const SAMPLE_RANGES_1772_KEPT: u16 = 0xdfb;

    /// Per kept line that parts on run1772's window, its first parting by
    /// range (item 4526). Minted at ten rows, each on the frame the hook's
    /// sums part (`SAMPLE_LINES`): world 5401, leaders 5403, units, groups
    /// and guys 5461, builds 5487, cities 5801, ammo 5969, deaths 6190,
    /// goods 6395. World 5401, `3629 0 danger` `+0x858` ours 20, theirs 8:
    /// the rebuild of 5400, the person's 13 buildings taken off computer 1
    /// at its defeat seen by computers 0, 2 and 4 in ours through computer
    /// 1's `ever_seen` bit; `WallData::is_seen@00642bd0` and
    /// `WorldData::is_seen@006b55c0` mask by the stored `ally_mask`
    /// (`LeaderData +0x6929`), which the census rewrites without a
    /// defeated leader. Built (item 4526): world 5401 -> 5462. Item 4529:
    /// leaders 5403, `190 1 new_rares` count ours 0x0b, theirs 0x0c, and
    /// `194 1` theirs one entry more, good 31: `Leader::new_rare@006d9e70`
    /// hands an ally's find to every slot with `leader_flags & 1`, which a
    /// defeated leader keeps, where ours skipped the defeated. Built:
    /// leaders 5403 -> 5427, `271 2 who..ally_mask` (`+0x6929`) ours 0x15,
    /// theirs 0x17. Built (item 4529, second: `gain_tech@006dcb60`'s
    /// `ALLY_LOS` arm, Mathematics ORs every seated ally into the stored
    /// mask): leaders 5427 -> 5486 (`500 3 encrypt`), units and builds
    /// 5461 and 5487 -> 6000, ammo 5969 -> 6000, guys 6017, groups and
    /// world 6022, cities 6151; goods 6395 -> 6281, a named fall past
    /// units' 6000. Built (item 4533: a capture's spoils skip Knowledge,
    /// the person's on 5485 Metal): leaders 5486 -> 5891 (`462 3
    /// oil_patches`). Built (item 4533, second: the capacity doubles from
    /// ten): leaders 5891 -> 6005 (`1 0 who..ally_mask`). Built (item
    /// 4543: the aged-up arm reads both leaders' `ages`, `3107 37
    /// attrition` 48|24 on 6108): units 6108 -> 6114 (`1665 11`, its
    /// orders), deaths 6190 -> 6186. Built (item 4549: the bell's
    /// building arm takes `2/11` and `2/14` off `2/2001`'s gatherer chain
    /// on 6113): units 6114 and builds 6114 -> past 6400 and 6162 (`2515
    /// 2025 ever_seen` 7|5); every other line past the window. Built
    /// (item 4552: the unstarted scan takes the owner's stored ally
    /// byte, seat 2's 5): builds 6162 -> past the window; no kept line
    /// parts below 6400.
    const SAMPLE_RANGES_1772: &[(&str, i64, usize, usize, usize, &str)] = &[];

    /// run1786's last frame, ours: the record holds trace frames −1 to 7999.
    const SAMPLE_RANGES_1786_WINDOW: i64 = 8000;

    /// run1786's mask: every line but walls (`RNG_MASK=0xffbu`).
    const SAMPLE_RANGES_1786_KEPT: u16 = 0xffb;

    /// Per kept line that parts on run1786's window, its first parting by
    /// range (item 4555). Minted at eleven rows, each on the frame the
    /// hook's sums part (`SAMPLE_LINES`): units 6669, groups 6928, builds,
    /// guys, leaders, cities and world 7021, ammo 7023, deaths 7085, goods
    /// 7423, items 7544. Built (item 4555: `check_build_order`'s kill is
    /// the front's, `3/55` keeps one `BUILD_AT 2042`): units 6669 -> 6928
    /// (`2909 15 collide_frame..play` `dest_angle`). Built (item 4557:
    /// `come_out`'s enemy arm, the Bushi `3/15`, `3/16`, `3/28` out of
    /// `3/2041` attack-move to the point on `2/2010` itself): units 6928 ->
    /// 7021 (`28 1 who..y_internal`), groups 6928 -> 7097 (`0 0
    /// id..march`).
    const SAMPLE_RANGES_1786: &[(&str, i64, usize, usize, usize, &str)] = &[
        (
            "units",
            7021,
            4579,
            4584,
            4359,
            "28 1 who..y_internal: z 131|115, x 23208|24600, y 42408|43032",
        ),
        (
            "builds",
            7021,
            4243,
            4243,
            6,
            "7 2000 myhits..launch_frames: inside_down 3|-1",
        ),
        (
            "ammo",
            7023,
            9,
            9,
            1,
            "1 0 rolling..start_roll_angle: +0x7 of 99|99: ce150000|fa140000",
        ),
        (
            "deaths",
            7085,
            30,
            30,
            1,
            "3 1 first_frame..unit_crew: +0x4 of 71|71: 11000000|12000000",
        ),
        (
            "groups",
            7097,
            628,
            628,
            1,
            "0 0 id..march: ox 18144|15072, oy 21984|24288, o_angle -1191706624|-1295384576",
        ),
        (
            "guys",
            7021,
            1477,
            1477,
            12,
            "15 1 guy: x 23208|24600, y 42408|43032, z 123|115, des_x 23208|24600, des_y 42408|43032, last_x 23208|24600, last_y 42408|43032, last_z 123|115, cur_time 7|8, last_time 6|7",
        ),
        (
            "leaders",
            7021,
            675,
            675,
            2,
            "0 0 flags: +0x3 of 8|8: 00000000|02000000",
        ),
        (
            "cities",
            7021,
            57,
            57,
            1,
            "0 1 city_flags: +0x0 of 2|2: 594e|194e",
        ),
        (
            "items",
            7544,
            20,
            20,
            1,
            "0 16 ever_seen: +0x0 of 1|1: 08|00",
        ),
        (
            "goods",
            7423,
            265,
            265,
            1,
            "190 38 ever_seen: +0x0 of 1|1: 08|0c",
        ),
        (
            "world",
            7021,
            7733,
            7705,
            108,
            "3293 3269 cell: +0x8 of 21|21: ffff0000|08000000",
        ),
    ];
}
