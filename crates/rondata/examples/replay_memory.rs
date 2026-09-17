//! Measure replay retention and save its complete report for before/after comparison.
use std::{
    fs::OpenOptions,
    io::{BufWriter, Write},
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    if cfg!(debug_assertions) {
        return Err("use --release".into());
    }
    let args: Vec<_> = std::env::args().skip(1).collect();
    if !(4..=5).contains(&args.len()) {
        return Err("INSTALL CAPTURE LIMIT NEW_REPORT_PATH [memory|indexed|verify-setup]".into());
    }
    let output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&args[3])?;
    let mut output = BufWriter::new(output);
    let loaded = rondata::load::load(&rondata::Install::new(&args[0]))?;
    let start = std::time::Instant::now();
    let report = match args.get(4).map(String::as_str).unwrap_or("memory") {
        "indexed" => {
            let mut source = rondata::capture::indexed::IndexedCapture::open(&args[1])?;
            rondata::diff::run_indexed_observed(
                &loaded,
                &mut source,
                sim::Tuning::RON,
                Some(args[2].parse()?),
                None,
                &[],
                None,
                |_, _, _| {},
            )?
        }
        "verify-setup" => {
            let text = std::fs::read_to_string(&args[1])?;
            let log = rondata::gamelog::Log::parse(&text);
            let mut expected = log.initial().ok_or("no initial state")?;
            expected.frame_bodies.clear();
            let mut source = rondata::capture::indexed::IndexedCapture::open(&args[1])?;
            source.with_replay_initial(|actual| {
                assert!(actual == expected, "complete replay setup differs")
            })?;
            writeln!(output, "complete replay setup equal")?;
            output.flush()?;
            return Ok(());
        }
        "memory" => {
            let text = std::fs::read_to_string(&args[1])?;
            let log = rondata::gamelog::Log::parse(&text);
            rondata::diff::run_traced(
                &loaded,
                &log,
                sim::Tuning::RON,
                Some(args[2].parse()?),
                None,
                &[],
                None,
            )
            .ok_or("no initial state")?
        }
        _ => return Err("unknown backend".into()),
    };
    eprintln!(
        "{} frames, load + replay {:.3}s",
        report.frames.len(),
        start.elapsed().as_secs_f64()
    );
    write!(output, "{report:#?}")?;
    output.flush()?;
    Ok(())
}
