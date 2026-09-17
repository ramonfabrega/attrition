//! Read-only measurement of capture input/index/seed-scan costs.
//! Run in release; arguments are a repetition count followed by capture paths.
use rondata::gamelog::Log;
use std::{hint::black_box, time::Instant};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if cfg!(debug_assertions) {
        return Err("run with --release".into());
    }
    let mut args = std::env::args().skip(1);
    let repetitions: usize = args.next().ok_or("missing repetition count")?.parse()?;
    assert!(repetitions > 0);
    println!(
        "path,iteration,bytes,read_ms,index_ms,frames_ms,seed_scan_ms,repeat_seed_scan_ms,drop_ms,frames,seeds"
    );
    for path in args {
        let mut expected = None;
        for iteration in 0..repetitions {
            let start = Instant::now();
            // Fail loudly on missing/unreadable input; an empty fallback is not a sample.
            let text = std::fs::read_to_string(&path)?;
            let read = start.elapsed();
            let start = Instant::now();
            let log = Log::parse(black_box(&text));
            let index = start.elapsed();
            let start = Instant::now();
            let frames: Vec<_> = log.frames().iter().map(|(n, _)| *n).collect();
            let frame_time = start.elapsed();
            let start = Instant::now();
            let seeds = black_box(log.frame_seeds());
            let seed_time = start.elapsed();
            let start = Instant::now();
            let repeated_seeds = black_box(log.frame_seeds());
            let repeat_seed_time = start.elapsed();
            assert_eq!(seeds, repeated_seeds);
            let observed = (frames, seeds);
            if let Some(ref expected) = expected {
                assert_eq!(
                    &observed, expected,
                    "input or result changed between repeats"
                );
            }
            let frame_count = observed.0.len();
            let seed_count = observed.1.len();
            expected = Some(observed);
            let bytes = text.len();
            let start = Instant::now();
            drop(log);
            drop(text);
            let drop_time = start.elapsed();
            println!(
                "{path},{iteration},{bytes},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},{frame_count},{seed_count}",
                read.as_secs_f64() * 1000.0,
                index.as_secs_f64() * 1000.0,
                frame_time.as_secs_f64() * 1000.0,
                seed_time.as_secs_f64() * 1000.0,
                repeat_seed_time.as_secs_f64() * 1000.0,
                drop_time.as_secs_f64() * 1000.0
            );
        }
    }
    Ok(())
}
