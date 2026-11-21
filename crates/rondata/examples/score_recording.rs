//! Score a recording against a checksum log or a verified native capture directory.
use rondata::checksum::{CHECKSUM_LINES, playback};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::Path;

struct Capture {
    last: i32,
    requested: i32,
    early: bool,
}

impl Capture {
    fn read(dir: &Path, recording: &Path) -> Result<Self, String> {
        let path = dir.join("receipt.json");
        let bytes = std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let r: Value =
            serde_json::from_slice(&bytes).map_err(|e| format!("{}: {e}", path.display()))?;
        if r["success"] != true
            || r["exit_code"] != 0
            || r["lifecycle_verified"] != true
            || r["map_verified"] != true
        {
            return Err("capture receipt has no verified successful playback".into());
        }
        let number = |key: &str| {
            r[key]
                .as_u64()
                .and_then(|n| i32::try_from(n).ok())
                .ok_or_else(|| format!("invalid capture receipt {key}"))
        };
        let last = number("last_frame")?;
        let requested = number("end_frame")?;
        let frames = number("frames")?;
        let early = last < requested;
        if last.checked_add(1) != Some(frames)
            || frames != number("closing_frame")?
            || last > requested
            || r["ended_early"] != early
        {
            return Err("inconsistent capture receipt endpoint".into());
        }
        let bytes =
            std::fs::read(recording).map_err(|e| format!("{}: {e}", recording.display()))?;
        if r["recording"]["sha256"] != format!("{:x}", Sha256::digest(bytes)) {
            return Err("recording SHA-256 differs from its capture receipt".into());
        }
        Ok(Self {
            last,
            requested,
            early,
        })
    }
}

fn run() -> Result<bool, String> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let [install, recording, take] = args.as_slice() else {
        return Err("usage: score_recording INSTALL GAME.rcx TAKE_DIRECTORY|rontrace.log".into());
    };
    let (install, recording, take) = (Path::new(install), Path::new(recording), Path::new(take));
    let mut sums_only = None;
    let score = if take.is_dir() {
        let capture = Capture::read(take, recording)?;
        let ranges = take.join("rontrace.ranges");
        let has_ranges = ranges.try_exists().map_err(|e| e.to_string())?;
        let score = playback::diagnose_capture(
            install,
            recording,
            &take.join("rontrace.log"),
            capture.last,
            has_ranges.then_some(ranges.as_path()),
        )?;
        let scope = if capture.early {
            "verified original early ending"
        } else if score.through_closing {
            "recording through closing package"
        } else {
            "requested capture window"
        };
        println!(
            "{scope}: states 0..={}, requested endpoint {}",
            capture.last, capture.requested
        );
        if let Some(f) = score.first.iter().flatten().map(|(f, ..)| f).min() {
            let lines: Vec<_> = score
                .first
                .iter()
                .zip(CHECKSUM_LINES)
                .filter_map(|(first, (name, _, _))| (first.as_ref()?.0 == *f).then_some(name))
                .collect();
            println!("first differing state {f} ({})", lines.join(", "));
        }
        if !has_ranges {
            sums_only = Some((capture.requested, capture.early));
        }
        score
    } else {
        playback::compare(install, recording, take)?
    };
    for (i, first) in score.first.iter().enumerate() {
        print!(
            "{} {}/{}",
            CHECKSUM_LINES[i].0, score.agreed[i], score.states
        );
        if let Some((f, ours, theirs)) = first {
            let ours = ours.map_or_else(|| "unemitted".to_owned(), |v| v.to_string());
            print!(", first differing state {f}: ours {ours}, original {theirs}");
        }
        if let Some(detail) = &score.details[i] {
            print!("\n  {detail}");
        }
        println!();
    }
    // A take that holds sums names no leaf: one take keeps every parted
    // line's ranges to the game's end, and no line that is whole.
    if let Some((requested, early)) = sums_only
        && let Some(args) = playback::joined_take(&score.first, requested, early)
    {
        println!("one take for every parted line: {args}");
    }
    Ok(score.first.iter().all(Option::is_none))
}

fn main() -> std::process::ExitCode {
    match run() {
        Ok(true) => std::process::ExitCode::SUCCESS,
        Ok(false) => 1.into(),
        Err(e) => {
            eprintln!("{e}");
            2.into()
        }
    }
}
