//! Observe actual correction writes, paired with an unaudited control replay.
use rondata::{Install, capture::indexed::IndexedCapture, diff, gamelog::Initial};

fn with_siblings<R>(
    sources: &mut [IndexedCapture],
    refs: &[&Initial<'_>],
    f: impl FnOnce(&[&Initial<'_>]) -> R,
) -> std::io::Result<R> {
    let Some((source, rest)) = sources.split_first_mut() else {
        return Ok(f(refs));
    };
    source.with_replay_initial(|init| {
        let mut next = refs.to_vec();
        next.push(&init);
        with_siblings(rest, &next, f)
    })?
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if cfg!(debug_assertions) {
        return Err("use --release".into());
    }
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() < 3 {
        return Err(
            "INSTALL CAPTURE RECORD_COUNT [--sibling CAPTURE] [--trace TRACE] [--recording REC]"
                .into(),
        );
    }
    let count: usize = args[2].parse()?;
    let loaded = rondata::load::load(&Install::new(&args[0]))?;
    let mut source = IndexedCapture::open(&args[1])?;
    if count == 0 || count > source.frames().len() {
        return Err("record count must be within the source capture".into());
    }
    let mut siblings = Vec::new();
    let mut trace = None;
    let mut recording = None;
    let mut i = 3;
    while i < args.len() {
        let value = args.get(i + 1).ok_or("option needs a value")?;
        match args[i].as_str() {
            "--sibling" => siblings.push(IndexedCapture::open(value)?),
            "--trace" => {
                trace = Some(
                    rondata::trace::Trace::read(std::path::Path::new(value))?
                        .ok_or("trace header missing")?,
                )
            }
            "--recording" => recording = Some(rondata::recgame::read(value)?),
            _ => return Err("unknown option".into()),
        }
        i += 2;
    }
    let mut control = with_siblings(&mut siblings, &[], |refs| {
        source.with_replay_initial(|init| {
            let mut init = init;
            diff::borrow_from_siblings(&mut init, refs);
            if let Some(trace) = &trace {
                diff::borrow_pasture(&mut init, trace);
            }
            diff::ReplaySession::new(
                &loaded,
                &init,
                sim::Tuning::RON,
                recording.as_ref().map(rondata::input::Stream::new),
            )
        })
    })??;
    let mut observed = control.clone();
    observed.enable_correction_audit();
    for index in 0..count {
        let frame = source.frame_state(index)?;
        if control.push(index, &frame)? != observed.push(index, &frame)? {
            return Err(format!("audit changed comparator at source index {index}").into());
        }
    }
    let mut normalized = observed.built().clone();
    let audit = normalized.correction_audit.take().ok_or("audit missing")?;
    if format!("{normalized:?}") != format!("{:?}", control.built())
        || observed.finish() != control.finish()
    {
        return Err("audit changed final state or complete report".into());
    }
    source.validate()?;
    for sibling in &siblings {
        sibling.validate()?;
    }
    println!(
        "source: {}; records: {count}; last source frame: {}; paired control: equal",
        args[1],
        source.frames()[count - 1].number
    );
    println!("{audit:#?}");
    Ok(())
}
