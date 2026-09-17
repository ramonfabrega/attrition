//! Verify complete shutdown records, or measure either reader, on local captures.
use rondata::{capture::indexed::IndexedCapture, gamelog::Log};
use std::{hint::black_box, path::PathBuf, time::Instant};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    let mode = args.get(1).ok_or("mode: whole, indexed, verify")?;
    if !["whole", "indexed", "verify"].contains(&mode.as_str()) {
        return Err("unknown mode".into());
    }
    let path = PathBuf::from(args.get(2).ok_or("capture file or directory")?);
    let paths = if path.is_dir() {
        let mut paths = std::fs::read_dir(path)?
            .map(|e| e.map(|e| e.path()))
            .collect::<Result<Vec<_>, _>>()?;
        paths.retain(|p| {
            p.file_name().is_some_and(|n| {
                let n = n.to_string_lossy();
                n.starts_with("gamelog-") && n.ends_with(".txt")
            })
        });
        paths.sort();
        paths
    } else {
        vec![path]
    };
    let start = Instant::now();
    let (mut files, mut closing, mut units, mut largest) = (0, 0, 0, 0);
    for path in paths {
        let expected = if mode != "indexed" {
            Some(Log::parse(&std::fs::read_to_string(&path)?).final_state())
        } else {
            None
        };
        let state = if mode != "whole" {
            let mut source = IndexedCapture::open(&path)?;
            let text = source.read_shutdown()?;
            largest = largest.max(text.len());
            let state = Log::parse(&text).final_state();
            if let Some(expected) = expected {
                assert_eq!(state, expected, "{}", path.display());
            }
            state
        } else {
            expected.unwrap()
        };
        files += 1;
        if let Some(state) = state {
            closing += 1;
            units += state.units.len();
            black_box(state);
        }
        println!(
            "{} {}",
            if mode == "verify" { "matched" } else { "read" },
            path.display()
        );
    }
    assert!(files > 0, "no capture files");
    println!(
        "mode={mode} files={files} closing={closing} units={units} largest_tail={largest} elapsed_ms={:.3}",
        start.elapsed().as_secs_f64() * 1000.0
    );
    Ok(())
}
