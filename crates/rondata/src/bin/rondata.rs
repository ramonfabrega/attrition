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

use rondata::{Install, Resource, Scalar};
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let Some(root) = args.next() else {
        eprintln!("usage: rondata <install-root>");
        eprintln!();
        eprintln!("The directory holding riseofnations.exe. No game data is");
        eprintln!("copied anywhere; this only reads.");
        return ExitCode::from(2);
    };

    let install = Install::new(&root);
    if !install.looks_valid() {
        eprintln!("{root}: no Data/rules.xml here — is this the install root?");
        return ExitCode::from(2);
    }

    match survey(&install) {
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
    let resources = install.resources()?;
    let mut redirect_drift = Vec::new();
    for (i, r) in Resource::ALL.iter().enumerate() {
        let Some(rec) = resources.records.get(i) else {
            redirect_drift.push(format!("{} is missing", name_of(*r)));
            continue;
        };
        let ours = sim::cost::Redirects::RON.undiscovered[i];
        let theirs_good = rec.text("UNDISC_SUPPORT_GOOD").unwrap_or("");
        let theirs_rate = rec
            .text("UNDISC_SUPPORT_RATE")
            .and_then(Scalar::parse)
            .map(|s| s.to_fx().raw() / 256);
        let good_agrees = theirs_good.eq_ignore_ascii_case(sim_name_of(ours.good));
        if !good_agrees || theirs_rate != Some(ours.rate) {
            redirect_drift.push(format!(
                "{}: we say {} at {}, install says {theirs_good} at {:?}",
                name_of(*r),
                sim_name_of(ours.good),
                ours.rate,
                theirs_rate
            ));
        }
    }
    failures += check(
        "the undiscovered-resource redirect matches this install",
        redirect_drift.is_empty(),
        &if redirect_drift.is_empty() {
            "metal is charged as timber, knowledge as food, the rest as wealth".into()
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
