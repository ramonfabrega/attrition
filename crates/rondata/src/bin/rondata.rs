//! Surveys an installed copy of Rise of Nations and reports what its data
//! layer actually contains.
//!
//! This is the front end of phase 0, and it doubles as a standing check on
//! `docs/FORMATS.md`. Every structural claim we make about the format is
//! re-tested here against the user's own files, so a claim that stops being
//! true shows up as a failed check rather than as a mystery three phases later.
//!
//! ```sh
//! cargo run -p rondata -- /path/to/Rise\ of\ Nations
//! ```

use rondata::{Install, Resource, Scalar, Table};
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let Some(root) = args.next() else {
        eprintln!(
            "usage: rondata <install-root> [--gamelog <Logs/gamelog.txt>] [--types <dump>] [--recgame <file.rcx>]"
        );
        eprintln!();
        eprintln!("The directory holding riseofnations.exe. No game data is");
        eprintln!("copied anywhere; this only reads.");
        eprintln!();
        eprintln!("--gamelog  a start-of-game dump written by the original with");
        eprintln!("           InitialDump=1 (docs/ORACLE.md): its CONSTANTS block");
        eprintln!("           is checked against rules.xml and sim::Tuning::RON.");
        eprintln!("--diff [N] with --gamelog: load the tables into the sim, stand");
        eprintln!("           it up from the dump's initial state, and step it");
        eprintln!("           against the logged frames (at most N), reporting");
        eprintln!("           ticks before divergence.");
        eprintln!("--sibling  with --diff: another dump of the same lobby and seed");
        eprintln!("           whose setup checksum trace (check_all_level=14) or");
        eprintln!("           DUMP_ALL height table this one lacks; repeatable.");
        eprintln!("--types    a DUMP_ALL=1 start-of-game dump (docs/ORACLE.md): its");
        eprintln!("           UNITTYPE blocks and COMBATTABLE are checked against the");
        eprintln!("           loader's Kinds and the combat table it builds.");
        eprintln!("--trace    a rontrace.log from tools/trace (docs/ORACLE.md): its");
        eprintln!("           per-frame draw sites are folded and printed beside the");
        eprintln!("           simulation's own, which is what --diff's `by phase` note");
        eprintln!("           has to be lined up against. Addresses are bare: naming");
        eprintln!("           them is tools/trace/report.py's job.");
        eprintln!("--recgame  a .rcx recorded game (docs/RECGAME.md): the header,");
        eprintln!("           lobby and command-package stream are parsed and");
        eprintln!("           summarised.");
        return ExitCode::from(2);
    };
    let mut gamelog: Option<String> = None;
    let mut siblings: Vec<String> = Vec::new();
    let mut types: Option<String> = None;
    let mut recgame: Option<String> = None;
    let mut trace: Option<String> = None;
    let mut diff: Option<Option<usize>> = None;
    while let Some(a) = args.next() {
        match a.as_str() {
            "--gamelog" => gamelog = args.next(),
            "--sibling" => siblings.extend(args.next()),
            "--types" => types = args.next(),
            "--recgame" => recgame = args.next(),
            "--trace" => trace = args.next(),
            "--diff" => {
                diff = Some(None);
                if let Some(n) = args.next() {
                    match n.parse() {
                        Ok(n) => diff = Some(Some(n)),
                        Err(_) => {
                            eprintln!("--diff takes an optional frame count, not {n:?}");
                            return ExitCode::from(2);
                        }
                    }
                }
            }
            other => {
                eprintln!("unknown argument {other}");
                return ExitCode::from(2);
            }
        }
    }

    let install = Install::new(&root);
    if !install.looks_valid() {
        eprintln!("{root}: no Data/rules.xml here — is this the install root?");
        return ExitCode::from(2);
    }

    let result = survey(&install)
        .and_then(|f| match &gamelog {
            Some(path) => {
                let mut f = f + gamelog_report(&install, path)?;
                if let Some(limit) = diff {
                    f += diff_report(&install, path, limit, recgame.as_deref(), &siblings)?;
                }
                Ok(f)
            }
            None => Ok(f),
        })
        .and_then(|f| match &types {
            Some(path) => Ok(f + types_report(&install, path)?),
            None => Ok(f),
        })
        .and_then(|f| match &recgame {
            Some(path) => Ok(f + recgame_report(&install, path)?),
            None => Ok(f),
        })
        .map(|f| match &trace {
            Some(path) => f + trace_report(path),
            None => f,
        });
    match result {
        Ok(0) => ExitCode::SUCCESS,
        Ok(failures) => {
            eprintln!("\n{failures} structural check(s) failed.");
            ExitCode::FAILURE
        }
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

/// Folds a `rontrace.log`'s draws by site, frame by frame — the original's
/// side of `--diff`'s `rng: frame N: ours by phase` note
/// (`docs/ORACLE.md`, "The draw-site trace and function coverage").
///
/// Sites are bare addresses. Naming them needs the Ghidra export's
/// `INDEX.tsv`, which never enters this repo; `tools/trace/report.py` is
/// where a name comes from, and this is for lining the two folds up
/// without leaving Rust.
fn trace_report(path: &str) -> usize {
    println!("\ntrace");
    let t = match rondata::trace::Trace::read(std::path::Path::new(path)) {
        Ok(Some(t)) => t,
        Ok(None) => {
            println!("  [FAIL] {path}: not a rontrace.log (no RONT header)");
            return 1;
        }
        Err(e) => {
            println!("  [FAIL] {path}: {e}");
            return 1;
        }
    };
    let sync = t.draws.iter().filter(|d| d.sync()).count();
    println!(
        "  base {:#010x}, {} draws of which {sync} on game_random, {} frames",
        t.base,
        t.draws.len(),
        t.frames.len()
    );
    // The sim-frames only. The setup path is the map generator and is tens
    // of thousands of draws; `report.py … sites setup` is where that is
    // read, and it is not what a frame's fold is for.
    let setup = t.draws.iter().filter(|d| d.frame < 0 && d.sync()).count();
    if setup > 0 {
        println!("  setup: {setup} draws, not folded (the map generator)");
    }
    for (f, _) in &t.frames {
        let n = t.frame_draws(*f).len();
        if n == 0 {
            continue;
        }
        // A frame with the renderer's sites in it can run to hundreds of
        // runs; the head is what lines up against `by phase`.
        let fold = t.site_fold(*f);
        let runs: Vec<&str> = fold.split(", ").collect();
        let head = runs
            .iter()
            .take(RUNS)
            .copied()
            .collect::<Vec<_>>()
            .join(", ");
        let tail = if runs.len() > RUNS {
            format!(", … {} more runs", runs.len() - RUNS)
        } else {
            String::new()
        };
        println!("  frame {f}: {n} draws — {head}{tail}");
    }
    0
}

/// How many runs of one site a frame's fold prints before it truncates.
const RUNS: usize = 60;

/// Loads the tables, stands the simulation up from the dump and diffs it
/// against the logged frames.
fn diff_report(
    install: &Install,
    path: &str,
    limit: Option<usize>,
    recgame: Option<&str>,
    siblings: &[String],
) -> Result<usize, rondata::Error> {
    use rondata::gamelog::{Initial, Log};

    let read = |p: &str| {
        std::fs::read_to_string(p).map_err(|source| rondata::Error::Io {
            path: p.to_string(),
            source,
        })
    };
    // The siblings' texts must outlive the initial state borrowed from them.
    let sibling_texts: Vec<String> = siblings.iter().map(|p| read(p)).collect::<Result<_, _>>()?;
    let text = read(path)?;
    let log = Log::parse(&text);
    let sibling_logs: Vec<Log> = sibling_texts.iter().map(|t| Log::parse(t)).collect();
    let sibling_inits: Vec<Initial> = sibling_logs.iter().filter_map(|l| l.initial()).collect();
    let sibling_refs: Vec<&Initial> = sibling_inits.iter().collect();
    for (p, i) in siblings.iter().zip(&sibling_inits) {
        println!(
            "  sibling {p}: {} checksum records, {} heights",
            i.checksums.len(),
            i.heights.len()
        );
    }
    let loaded = rondata::load::load(install)?;
    let mut failures = 0;
    println!("\ndiff");
    failures += check(
        "the shipped tables load with every name resolved",
        loaded.warnings.is_empty(),
        &if loaded.warnings.is_empty() {
            format!(
                "{} units, {} buildings, {} techs, {} goods; tree of {}",
                loaded.unit_types.len(),
                loaded.build_types.len(),
                loaded.tech_names.len(),
                loaded.good_names.len(),
                loaded.tree.types.len()
            )
        } else {
            join(loaded.warnings.iter().cloned())
        },
    );
    // The order stream, when a recording of the *same run* is given. Pairing
    // is the caller's claim and a wrong pairing is worth catching loudly, so
    // the frame counts are checked against each other before anything is fed
    // in (`docs/DATALAYER.md`).
    let mut stream = match recgame {
        Some(rc) => {
            let data = rondata::recgame::decompress(rc)?;
            let rec = rondata::recgame::parse(&data, rc)?;
            let s = rondata::input::Stream::new(&rec);
            println!(
                "  order stream: {} packages, {} input commands, last on frame {}",
                rec.packages.len(),
                s.len(),
                s.last_frame()
            );
            let logged = log.frame_states().len();
            failures += check(
                "the recording and the dump describe the same run",
                rec.packages.len() == logged,
                &format!("{} packages, {logged} logged frames", rec.packages.len()),
            );
            Some(s)
        }
        None => None,
    };
    let Some(report) = rondata::diff::run_traced(
        &loaded,
        &log,
        sim::Tuning::RON,
        limit,
        stream.as_mut(),
        &sibling_refs,
    ) else {
        println!("  no BEGIN GAME in the log; nothing to diff");
        return Ok(failures);
    };
    // The units the original has that the simulation never stood up or
    // trained, by name, with the first frame each appears on.
    let mut missing: Vec<((i64, i64), i64, usize)> = Vec::new();
    for f in &report.frames {
        for u in &f.unlinked_units {
            match missing.iter_mut().find(|(k, _, _)| k == u) {
                Some((_, _, n)) => *n += 1,
                None => missing.push((*u, f.frame, 1)),
            }
        }
    }
    for ((who, o), first, n) in &missing {
        println!("  unlinked: unit {who}/{o} from frame {first}, {n} frame(s)");
    }
    if recgame.is_some() {
        println!(
            "  the stream drove {} order(s); {} command(s) carried but not acted on",
            report.applied.orders,
            report.applied.skipped_total()
        );
        for ((name, why), n) in &report.applied.skipped {
            println!("    {n:5} {name}: {why}");
        }
    }
    for n in &report.notes {
        println!("  note: {n}");
    }
    let compared: usize = report.frames.iter().map(|f| f.compared).sum();
    let unlinked: usize = report.frames.iter().map(|f| f.unlinked).sum();
    println!(
        "  {} frames stepped, {} unit-frames compared, {} unit-frames the sim has no unit for",
        report.frames.len(),
        compared,
        unlinked
    );
    println!(
        "  ticks before divergence: {}",
        report.ticks_before_divergence()
    );
    // Derive-then-read: the starting orders our §9.3 rule produced against the
    // ones the original actually issued. Positions cannot show this.
    if !report.orders_seen() {
        // Nothing to compare against: a dump below `UNITS=3` writes no order
        // list at all, and calling the derivation wrong for that would be
        // blaming the simulation for the logger's detail threshold.
        println!(
            "  the dump carries no order lists (below UNITS=3), so the start-of-game rule and the order diff are both unchecked here"
        );
    } else if let Some(built) = rondata::diff::build_for_check(&loaded, &log, sim::Tuning::RON) {
        let checks = rondata::diff::check_start_orders(&built, &log);
        let held: Vec<_> = checks
            .iter()
            .filter(|c| c.theirs.is_some() || c.ours.is_some())
            .collect();
        let bad: Vec<_> = held.iter().filter(|c| !c.agrees()).collect();
        failures += check(
            "every starting citizen's derived GATHER target matches the one the original issued",
            bad.is_empty() && !held.is_empty(),
            &if held.is_empty() {
                "no unit in the first logged frame holds a gather order — is the dump below UNITS=3?".to_string()
            } else if bad.is_empty() {
                format!(
                    "{} citizens, derived from §9.3 without reading the log: {}",
                    held.len(),
                    held.iter()
                        .map(|c| format!("{}/{}→{}", c.who, c.o, c.ours.unwrap_or(-1)))
                        .collect::<Vec<_>>()
                        .join(" ")
                )
            } else {
                bad.iter()
                    .map(|c| {
                        format!(
                            "who {} o {}: we derived {:?}, the log has {:?} (order kind {})",
                            c.who, c.o, c.ours, c.theirs, c.their_kind
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("; ")
            },
        );
    }
    let by_unit = report.first_divergence_by_unit();
    if by_unit.is_empty() {
        println!("  no unit ever diverged");
    } else {
        let cells: Vec<String> = by_unit
            .iter()
            .map(|(w, o, f)| format!("{w}/{o}@{f}"))
            .collect();
        println!(
            "  units that diverge, as who/o@frame — everything else tracked to the end: {}",
            cells.join(" ")
        );
    }
    // The order lists, frame by frame — the intent diff, which sees what a
    // position diff cannot. `UNITS=3` or nothing.
    if report.orders_seen() {
        let seen: usize = report.frames.iter().map(|f| f.order_compared).sum();
        let orders: usize = report.frames.iter().map(|f| f.order_only().count()).sum();
        let paths: usize = report.frames.iter().map(|f| f.path_only().count()).sum();
        println!(
            "  order lists: {seen} unit-frames compared, {orders} order disagreements, {paths} path-stack disagreements"
        );
        println!(
            "  ticks before an order diverges: {}",
            report.order_ticks_before_divergence()
        );
        let mut tally: std::collections::BTreeMap<&str, usize> = std::collections::BTreeMap::new();
        for f in &report.frames {
            for d in &f.order_diverged {
                *tally.entry(d.what.name()).or_default() += 1;
            }
        }
        if !tally.is_empty() {
            let cells: Vec<String> = tally.iter().map(|(k, n)| format!("{k} {n}")).collect();
            println!("  by kind: {}", cells.join(", "));
        }
        let by_order = report.order_divergence_by_unit();
        if by_order.is_empty() {
            println!("  every unit's order list matched on every frame");
        } else {
            for (w, o, f, what) in &by_order {
                println!("  who {w} o {o}: first order disagreement at frame {f} — {what:?}");
            }
        }
    }
    for (who, first) in &report.first_divergence {
        match first {
            Some(f) => {
                let d = report
                    .frames
                    .iter()
                    .find(|fr| fr.frame == *f)
                    .and_then(|fr| fr.diverged.iter().find(|d| d.who == *who));
                match d {
                    Some(d) => println!(
                        "  player {who}: first divergence at frame {f} — unit o {} ours ({}, {}) theirs ({}, {})",
                        d.o, d.ours.x, d.ours.y, d.theirs.x, d.theirs.y
                    ),
                    None => println!("  player {who}: first divergence at frame {f}"),
                }
            }
            None => println!(
                "  player {who}: no divergence over {} frames",
                report.frames.len()
            ),
        }
    }
    if let Some(last) = report.frames.last() {
        let s: Vec<String> = last
            .scores
            .iter()
            .map(|(w, s)| format!("{w}: {s}"))
            .collect();
        println!(
            "  logged scores at frame {} (not matched yet): {}",
            last.frame,
            s.join(", ")
        );
    }
    Ok(failures)
}

/// Reads a start-of-game dump and checks the constants it carries.
fn gamelog_report(install: &Install, path: &str) -> Result<usize, rondata::Error> {
    use rondata::dump::{self, Loaded};
    use rondata::gamelog::Log;

    let text = std::fs::read_to_string(path).map_err(|source| rondata::Error::Io {
        path: path.to_string(),
        source,
    })?;
    let log = Log::parse(&text);
    let Some(init) = log.initial() else {
        return Err(rondata::Error::Missing {
            path: path.to_string(),
            what: "BEGIN GAME".into(),
        });
    };
    let rules = install.rules()?;
    let mut failures = 0;

    println!("\ngamelog: {path}");
    println!("  {:<24} {:>5}", "frames", log.frames().len());
    println!("  {:<24} {:>5}", "CONSTANTS keys", init.constants.len());
    println!("  {:<24} {:>5}", "units", init.units.len());
    println!("  {:<24} {:>5}", "buildings", init.builds.len());
    println!("  {:<24} {:>5}", "leaders", init.leaders.len());
    println!("  {:<24} {:>5}", "cities", init.cities.len());
    if let Some((_, seed)) = init.game_info.iter().find(|(k, _)| *k == "(int)seed") {
        println!("  {:<24} {:>5}", "seed", seed);
    }

    let (xml_only, dump_only) = dump::unmatched(&rules, &init.constants);
    println!(
        "\nconstants: {} tags matched by lowercased name; {} in the file only; {} in the dump only",
        rules.constants.len() - xml_only.len(),
        xml_only.len(),
        dump_only.len()
    );
    if !xml_only.is_empty() {
        println!("  file only: {}", xml_only.join(", "));
    }
    if !dump_only.is_empty() {
        println!("  dump only: {}", dump_only.join(", "));
    }

    let classified = dump::classify(&rules, &init.constants);
    let count = |f: &dyn Fn(&Loaded) -> bool| classified.iter().filter(|c| f(&c.loaded)).count();
    println!("\nhow the engine loads them, from the dump");
    println!("  {:<24} {:>5}", "plain", count(&|l| *l == Loaded::Plain));
    println!(
        "  {:<24} {:>5}",
        "x256 (8.8)",
        count(&|l| *l == Loaded::Scaled256)
    );
    println!(
        "  {:<24} {:>5}",
        "x100",
        count(&|l| *l == Loaded::Scaled100)
    );
    println!(
        "  {:<24} {:>5}",
        "x192",
        count(&|l| *l == Loaded::Scaled192)
    );
    println!(
        "  {:<24} {:>5}",
        "x48 (UCoord)",
        count(&|l| *l == Loaded::Scaled48)
    );
    println!(
        "  {:<24} {:>5}",
        "x10 (attack)",
        count(&|l| *l == Loaded::Scaled10)
    );
    println!(
        "  {:<24} {:>5}",
        "ambiguous",
        count(&|l| matches!(l, Loaded::Ambiguous(_)))
    );
    println!(
        "  {:<24} {:>5}",
        "unexplained",
        count(&|l| *l == Loaded::Unexplained)
    );
    for (title, want) in [
        ("x256 (8.8)", Loaded::Scaled256),
        ("x100", Loaded::Scaled100),
        ("x192", Loaded::Scaled192),
        ("x48 (UCoord)", Loaded::Scaled48),
        ("x10 (attack)", Loaded::Scaled10),
        ("unexplained", Loaded::Unexplained),
    ] {
        let names: Vec<String> = classified
            .iter()
            .filter(|c| c.loaded == want)
            .map(|c| {
                format!(
                    "{} ({} -> {})",
                    c.name,
                    join(c.written.iter().map(|s| describe(*s))),
                    join(c.dumped.iter().map(i64::to_string))
                )
            })
            .collect();
        if !names.is_empty() {
            println!("  {title}:");
            for n in names {
                println!("    {n}");
            }
        }
    }

    println!("\ntuning against the dump");
    let report = dump::tuning_drift(&init.constants);
    if !report.missing.is_empty() {
        println!(
            "  not in the dump (Constants::log_data omits them): {}",
            report.missing.join(", ")
        );
    }
    let drift = report.mismatched;
    failures += check(
        "every sim::Tuning::RON slot the dump carries equals the loaded field",
        drift.is_empty(),
        &if drift.is_empty() {
            format!(
                "{} slots",
                sim::tuning::Tuning::ron_slots().len() - report.missing.len()
            )
        } else {
            join(drift.iter().map(|d| {
                format!(
                    "{}: ours {} theirs {}",
                    d.name,
                    d.ours,
                    d.theirs.as_deref().unwrap_or("missing")
                )
            }))
        },
    );
    Ok(failures)
}

/// `--types`: the program's own loaded types and composed combat table,
/// against the loader's `Kind`s and the table it builds from them
/// (`docs/COMBAT.md` §15). The comparison is [`rondata::typesdump::compare`],
/// which the install-gated test in that module also runs against run3's
/// dump; this prints its report. `RONDATA_VERBOSE=1` lists every offender
/// instead of the first eight.
fn types_report(install: &Install, path: &str) -> Result<usize, rondata::Error> {
    use rondata::typesdump::{self, Line};

    let dump = typesdump::read(path)?;
    let loaded = rondata::load::load(install)?;
    let verbose = std::env::var_os("RONDATA_VERBOSE").is_some();
    println!("\ntypes dump: {path}");
    let mut failures = 0;
    for line in &typesdump::compare(&loaded, &dump, verbose).lines {
        match line {
            Line::Note(s) => println!("{s}"),
            Line::Check(c) => failures += check(&c.what, c.ok, &c.detail),
        }
    }
    Ok(failures)
}

fn survey(install: &Install) -> Result<usize, rondata::Error> {
    let rules = install.rules()?;
    let units = install.units()?;
    let buildings = install.buildings()?;
    let techs = install.techs()?;

    println!("install: {}", install.root().display());
    match install.pdb() {
        Some(p) => println!("symbols: {}", p.display()),
        None => println!("symbols: none (no sbl/rise.pdb)"),
    }

    println!("\ntables");
    println!("  {:<24} {:>5}", "CONSTANTS", rules.constants.len());
    println!("  {:<24} {:>5}", "TECHBONUSES", rules.tech_bonuses.len());
    println!("  {:<24} {:>5}", "FORMATIONS", rules.formations.len());
    println!("  {:<24} {:>5}", "LANDS", rules.lands.len());
    println!("  {:<24} {:>5}", "TRIBES", rules.tribes.len());
    println!(
        "  {:<24} {:>5}",
        "CATEGORIES (groups)",
        rules.categories.len()
    );
    println!("  {:<24} {:>5}", "UNIT", units.len());
    println!("  {:<24} {:>5}", "BUILDING", buildings.len());
    println!("  {:<24} {:>5}", "TECH", techs.len());

    let mut failures = 0;
    println!("\nstructural checks");

    // The claim that forces index-keyed loading. If this ever comes back
    // empty, name-keying would be safe and decision 9 deserves revisiting.
    let dups = rules.constants.duplicate_tags();
    failures += check(
        "constant names are not unique (so index is identity)",
        !dups.is_empty(),
        &format!(
            "{} duplicated: {}",
            dups.len(),
            join(dups.iter().map(|(t, n)| format!("{t}×{n}")))
        ),
    );

    // Every constant must carry a parseable leading number. A miss means the
    // scalar grammar is incomplete.
    let mut unparsed = Vec::new();
    for (i, rec) in rules.constants.iter() {
        let Some(f) = rec.fields.first() else {
            continue;
        };
        if f.attrs.iter().any(|(k, _)| k.starts_with("entry")) {
            for (n, e) in f.entries().iter().enumerate() {
                if Scalar::parse(e).is_none() {
                    unparsed.push(format!("[{i}] {}.entry{n}={e:?}", rec.tag));
                }
            }
        } else if let Some(t) = f.scalar_text()
            && Scalar::parse(t).is_none()
        {
            unparsed.push(format!("[{i}] {}={t:?}", rec.tag));
        }
    }
    failures += check(
        "every constant starts with a number",
        unparsed.is_empty(),
        &if unparsed.is_empty() {
            "all values parsed".into()
        } else {
            format!(
                "{} unparsed: {}",
                unparsed.len(),
                join(unparsed.iter().take(5).cloned())
            )
        },
    );

    // Costs must decode against the six-resource alphabet.
    let mut bad_costs = Vec::new();
    for (i, rec) in units.iter() {
        for tag in ["COST", "SUPPORT"] {
            if let Some(t) = rec.text(tag)
                && rondata::Cost::parse(t).is_none()
            {
                bad_costs.push(format!("[{i}] {tag}={t:?}"));
            }
        }
    }
    failures += check(
        "every unit cost decodes to known resources",
        bad_costs.is_empty(),
        &if bad_costs.is_empty() {
            "all costs parsed".into()
        } else {
            format!(
                "{} bad: {}",
                bad_costs.len(),
                join(bad_costs.iter().take(5).cloned())
            )
        },
    );

    // TRIBE_MASK is as wide as the nation roster. If the widths ever diverge
    // the mask decoding is wrong and every nation-specific unit is misassigned.
    let n_tribes = rules.tribes.len();
    let widths: Vec<usize> = units
        .records
        .iter()
        .filter_map(|r| r.text("TRIBE_MASK").map(str::len))
        .collect();
    let uniform = widths.iter().all(|w| *w == n_tribes);
    failures += check(
        "TRIBE_MASK width matches the nation count",
        uniform && !widths.is_empty(),
        &format!("{n_tribes} nations, mask widths {:?}", uniq(&widths)),
    );

    // The mask reading, re-derived from the data rather than asserted.
    let keys = rules.tribe_keys();
    let mut named = Vec::new();
    for probe in ["Samurai", "Cossack"] {
        if let Some(r) = units.records.iter().find(|r| r.text("NAME") == Some(probe))
            && let Some(m) = r.text("TRIBE_MASK")
        {
            let who: Vec<&str> = rondata::tribe_mask(m)
                .iter()
                .filter_map(|i| keys.get(*i).map(String::as_str))
                .collect();
            named.push(format!(
                "{probe}→{}",
                join(who.iter().map(|s| (*s).to_string()))
            ));
        }
    }
    let expected = named.iter().any(|s| s.contains("japanese"))
        && named.iter().any(|s| s.contains("russians"));
    failures += check(
        "TRIBE_MASK decodes MSB-first",
        expected,
        &join(named.iter().cloned()),
    );

    // `SUPPORT` is the ramping cost, and the engine keeps it in two ordered
    // slots rather than a six-slot array: `ObjectType::load_support` walks the
    // pairs in order, skips zero amounts, and stops after two. So a field can
    // name the same resource twice, and if one does, that resource ramps
    // twice. Both halves of that are checked here.
    let mut overlong = Vec::new();
    let mut duplicated = Vec::new();
    for (i, rec) in units.iter() {
        let Some(c) = rec.text("SUPPORT").and_then(rondata::Cost::parse) else {
            continue;
        };
        let written = c.0.iter().filter(|(_, n)| *n != 0).count();
        if written > 2 {
            overlong.push(format!("[{i}] {}", rec.text("NAME").unwrap_or("?")));
        }
        let slots = c.support_slots();
        if slots.len() == 2 && slots[0].0 == slots[1].0 {
            duplicated.push(format!(
                "[{i}] {} {}×2",
                rec.text("NAME").unwrap_or("?"),
                slots[0].0.letter()
            ));
        }
    }
    failures += check(
        "no unit's SUPPORT names more than the two slots the engine keeps",
        overlong.is_empty(),
        &if overlong.is_empty() {
            format!(
                "{} with a duplicated resource, which therefore ramps twice: {}",
                duplicated.len(),
                join(duplicated.iter().take(3).cloned())
            )
        } else {
            format!(
                "{} overlong: {}",
                overlong.len(),
                join(overlong.iter().take(5).cloned())
            )
        },
    );

    // Where a cost written in an unavailable resource is charged instead —
    // `docs/COSTS.md`'s headline, and why early units want timber and later
    // ones want metal.
    //
    // The columns are `UNDISC_COST_GOOD`/`UNDISC_COST_RATE` and their obsolete
    // pair. They are what `TypeData::get_cost` reads, at `GoodTypeData +0x2b4`
    // and `+0x2c8`. This check used to read the `*_SUPPORT_*` columns four
    // fields further on, which are a different table and are read by nothing
    // found — so it agreed with a wrong `Redirects::RON` and proved nothing.
    let resources = install.resources()?;
    let mut redirect_drift = Vec::new();
    for (i, r) in Resource::ALL.iter().enumerate() {
        let Some(rec) = resources.records.get(i) else {
            redirect_drift.push(format!("{} is missing", name_of(*r)));
            continue;
        };
        let tables = [
            (
                "undiscovered",
                "UNDISC",
                sim::cost::Redirects::RON.undiscovered[i],
            ),
            ("obsolete", "OBS", sim::cost::Redirects::RON.obsolete[i]),
        ];
        for (which, prefix, ours) in tables {
            let theirs_good = rec.text(&format!("{prefix}_COST_GOOD")).unwrap_or("");
            let theirs_rate = rec
                .text(&format!("{prefix}_COST_RATE"))
                .and_then(Scalar::parse)
                .map(|s| s.to_fx().raw() / 256);
            let good_agrees = theirs_good.eq_ignore_ascii_case(sim_name_of(ours.good));
            if !good_agrees || theirs_rate != Some(ours.rate) {
                redirect_drift.push(format!(
                    "{} {which}: we say {} at {}, install says {theirs_good} at {:?}",
                    name_of(*r),
                    sim_name_of(ours.good),
                    ours.rate,
                    theirs_rate
                ));
            }
        }
    }
    failures += check(
        "the undiscovered-resource redirect matches this install",
        redirect_drift.is_empty(),
        &if redirect_drift.is_empty() {
            "metal is charged as timber at 5/4, knowledge as food at 3/2, oil as metal at 3/2"
                .into()
        } else {
            join(redirect_drift.iter().cloned())
        },
    );

    // Every number the attrition simulation is built on, re-read from the
    // user's own file. A drift here means the sim is running on values this
    // install does not have.
    let drift = rondata::drift(&rules);
    failures += check(
        "the simulation's tuning table matches this install",
        drift.is_empty(),
        &if drift.is_empty() {
            format!("{} constants agree", sim::Tuning::ron_slots().len())
        } else {
            join(drift.iter().map(|d| {
                format!(
                    "{}: we say {}, install says {}",
                    d.name,
                    d.ours,
                    d.theirs.as_deref().unwrap_or("(absent)")
                )
            }))
        },
    );

    // The tech tree's shape, re-derived from the tables the way `docs/TECH.md`
    // reads it out of the engine: 85 technologies in blocks, the four library
    // lines seven deep with `AGE` as the level, the governments in pairs, and a
    // prerequisite vocabulary of tech names plus two words. The engine's
    // `Types::tech_key` matches names case-insensitively, and the shipped file
    // needs it to: it writes both `none` and `None`, and one `Information age`.
    println!("\nthe tech tree");
    let tech_names: Vec<String> = techs
        .records
        .iter()
        .map(|r| r.text("NAME").unwrap_or("").to_lowercase())
        .collect();
    failures += check(
        "85 technologies: 7 ages, 28 epochs, 4 finals, 40 building techs, 6 governments",
        techs.len() == 85,
        &format!("{} TECH records", techs.len()),
    );
    let age_of = |i: usize| -> Option<i32> {
        techs
            .get(i)
            .and_then(|r| r.text("AGE"))
            .and_then(Scalar::parse)
            .map(Scalar::to_int)
    };
    let ages_in_order = (0..7).all(|i| age_of(i) == Some(i as i32));
    failures += check(
        "the seven ages come first, AGE 0 through 6",
        ages_in_order,
        &join((0..7).map(|i| {
            format!(
                "{}={:?}",
                tech_names.get(i).cloned().unwrap_or_default(),
                age_of(i)
            )
        })),
    );
    // Rows 7..35 are the epochs, four lines of seven; `AGE` is the level
    // within the line, which is what `cat` and `epoch[cat]` are built from.
    let epochs_by_level = (7..35).all(|i| age_of(i) == Some(((i - 7) % 7) as i32));
    failures += check(
        "the 28 epochs follow in four lines of seven, AGE equal to the level",
        epochs_by_level,
        &format!(
            "lines start at {}, {}, {}, {}",
            tech_names.get(7).cloned().unwrap_or_default(),
            tech_names.get(14).cloned().unwrap_or_default(),
            tech_names.get(21).cloned().unwrap_or_default(),
            tech_names.get(28).cloned().unwrap_or_default()
        ),
    );
    // The governments close the table, researched at the Senate, and each
    // tier's pair shares its age prerequisite.
    let senate_govs = (79..85).all(|i| {
        techs
            .get(i)
            .and_then(|r| r.text("WHERE"))
            .is_some_and(|w| w.eq_ignore_ascii_case("Senate"))
    });
    let paired = [(79, 80), (81, 82), (83, 84)].iter().all(|&(a, b)| {
        let p = |i: usize| {
            techs
                .get(i)
                .and_then(|r| r.text("PREQ1"))
                .map(str::to_lowercase)
        };
        p(a) == p(b)
    });
    failures += check(
        "the last six are the governments, at the Senate, in three pairs",
        senate_govs && paired,
        &join((79..85).map(|i| tech_names.get(i).cloned().unwrap_or_default())),
    );
    // Every prerequisite column across the four tables names a tech, `none`
    // or `disable` — the three answers `tech_key` gives.
    let mut bad_preqs = Vec::new();
    let tables: [(&str, &Table, &[&str]); 4] = [
        ("UNIT", &units, &["PREQ0", "PREQ1"]),
        (
            "BUILDING",
            &buildings,
            &["PREQ0", "PREQ1", "PREQ2", "OBSOLETE"],
        ),
        ("TECH", &techs, &["PREQ0", "PREQ1", "PREQ2"]),
        ("RESOURCE", &resources, &["PREQ0", "PREQ1", "OBS"]),
    ];
    for (what, table, cols) in tables {
        for (i, rec) in table.iter() {
            for col in cols {
                if let Some(v) = rec.text(col) {
                    let v = v.trim().to_lowercase();
                    if !(v == "none" || v == "disable" || tech_names.contains(&v)) {
                        bad_preqs.push(format!("{what}[{i}] {col}={v:?}"));
                    }
                }
            }
        }
    }
    failures += check(
        "every prerequisite is a tech name, `none` or `disable`",
        bad_preqs.is_empty(),
        &if bad_preqs.is_empty() {
            "all four tables resolve".into()
        } else {
            join(bad_preqs.iter().take(6).cloned())
        },
    );
    // The unit flag letters the cascades key on: `h` free with its
    // prerequisite, `j` a jump-chain upgrade. The Citizen and the Hoplites
    // are free; the Phalanx is not, and jumps.
    let flags_of = |name: &str| -> String {
        units
            .records
            .iter()
            .find(|r| r.text("NAME") == Some(name))
            .and_then(|r| r.text("FLAGS"))
            .unwrap_or("")
            .to_string()
    };
    let (citizen, hoplites, phalanx) = (
        flags_of("Citizen"),
        flags_of("Hoplites"),
        flags_of("Phalanx"),
    );
    failures += check(
        "the Citizen and Hoplites carry flag `h`, the Phalanx `j` and not `h`",
        citizen.contains('h')
            && hoplites.contains('h')
            && phalanx.contains('j')
            && !phalanx.contains('h'),
        &format!("Citizen={citizen:?} Hoplites={hoplites:?} Phalanx={phalanx:?}"),
    );

    // ---- combat: the columns and the table's XML half (`docs/COMBAT.md`) ----
    //
    // `UnitType::init` reads `OBJ_MASK` as a string of letters, `RANGE` as a
    // `min-max` pair, and `ATTACK` as a number it stores ×10; `balance.xml` is
    // the category table the combat table is built from, and its rows are
    // named in the order `Balance::lookup_absolute_name` generates. All of
    // that is checked here against the install.
    let obj_masks_parse = units.records.iter().all(|r| {
        r.text("OBJ_MASK").is_none_or(|m| {
            m.chars()
                .all(|c| c.is_ascii_alphabetic() || c.is_ascii_digit() || c == ' ')
        })
    });
    failures += check(
        "every OBJ_MASK is letters and digits (the loader's letter encoding)",
        obj_masks_parse,
        "",
    );
    let ranges_ok = units.records.iter().all(|r| {
        r.text("RANGE").is_none_or(|t| {
            let t = t.trim();
            let mut it = t.splitn(2, '-');
            let a = it.next().unwrap_or("");
            let lead = |s: &str| {
                s.chars()
                    .take_while(char::is_ascii_digit)
                    .collect::<String>()
            };
            !lead(a).is_empty() && it.next().is_none_or(|b| !lead(b).is_empty())
        })
    });
    failures += check(
        "every RANGE is `min` or `min-max` with leading integers",
        ranges_ok,
        "",
    );
    let names = rondata::balance::category_names(&units);
    failures += check(
        "399 combat-table categories: 352 unit rows then the fixed tail",
        names.len() == rondata::balance::CATEGORIES
            && names[rondata::balance::FIRST_LINE] == "SIEGE"
            && names[rondata::balance::FIRST_LINE + 0x0f] == "Flag_AOBJMASK_ARMORED"
            && names[398] == "Flag_6OBJMASK_ANTI_AIR",
        &format!("{} names", names.len()),
    );
    match install.balance() {
        Ok(bx) => {
            let rows = bx.row_names();
            // Every row name is one of the 399, and the rows come in the
            // generated order (a subsequence of it).
            let mut cursor = 0usize;
            let mut in_order = true;
            let mut unknown = Vec::new();
            for r in &rows {
                match names.iter().skip(cursor).position(|n| n == r) {
                    Some(k) => cursor += k + 1,
                    None => {
                        if names.iter().any(|n| n == r) {
                            in_order = false;
                        } else {
                            unknown.push((*r).to_string());
                        }
                    }
                }
            }
            failures += check(
                "balance.xml rows come in the engine's category order",
                in_order,
                &format!("{} rows", rows.len()),
            );
            // Rows the engine cannot name are dead data: it looks rows up by
            // name and a row it cannot name is 100 everywhere. The shipped file
            // has four from units renamed since it was written, and its 32
            // `Flag_X_OBJMASK_*` rows, which the engine composes under another
            // name and never matches (`rondata::balance::tail_names`).
            let (flag_rows, other): (Vec<_>, Vec<_>) = unknown
                .iter()
                .cloned()
                .partition(|n| n.starts_with("Flag_"));
            println!(
                "       rows naming no unit in this install (dead): {}; plus {} Flag_ rows the engine never matches",
                if other.is_empty() {
                    "none".to_string()
                } else {
                    join(other.into_iter())
                },
                flag_rows.len(),
            );
            let non_default = bx
                .rows
                .iter()
                .flat_map(|(_, a)| a.iter())
                .filter(|(_, v)| *v != 100)
                .count();
            failures += check(
                "balance.xml is not the identity (the table needs the file)",
                non_default > 0,
                &format!("{non_default} entries other than 100"),
            );
            // A worked pair: the hardcoded half from the masks alone, the file's
            // half from the categories. The age and the named lineages are not
            // loaded here, so this is the table less those rules — the lineage
            // half waits on a tree loader, and the `RULES=1` log is the oracle.
            let find = |name: &str| {
                units
                    .records
                    .iter()
                    .enumerate()
                    .find(|(_, r)| r.text("NAME") == Some(name))
                    .map(|(i, r)| rondata::balance::unit_kind(i, r))
            };
            if let (Some(a), Some(b)) = (find("Hoplites"), find("Bowmen")) {
                let xml = bx.table(&names);
                let t = sim::Tuning::RON;
                println!(
                    "  Hoplites vs Bowmen: type_damage {} %, with the file {} %; \
                     Bowmen vs Hoplites: {} %, with the file {} %  (masks only; \
                     no age, no lineages)",
                    sim::balance::type_damage(&t, &a, &b),
                    rondata::balance::entry(&t, &xml, &a, &b),
                    sim::balance::type_damage(&t, &b, &a),
                    rondata::balance::entry(&t, &xml, &b, &a),
                );
            }
        }
        Err(e) => {
            failures += check("balance.xml reads", false, &format!("{e}"));
        }
    }

    println!("\nnations ({n_tribes}, index is the TRIBE_MASK bit)");
    for (i, k) in keys.iter().enumerate() {
        print!("{:>3}:{:<11}", i, k);
        if i % 6 == 5 {
            println!();
        }
    }
    if keys.len() % 6 != 0 {
        println!();
    }

    println!("\nconstants that later phases will need");
    for name in ["UNIT_MOVE_SPEED", "UNIT_COST_FACTOR", "OVERKILL_FRAMES"] {
        match rules.constant(name) {
            Some(s) => println!("  {name:<28} {}", describe(s)),
            None => println!("  {name:<28} (absent)"),
        }
    }

    println!("\nworked example");
    if let (Some(speed), Some(citizen)) = (
        rules.constant("UNIT_MOVE_SPEED"),
        units
            .records
            .iter()
            .find(|r| r.text("NAME") == Some("Citizen")),
    ) {
        if let Some(moves) = citizen.text("MOVES").and_then(Scalar::parse) {
            println!(
                "  a Citizen moves {} × {} per frame, at 15 frames/second",
                moves.to_int(),
                describe(speed)
            );
        }
        if let Some(cost) = citizen.text("COST").and_then(rondata::Cost::parse) {
            let factor = rules.constant("UNIT_COST_FACTOR").map_or(1, Scalar::to_int);
            let parts: Vec<String> = cost
                .0
                .iter()
                .map(|(r, n)| format!("{} {}", n * factor, name_of(*r)))
                .collect();
            println!("  and costs {}", join(parts.into_iter()));
        }
    }

    Ok(failures)
}

/// Parses a `.rcx` recorded game and summarises what it carries
/// (`docs/RECGAME.md`). The parse itself is the check: every byte between
/// the gzip header and end of file is accounted for or the reader errors.
/// The embedded combat table is then compared against the one the loader
/// composes from this install. A recording carries the *recording* build's
/// table, so in principle a difference could be patch drift rather than a
/// bug — but the 2017 sample's 493×493 equals ours cell for cell, so drift
/// has never been observed and a difference should be treated as a finding.
fn recgame_report(install: &Install, path: &str) -> Result<usize, rondata::Error> {
    let data = rondata::recgame::decompress(path)?;
    let rec = rondata::recgame::parse(&data, path)?;
    println!("\nrecgame {path}");
    println!("  {}", rec.version);
    println!(
        "  seed {} flags {:#x} save_name {:?} start frame {}",
        rec.seed, rec.flags, rec.save_name, rec.start_frame
    );
    for (n, s) in rec.slots.iter().enumerate() {
        if s.active() {
            println!(
                "  slot {n}: {:?} tribe {} who {} team {} play {} diff {}",
                s.name, s.tribe, s.who, s.team, s.play, s.diff
            );
        }
    }
    let sp = rec.spans;
    println!(
        "  spans: types {:#x}..{:#x} constants {:#x}..{:#x} balance {:#x}..{:#x} tribes {:#x}..{:#x}",
        sp.types.0,
        sp.types.1,
        sp.constants.0,
        sp.constants.1,
        sp.balance.0,
        sp.balance.1,
        sp.tribes.0,
        sp.tribes.1
    );
    let (first, last) = match (rec.packages.first(), rec.packages.last()) {
        (Some(a), Some(b)) => (a.frame, b.frame),
        _ => (0, 0),
    };
    let bytes: usize = rec.packages.iter().map(|p| p.data.len()).sum();
    println!(
        "  {} packages, frames {first}..{last}, {bytes} payload bytes",
        rec.packages.len()
    );
    let mut failures = check(
        "the whole stream parsed, packages to exact end of file",
        !rec.packages.is_empty(),
        "",
    );

    // The embedded combat table against ours, row-major units-then-buildings
    // on both sides (docs/RECGAME.md §4.2, docs/COMBAT.md §15.2).
    if sp.balance.1 > sp.balance.0 {
        let loaded = rondata::load::load(install)?;
        let n = loaded.kinds.len();
        let side = n + loaded.build_kinds.len();
        let table = &data[sp.balance.0..sp.balance.1];
        let at = |i: usize| {
            if i < n {
                sim::combat::TypeRef::Unit(i)
            } else {
                sim::combat::TypeRef::Build(i - n)
            }
        };
        let mut mism = 0usize;
        let mut example = String::new();
        for a in 0..side {
            for b in 0..side {
                let cell = 2 * (a * side + b);
                let want = i32::from(i16::from_le_bytes([table[cell], table[cell + 1]]));
                let ours = loaded.table.pct_of(at(a), at(b));
                if want != ours {
                    mism += 1;
                    if example.is_empty() {
                        example = format!("first: [{a}][{b}] {want} vs ours {ours}");
                    }
                }
            }
        }
        failures += check(
            "the embedded combat table matches the one composed from this install",
            mism == 0,
            &if mism == 0 {
                format!("{side}×{side}, every cell equal")
            } else {
                format!("{mism} of {} cells differ; {example}", side * side)
            },
        );
    }

    // The command payloads (docs/COMMANDS.md). A single-player recording is
    // plain; MP payloads would need the seed-keyed XOR (§5), which no sample
    // exercises yet — a decode failure here on an MP file is the expected
    // signal to route through commands::decode_mp.
    let mut histogram: std::collections::BTreeMap<&'static str, usize> = Default::default();
    let mut bad = 0usize;
    let mut example = String::new();
    for p in &rec.packages {
        match rondata::commands::decode(&p.data) {
            Ok(cmds) => {
                for cmd in &cmds {
                    *histogram.entry(cmd.name()).or_default() += 1;
                }
            }
            Err(e) => {
                bad += 1;
                if example.is_empty() {
                    example = format!("first: stamp {} frame {}: {e}", p.stamp, p.frame);
                }
            }
        }
    }
    failures += check(
        "every command payload decodes to exact size",
        bad == 0,
        &if bad == 0 {
            let total: usize = histogram.values().sum();
            format!("{total} commands across {} packages", rec.packages.len())
        } else {
            format!("{bad} of {} packages failed; {example}", rec.packages.len())
        },
    );
    if bad == 0 {
        let mut by_count: Vec<_> = histogram.into_iter().collect();
        by_count.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
        for (name, n) in by_count {
            println!("    {n:7} {name}");
        }
    }

    // The input itself, minus the two per-frame housekeeping commands. A
    // recording's whole point for the diff harness is this list: what the
    // players did, and on which frame. `Camera` is one a frame and
    // `PlayerSpeed` is the turn pump's, so neither is input
    // (`docs/COMMANDS.md` §4); everything else is.
    if bad == 0 {
        let mut input = Vec::new();
        for p in &rec.packages {
            for cmd in rondata::commands::decode(&p.data).into_iter().flatten() {
                if !matches!(
                    cmd,
                    rondata::commands::Command::Camera { .. }
                        | rondata::commands::Command::PlayerSpeed { .. }
                ) {
                    input.push((p.frame, p.play, cmd));
                }
            }
        }
        println!("  the input, {} commands over {} frames", input.len(), {
            let mut fs: Vec<i32> = input.iter().map(|(f, _, _)| *f).collect();
            fs.dedup();
            fs.len()
        });
        for (frame, play, cmd) in &input {
            println!("    frame {frame:5} play {play}  {cmd:?}");
        }
    }
    Ok(failures)
}

fn check(what: &str, ok: bool, detail: &str) -> usize {
    println!("  [{}] {what}", if ok { "ok" } else { "FAIL" });
    if !detail.is_empty() {
        println!("       {detail}");
    }
    usize::from(!ok)
}

fn describe(s: Scalar) -> String {
    match s {
        Scalar::Int(n) => format!("{n}"),
        Scalar::Ratio { num, den } => format!("{num}/{den}"),
        Scalar::Percent(p) => format!("{p}%"),
        Scalar::Multiplier(m) => format!("{m}x"),
    }
}

fn name_of(r: Resource) -> &'static str {
    match r {
        Resource::Food => "food",
        Resource::Timber => "timber",
        Resource::Gold => "gold",
        Resource::Knowledge => "knowledge",
        Resource::Metal => "metal",
        Resource::Oil => "oil",
    }
}

/// The same six, named the way `resourcerules.xml` names them — which calls
/// slot 2 "Wealth" where the cost grammar's letter for it is `g`.
fn sim_name_of(r: sim::economy::Resource) -> &'static str {
    match r {
        sim::economy::Resource::Food => "food",
        sim::economy::Resource::Timber => "timber",
        sim::economy::Resource::Wealth => "wealth",
        sim::economy::Resource::Knowledge => "knowledge",
        sim::economy::Resource::Metal => "metal",
        sim::economy::Resource::Oil => "oil",
    }
}

fn join(items: impl Iterator<Item = String>) -> String {
    items.collect::<Vec<_>>().join(", ")
}

fn uniq(v: &[usize]) -> Vec<usize> {
    let mut out: Vec<usize> = v.to_vec();
    out.sort_unstable();
    out.dedup();
    out
}
