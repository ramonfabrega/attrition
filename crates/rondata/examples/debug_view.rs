//! Export a bounded, read-only view of the existing differential replay.
use rondata::{Install, diff, gamelog::Log, input::Stream};
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if cfg!(debug_assertions) {
        return Err("use --release".into());
    }
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() < 3 {
        return Err("INSTALL CAPTURE NEW_OUTPUT.html [--reader memory|indexed|checkpoint] [--windows 1..8] [--corrections standard|without-future-figures|compare-figures] [--from FRAME] [--count 1..200] [--sibling CAPTURE] [--trace TRACE] [--recording REC]".into());
    }
    let (install, capture, output) = (&args[0], &args[1], PathBuf::from(&args[2]));
    if output.exists() {
        return Err("output already exists; choose a new file".into());
    }
    let mut reader = "memory";
    let mut corrections = "standard";
    let mut windows = 1usize;
    let (mut from, mut count) = (1i64, 40usize);
    let (mut siblings, mut trace_path, mut recording) = (Vec::new(), None, None);
    let mut i = 3;
    while i < args.len() {
        let value = args.get(i + 1).ok_or("option needs a value")?;
        match args[i].as_str() {
            "--reader" => reader = value,
            "--corrections" => corrections = value,
            "--windows" => windows = value.parse()?,
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
    if !["standard", "without-future-figures", "compare-figures"].contains(&corrections)
        || (reader != "checkpoint" && corrections != "standard")
    {
        return Err(
            "nonstandard corrections require checkpoint reader and without-future-figures policy"
                .into(),
        );
    }
    let loaded = rondata::load::load(&Install::new(install))?;
    if !(1..=8).contains(&windows) || (reader != "checkpoint" && windows != 1) {
        return Err("windows must be 1..8 and multiple windows require checkpoint reader".into());
    }
    if !["memory", "indexed", "checkpoint"].contains(&reader) {
        return Err("reader must be memory, indexed or checkpoint".into());
    }
    let text = if reader == "memory" {
        std::fs::read_to_string(capture)?
    } else {
        String::new()
    };
    let log = Log::parse(&text);
    let mut source = if reader != "memory" {
        Some(rondata::capture::indexed::IndexedCapture::open(capture)?)
    } else {
        None
    };
    let frame_names: Vec<i64> = match &source {
        Some(source) => source.frames().iter().map(|f| f.number).collect(),
        None => log.frames().iter().map(|(n, _)| *n).collect(),
    };
    let start = frame_names
        .iter()
        .position(|n| *n >= from)
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
        source_bytes: source
            .as_ref()
            .map_or(text.len(), |s| s.source_bytes() as usize),
        siblings: siblings.clone(),
        trace: trace_path.clone(),
        recording: recording.clone(),
        reproduce: reproduction,
        focus_index: None,
    };
    if reader == "checkpoint" {
        let source = source.as_mut().unwrap();
        let end = start
            .checked_add(count * windows)
            .ok_or("window range overflow")?;
        if end > frame_names.len() {
            return Err("requested checkpoint windows exceed capture".into());
        }
        let outputs: Vec<_> = (0..windows)
            .map(|i| {
                if i == 0 {
                    output.clone()
                } else {
                    output.with_file_name(format!(
                        "{}.window-{i}.html",
                        output.file_stem().unwrap().to_string_lossy()
                    ))
                }
            })
            .collect();
        let outputs: Vec<_> = outputs
            .into_iter()
            .enumerate()
            .flat_map(|(w, path)| {
                if corrections == "compare-figures" {
                    let variant = path.with_file_name(format!(
                        "{}.without-future-figures.html",
                        path.file_stem().unwrap().to_string_lossy()
                    ));
                    vec![
                        (w, path, "standard"),
                        (w, variant, "without-future-figures"),
                    ]
                } else {
                    vec![(w, path, corrections)]
                }
            })
            .collect();
        if outputs.iter().any(|(_, p, _)| p.exists()) {
            return Err("checkpoint output already exists".into());
        }
        let prefix_start = std::time::Instant::now();
        let mut session = source.with_replay_initial(|init| {
            let mut init = init;
            diff::borrow_from_siblings(&mut init, &refs);
            if let Some(trace) = &trace {
                diff::borrow_pasture(&mut init, trace);
            }
            diff::ReplaySession::new(&loaded, &init, sim::Tuning::RON, stream.take())
        })?;
        for i in 0..start {
            session.push(i, &source.frame_state(i)?)?;
        }
        source.validate()?;
        eprintln!(
            "checkpoint prefix: {:?}; next source index {}",
            prefix_start.elapsed(),
            session.next_record()
        );
        // This operation owns one source handle. Checkpoints cannot be rebound
        // to another capture through the CLI; ordinary mutations are validated.
        for (w, destination, policy) in &outputs {
            source.validate()?;
            let begin = std::time::Instant::now();
            let mut restored = if *policy == "standard" {
                session.clone()
            } else {
                session.fork_without_future_figure_corrections()?
            };
            let first = start + *w * count;
            let mut window =
                rondata::debug_view::Window::resume(first, count, restored.next_record())?;
            for i in restored.next_record()..first + count {
                let frame = source.frame_state(i)?;
                let result = restored.push(i, &frame)?.clone();
                window.observe(restored.built(), &frame, &result);
            }
            source.validate()?;
            let report = restored.finish();
            let bytes = window.write(&report, &meta, destination)?;
            eprintln!("checkpoint window {w} ({policy}): {:?}", begin.elapsed());
            println!("{bytes} bytes: {}", destination.display());
        }
        return Ok(());
    }
    let mut window = rondata::debug_view::Window::new(start, count)?;
    let report = if let Some(source) = &mut source {
        diff::run_indexed_observed(
            &loaded,
            source,
            sim::Tuning::RON,
            Some(limit),
            stream.as_mut(),
            &refs,
            trace.as_ref(),
            |built, frame, result| window.observe(built, frame, result),
        )?
    } else {
        diff::run_traced_observed(
            &loaded,
            &log,
            sim::Tuning::RON,
            Some(limit),
            stream.as_mut(),
            &refs,
            trace.as_ref(),
            |built, frame, result| window.observe(built, frame, result),
        )
        .ok_or("capture lacks initial state")?
    };
    let bytes = window.write(&report, &meta, &output)?;
    println!("{bytes} bytes: {}", output.display());
    Ok(())
}
