//! Explicit local trial driver: pack SOURCE DEST.rcap | compare SOURCE ARCHIVE.rcap
use rondata::capture::{archive, indexed::IndexedCapture};
use std::io;
use std::time::Instant;
fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 4 {
        return Err(io::Error::other(
            "usage: capture_archive pack|compare SOURCE ARCHIVE.rcap",
        ));
    }
    let started = Instant::now();
    match args[1].as_str() {
        "pack" => {
            archive::pack(&args[2], &args[3])?;
            println!(
                "pack_verified_seconds={:.6} raw_bytes={} archive_bytes={}",
                started.elapsed().as_secs_f64(),
                std::fs::metadata(&args[2])?.len(),
                std::fs::metadata(&args[3])?.len()
            );
        }
        "compare" => {
            let mut raw = IndexedCapture::open(&args[2])?;
            println!("raw_index_seconds={:.6}", started.elapsed().as_secs_f64());
            let t = Instant::now();
            let mut packed = IndexedCapture::open(&args[3])?;
            println!("archive_index_seconds={:.6}", t.elapsed().as_secs_f64());
            assert_eq!(raw.frames(), packed.frames());
            assert_eq!(raw.source_bytes(), packed.source_bytes());
            assert_eq!(raw.read_replay_setup()?, packed.read_replay_setup()?);
            assert_eq!(raw.read_shutdown()?, packed.read_shutdown()?);
            let len = raw.frames().len();
            let mut random: Vec<usize> = (0..len).collect();
            let mut state = 42u64;
            for i in (1..len).rev() {
                state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
                random.swap(i, (state % (i as u64 + 1)) as usize);
            }
            for (label, order) in [
                ("forward", (0..len).collect::<Vec<_>>()),
                ("reverse", (0..len).rev().collect()),
                ("random", random),
            ] {
                let t = Instant::now();
                let mut raw_time = std::time::Duration::ZERO;
                let mut archive_time = std::time::Duration::ZERO;
                for &i in &order {
                    let r = Instant::now();
                    let frame = raw.read_frame(i)?;
                    let siblings = raw.read_frame_and_siblings(i)?;
                    raw_time += r.elapsed();
                    let r = Instant::now();
                    let packed_frame = packed.read_frame(i)?;
                    let packed_siblings = packed.read_frame_and_siblings(i)?;
                    archive_time += r.elapsed();
                    assert_eq!(frame, packed_frame, "frame {i}");
                    assert_eq!(siblings, packed_siblings, "siblings {i}");
                }
                println!(
                    "{label}_equal_frames={len} seconds={:.6} raw_read_seconds={:.6} archive_read_seconds={:.6}",
                    t.elapsed().as_secs_f64(),
                    raw_time.as_secs_f64(),
                    archive_time.as_secs_f64()
                );
            }
            let mut reopened = IndexedCapture::open(&args[3])?;
            assert_eq!(raw.frames(), reopened.frames());
            assert_eq!(raw.read_shutdown()?, reopened.read_shutdown()?);
            println!("comparison_seconds={:.6}", started.elapsed().as_secs_f64());
        }
        _ => return Err(io::Error::other("unknown command")),
    }
    Ok(())
}
