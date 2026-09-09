//! Export a bounded, read-only view of the existing differential replay.
use rondata::{Install, diff, gamelog::Log, input::Stream};
use std::path::PathBuf;

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
    let mut window = rondata::debug_view::Window::new(start, count)?;
    let report = diff::run_traced_observed(
        &loaded,
        &log,
        sim::Tuning::RON,
        Some(limit),
        stream.as_mut(),
        &refs,
        trace.as_ref(),
        |built, frame, result| window.observe(built, frame, result),
    )
    .ok_or("capture lacks initial state")?;
    let reproduction =
        std::iter::once("cargo run -p rondata --release --example debug_view --".to_owned())
            .chain(
                args.iter()
                    .map(|a| format!("'{}'", a.replace('\'', "'\\''"))),
            )
            .collect::<Vec<_>>()
            .join(" ");
    let meta = rondata::debug_view::Metadata {
        capture: capture.clone(),
        source_bytes: text.len(),
        siblings,
        trace: trace_path,
        recording,
        reproduce: reproduction,
        focus_index: None,
    };
    let bytes = window.write(&report, &meta, &output)?;
    println!("{bytes} bytes: {}", output.display());
    Ok(())
}
