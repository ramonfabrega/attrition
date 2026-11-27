//! Print every range that differs between our walk of a recording and the
//! original's ranges record, over a window of states.
//!
//! ```text
//! print_ranges INSTALL GAME.rcx TAKE_DIRECTORY|rontrace.ranges LO HI [--rec LINE:RECORD] [--all]
//! ```
//!
//! `score_recording` names each line's first leaf; this prints the states
//! round one. `--rec units:68` prints record 68 of the units line on both
//! sides on every state of the window, parted or not; `--all` prints every
//! field of a parted range. Exit 0 when nothing differs, 1 when something
//! does, 2 on an error.
use rondata::checksum::playback::{Window, print_ranges};
use std::path::Path;

const USAGE: &str = "usage: print_ranges INSTALL GAME.rcx TAKE_DIRECTORY|rontrace.ranges LO HI \
                     [--rec LINE:RECORD] [--all]";

fn run() -> Result<bool, String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (mut plain, mut rec, mut all) = (Vec::new(), None, false);
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--all" => all = true,
            "--rec" => {
                let v = it.next().ok_or(USAGE)?;
                let (line, record) = v.split_once(':').ok_or(USAGE)?;
                rec = Some((
                    line.to_string(),
                    record.parse::<i16>().map_err(|e| format!("{v}: {e}"))?,
                ));
            }
            _ => plain.push(a.as_str()),
        }
    }
    let [install, recording, take, lo, hi] = plain.as_slice() else {
        return Err(USAGE.into());
    };
    let take = Path::new(take);
    let ranges = if take.is_dir() {
        take.join("rontrace.ranges")
    } else {
        take.to_path_buf()
    };
    let window = Window {
        lo: lo.parse().map_err(|e| format!("{lo}: {e}"))?,
        hi: hi.parse().map_err(|e| format!("{hi}: {e}"))?,
        record: rec.as_ref().map(|(l, r)| (l.as_str(), *r)),
        all,
    };
    let differing = print_ranges(
        Path::new(install),
        Path::new(recording),
        &ranges,
        &window,
        &mut |line| println!("{line}"),
    )?;
    println!(
        "{differing} ranges differ on states {}..={}",
        window.lo, window.hi
    );
    Ok(differing == 0)
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
