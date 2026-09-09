//! Export a bounded, read-only view of the existing differential replay.
use rondata::{Install, diff, gamelog::Log, input::Stream};
use std::{fmt::Write as _, fs::OpenOptions, io::Write, path::PathBuf};

fn quoted(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '<' => out.push_str("\\u003c"),
            c if c.is_control() => {
                write!(out, "\\u{:04x}", c as u32).unwrap();
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}
fn strings(xs: &[String]) -> String {
    format!(
        "[{}]",
        xs.iter().map(|s| quoted(s)).collect::<Vec<_>>().join(",")
    )
}
fn point(p: sim::Pos) -> String {
    format!("[{},{}]", p.x, p.y)
}
fn row(
    who: i64,
    o: i64,
    original: Option<&rondata::gamelog::UnitDump>,
    ours: Option<&sim::Unit>,
    scope: bool,
) -> String {
    let a = original.map_or("null".into(), |u| format!("[{},{}]", u.pos.x, u.pos.y));
    let b = ours.map_or("null".into(), |u| point(u.pos));
    let original_path = original.map_or(Vec::new(), |u| {
        u.path
            .iter()
            .rev()
            .map(|p| format!("[{},{}]", p.to.0, p.to.1))
            .collect()
    });
    let rust_path = ours.map_or(Vec::new(), |u| {
        u.path.iter().rev().map(|p| point(p.to)).collect()
    });
    format!(
        "{{\"id\":{},\"who\":{who},\"o\":{o},\"original\":{a},\"rust\":{b},\"scope\":{scope},\"originalPath\":[{}],\"rustPath\":[{}],\"originalRecord\":{},\"rustRecord\":{}}}",
        quoted(&format!("{who}/{o}")),
        original_path.join(","),
        rust_path.join(","),
        quoted(&original.map_or("Not present in this frame".into(), |u| format!("{u:#?}"))),
        quoted(
            &ours.map_or("Not linked in this comparison".into(), |u| format!(
                "{u:#?}"
            ))
        )
    )
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    if cfg!(debug_assertions) {
        return Err("use --release".into());
    }
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() < 3 {
        return Err("INSTALL CAPTURE NEW_OUTPUT.html [--from FRAME] [--count 1..200] [--sibling CAPTURE] [--trace TRACE] [--recording REC]".into());
    }
    let (install, capture, output) = (&args[0], &args[1], PathBuf::from(&args[2]));
    if output.exists() {
        return Err("output already exists; choose a new file".into());
    }
    let (mut from, mut count) = (1i64, 40usize);
    let (mut siblings, mut trace_path, mut recording) = (Vec::new(), None, None);
    let mut i = 3;
    while i < args.len() {
        let value = args.get(i + 1).ok_or("option needs a value")?;
        match args[i].as_str() {
            "--from" => from = value.parse()?,
            "--count" => count = value.parse()?,
            "--sibling" => siblings.push(value.clone()),
            "--trace" => trace_path = Some(value.clone()),
            "--recording" => recording = Some(value.clone()),
            _ => return Err("unknown option".into()),
        }
        i += 2;
    }
    if !(1..=200).contains(&count) {
        return Err("count must be 1..200".into());
    }
    let loaded = rondata::load::load(&Install::new(install))?;
    let text = std::fs::read_to_string(capture)?;
    let log = Log::parse(&text);
    let frame_names = log.frames();
    let start = frame_names
        .iter()
        .position(|(n, _)| *n >= from)
        .ok_or("no frames at or after --from")?;
    let limit = (start + count).min(frame_names.len());
    let sibling_texts = siblings
        .iter()
        .map(std::fs::read_to_string)
        .collect::<Result<Vec<_>, _>>()?;
    let sibling_logs: Vec<_> = sibling_texts.iter().map(|s| Log::parse(s)).collect();
    let inits: Vec<_> = sibling_logs
        .iter()
        .map(|l| l.initial().ok_or("sibling lacks initial state"))
        .collect::<Result<_, _>>()?;
    let refs: Vec<_> = inits.iter().collect();
    let trace = trace_path
        .as_ref()
        .map(|p| {
            rondata::trace::Trace::read(std::path::Path::new(p))
                .and_then(|t| t.ok_or_else(|| std::io::Error::other("trace header missing")))
        })
        .transpose()?;
    let rec = recording
        .as_ref()
        .map(|p| rondata::recgame::read(p))
        .transpose()?;
    let mut stream = rec.as_ref().map(Stream::new);
    let (mut frames, mut visited, mut bytes, mut too_large) = (Vec::new(), 0usize, 0usize, false);
    let mut geometry = (0, 0);
    let mut seed_inputs = 0;
    let mut guy_inputs = 0;
    let report = diff::run_traced_observed(&loaded, &log, sim::Tuning::RON, Some(limit), stream.as_mut(), &refs, trace.as_ref(), |built, frame, result| {
        let index = visited; visited += 1;
        if index < start || too_large { return; }
        geometry = (built.sim.world.width() * sim::world::UNITS_PER_CELL, built.sim.world.height() * sim::world::UNITS_PER_CELL);
        seed_inputs = built.frame_seeds.len(); guy_inputs = built.frame_guys.len();
        let players = built.sim.players.len() as i64;
        let mut units = Vec::new();
        for unit in &frame.units {
            let scope = (0..players).contains(&unit.who);
            let linked = if scope {
                built.units.iter().find(|l| l.who == unit.who && l.o == unit.o).map(|l| l.unit)
                    .or_else(|| i16::try_from(unit.o).ok().and_then(|o| built.sim.unit_by_o(unit.who as sim::Player, o)))
            } else { None };
            units.push(row(unit.who, unit.o, Some(unit), linked.map(|id| &built.sim.units[id]), scope));
        }
        for &(who,o) in &result.extra_units {
            let ours = i16::try_from(o).ok().and_then(|o| built.sim.unit_by_o(who as sim::Player, o)).map(|id| &built.sim.units[id]);
            units.push(row(who, o, None, ours, true));
        }
        let issues = result.diverged.len() + result.unlinked + result.extra_units.len() + result.order_diverged.len() + result.los_diverged.len() + result.packed_diverged.len() + result.angle_diverged.len() + result.collide_diverged.len() + result.gather_diverged.len() + result.build_unlinked + result.build_diverged.len() + result.queue_diverged.len() + result.city_diverged.len() + result.city_unlinked;
        let json = format!("{{\"n\":{},\"index\":{index},\"issues\":{issues},\"compared\":{},\"positionMismatches\":{},\"orderCompared\":{},\"rng\":{},\"seedInstalledThisTick\":{},\"units\":[{}],\"report\":{}}}", frame.n, result.compared, result.diverged.len(), result.order_compared, built.sim.rng.seed, built.frame_seeds.iter().any(|(n,_)| *n == frame.n - 1), units.join(","), quoted(&format!("{result:#?}")));
        bytes += json.len();
        if bytes > 64*1024*1024 { too_large = true; return; }
        frames.push(json);
    }).ok_or("capture lacks initial state")?;
    if too_large {
        return Err("export exceeds 64 MiB; reduce --count".into());
    }
    if frames.is_empty() {
        return Err("empty export".into());
    }
    let revision = std::process::Command::new("git")
        .args(["describe", "--always", "--dirty"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map_or("unknown".into(), |o| {
            String::from_utf8_lossy(&o.stdout).trim().to_owned()
        });
    let reproduction =
        std::iter::once("cargo run -p rondata --release --example debug_view --".to_owned())
            .chain(
                args.iter()
                    .map(|a| format!("'{}'", a.replace('\'', "'\\''"))),
            )
            .collect::<Vec<_>>()
            .join(" ");
    let data = format!(
        "{{\"schema\":1,\"capture\":{},\"sourceBytes\":{},\"revision\":{},\"siblings\":{},\"trace\":{},\"recording\":{},\"seedInputs\":{seed_inputs},\"guyInputs\":{guy_inputs},\"world\":[{},{}],\"notes\":{},\"applied\":{},\"reproduce\":{},\"frames\":[{}]}}",
        quoted(capture),
        text.len(),
        quoted(&revision),
        strings(&siblings),
        quoted(trace_path.as_deref().unwrap_or("Not supplied")),
        quoted(recording.as_deref().unwrap_or("Not supplied")),
        geometry.0,
        geometry.1,
        strings(&report.notes),
        quoted(&format!("{:#?}", report.applied)),
        quoted(&reproduction),
        frames.join(",")
    );
    if data.len() > 64 * 1024 * 1024 {
        return Err("export exceeds 64 MiB; reduce --count".into());
    }
    let template = include_str!("../../../tools/viewer/viewer.html");
    let html = template
        .replace(
            "/*__VIEWER_CODE__*/",
            include_str!("../../../tools/viewer/viewer.js"),
        )
        .replace("__REPLAY_DATA__", &data);
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&output)?;
    file.write_all(html.as_bytes())?;
    println!(
        "{} frames, {} bytes: {}",
        frames.len(),
        html.len(),
        output.display()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn json_string_cannot_close_the_embedded_data_script() {
        assert_eq!(
            quoted("</script>\n\"\\"),
            "\"\\u003c/script>\\u000a\\\"\\\\\""
        );
    }
}
