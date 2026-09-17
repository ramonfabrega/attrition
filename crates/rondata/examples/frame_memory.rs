//! Compare indexed frame loading with the whole-text reader in separate runs.
use rondata::{capture::indexed::IndexedCapture, gamelog::Log};
use std::{hint::black_box, time::Instant};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    let mode = args
        .get(1)
        .ok_or("mode: whole, indexed, indexed-warm, or verify")?;
    let path = args.get(2).ok_or("capture path")?;
    if mode == "indexed-warm" {
        black_box(IndexedCapture::open(path)?);
    }
    let start = Instant::now();
    let mut count = 0;
    let mut units = 0;
    match mode.as_str() {
        "whole" => {
            let text = std::fs::read_to_string(path)?;
            let log = Log::parse(&text);
            for frame in log.frame_states() {
                count += 1;
                units += frame.units.len();
                black_box(frame);
            }
        }
        "indexed" | "indexed-warm" | "verify" => {
            let mut source = IndexedCapture::open(path)?;
            let index_time = start.elapsed();
            let expected = if mode == "verify" {
                let text = std::fs::read_to_string(path)?;
                Some(Log::parse(&text).frame_states())
            } else {
                None
            };
            let mut largest = 0;
            for i in 0..source.frames().len() {
                let text = source.read_frame(i)?;
                largest = largest.max(text.len());
                let log = Log::parse(&text);
                let frames = log.frame_states();
                assert_eq!(frames.len(), 1);
                assert_eq!(frames[0].n, source.frames()[i].number);
                if let Some(ref expected) = expected {
                    assert_eq!(frames[0], expected[i], "frame {}", frames[0].n);
                }
                count += 1;
                units += frames[0].units.len();
                black_box(frames);
            }
            if let Some(expected) = expected {
                assert_eq!(count, expected.len());
            }
            println!(
                "source_bytes={} largest_frame_bytes={largest} index_ms={:.3}",
                source.source_bytes(),
                index_time.as_secs_f64() * 1000.0
            );
        }
        _ => return Err("unknown mode".into()),
    }
    println!(
        "mode={mode} frames={count} units={units} elapsed_ms={:.3}",
        start.elapsed().as_secs_f64() * 1000.0
    );
    Ok(())
}
