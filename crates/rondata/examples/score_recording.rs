//! Replay a locally minted recording against a RON_CHECKSUMS take.
use rondata::checksum::CHECKSUM_LINES;

fn run() -> Result<bool, String> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let [install, recording, trace] = args.as_slice() else {
        return Err("usage: score_recording INSTALL GAME.rcx rontrace.log".into());
    };
    let score =
        rondata::checksum::playback::compare(install.as_ref(), recording.as_ref(), trace.as_ref())?;
    for (i, first) in score.first.iter().enumerate() {
        print!(
            "{} {}/{}",
            CHECKSUM_LINES[i].0, score.agreed[i], score.states
        );
        if let Some((f, ours, theirs)) = first {
            let ours = ours.map_or_else(|| "unemitted".to_owned(), |v| v.to_string());
            print!(", first differing state {f}: ours {ours}, original {theirs}");
        }
        println!();
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
