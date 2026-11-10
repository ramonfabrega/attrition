use super::*;

/// A 67-line record's text: `CHECKSUM n`, `FILE`, `LINE`, then every label
/// of [`RECORD_LABELS`] with value `base + i`, the quiet four at 1.
fn record_text(n: u64, file: &str, base: u32, level16: bool) -> String {
    let mut s = format!("  CHECKSUM {n}\n      FILE {file}\n      LINE 135\n");
    let labels = if level16 { 64 } else { 29 };
    for (i, label) in RECORD_LABELS.iter().take(labels).enumerate() {
        let v = if ["searcher", "scene", "message_win", "storm"].contains(label) {
            1
        } else {
            base + i as u32
        };
        s.push_str(&format!("  {label} {v}\r\n"));
    }
    s
}

/// **A record is read whole, and placed by the hook that printed it**: a
/// begin-game record, a frame's begin record (first child of its block),
/// its end record (beside the block), and a setup record at level 14 whose
/// next line looks like a label but is not the record's.
#[test]
fn a_record_is_read_whole_and_placed_by_its_hook() {
    let text = [
        record_text(0, "map.cpp", 10, false),
        "  tribes 7\n".to_string(),
        "BEGIN GAME\n BEGIN FULL DUMP\n".to_string(),
        record_text(1, "gamelog.cpp", 100, true),
        " END FULL DUMP\n BEGIN FRAME 7\n  BEGIN FULL DUMP\n".to_string(),
        record_text(2, "gamelog.cpp", 200, true),
        "   BEGIN COMMANDMANAGER\n   END COMMANDMANAGER\n  END FULL DUMP\n BEGIN FULL DUMP\n"
            .to_string(),
        record_text(3, "gamelog.cpp", 300, true),
        " END FULL DUMP\n".to_string(),
    ]
    .concat();
    let recs = records(text.as_bytes()).unwrap();
    let got: Vec<_> = recs
        .iter()
        .map(|r| (r.number, r.hook, r.values.len()))
        .collect();
    assert_eq!(
        got,
        [
            (0, Hook::Setup, 29),
            (1, Hook::Game, 64),
            (2, Hook::Begin(7), 64),
            (3, Hook::End(7), 64),
        ]
    );
    let (begin, end) = frame(&recs, 7);
    let (begin, end) = (begin.unwrap(), end.unwrap());
    assert_eq!(begin.get("Units"), Some(225), "Units is the 26th label");
    assert_eq!(
        end.get("World"),
        Some(322),
        "World is the sixteenth World, not the first"
    );
    assert_eq!(end.world_parts().len(), 16);
    assert_eq!(end.get("game_random seed"), Some(328));
    assert_eq!(end.get("turn_control"), Some(363));
    assert_eq!(end.get("searcher"), Some(1));
    assert_eq!(recs[0].get("Rules"), Some(37));
    assert_eq!(
        recs[0].get("tribes"),
        None,
        "a label out of print order is not the record's"
    );
    assert_eq!(
        end.fifteen().unwrap()[0],
        325,
        "check_all's first line is units"
    );
}

/// **The instrument's comparison can part**: a record whose one line is
/// changed parts on that line and no other.
#[test]
fn a_changed_line_parts_from_the_instrument() {
    let text = record_text(5, "gamelog.cpp", 1000, true);
    let rec = &records(text.as_bytes()).unwrap()[0];
    let mut values: Vec<(String, u32)> = CHECKSUM_LINES
        .iter()
        .map(|(n, l, _)| (n.to_string(), rec.get(l).unwrap()))
        .collect();
    assert!(parting(rec, &values).is_empty());
    values[7].1 ^= 1;
    assert_eq!(parting(rec, &values), [("leaders", 1003, 1002)]);
}

const RUN1100: &str = "gamelog-run1100-greatlakes-team-herd-packet-5408.txt";

fn run1100() -> Option<Vec<Record>> {
    let path = crate::testenv::dump(RUN1100)?;
    let file = std::fs::File::open(&path).expect("a kept dump opens");
    Some(records(std::io::BufReader::new(file)).expect("a kept dump reads"))
}

/// **run1100's two windowed records, read whole and checked against a
/// second reader** (`docs/CHECKSUM.md` §2): the `@dumpall` window holds
/// the end record of frame 5408 (3502) and the begin record of 5409
/// (3503), each 64 labelled lines. The quiet lines read 1, the busy ones
/// do not — a reader that defaults to 1 would pass the first half and
/// fail the second — and the begin record's seed is the one
/// `gamelog::Log::frame_seeds`, the block-tree reader, takes from it.
#[test]
fn run1100_s_records_are_read_whole() {
    let Some(recs) = run1100() else {
        eprintln!("skipping: no {RUN1100} (set RON_GAMELOG_DIR)");
        return;
    };
    let windowed: Vec<_> = recs
        .iter()
        .filter(|r| r.file == "gamelog.cpp")
        .map(|r| (r.number, r.hook, r.values.len()))
        .collect();
    assert_eq!(
        windowed,
        [(3502, Hook::End(5408), 64), (3503, Hook::Begin(5409), 64)]
    );
    let (_, end) = frame(&recs, 5408);
    let end = end.unwrap();
    for quiet in [
        "searcher",
        "scene",
        "message_win",
        "storm",
        "Ammo",
        "Walls",
        "Deaths",
    ] {
        assert_eq!(end.get(quiet), Some(1), "{quiet} is quiet on 5408");
    }
    for busy in [
        "Units",
        "Builds",
        "Guys",
        "Leaders",
        "World",
        "Rules",
        "turn_control",
    ] {
        assert!(
            end.get(busy).is_some_and(|v| v > 1),
            "{busy} is busy on 5408"
        );
    }
    let text = std::fs::read_to_string(crate::testenv::dump(RUN1100).unwrap()).unwrap();
    let log = crate::gamelog::Log::parse(&text);
    let (begin, _) = frame(&recs, 5409);
    let seed = begin.unwrap().get("game_random seed").unwrap();
    assert!(
        log.frame_seeds().contains(&(5408, seed)),
        "frame_seeds reads 5409's begin record as engine frame 5408's seed {seed}: {:?}",
        log.frame_seeds()
    );
}

/// The instrument's values on run1100's packet, if this machine has them
/// (`tools/recomp/checksum.py --values`, outside git).
fn instrument(tag: &str) -> Option<Vec<(String, u32)>> {
    let dir = std::env::var("RON_CHECKSUM_DIR").unwrap_or_else(|_| {
        let home = std::env::var("HOME").unwrap_or_default();
        format!("{home}/ron-data/lab-experiments/2026-11-10-item-4060/run1100")
    });
    let text = std::fs::read_to_string(format!("{dir}/values-{tag}.tsv")).ok()?;
    Some(instrument_values(&text))
}

/// **The instrument reproduces the printed record, and a frame it is not
/// of parts** (`docs/CHECKSUM.md` §3): the fifteen sums
/// `tools/recomp/checksum.py` reads off run1100's packet (logger frame
/// 5408, `GameLog::end_frame` entry) equal record 3502 on
/// [`CHECKSUM_REPRODUCED`]'s count of lines, and part from 3503 — the
/// begin record of 5409, one tick later — on the six lines that tick
/// moves. That second half is the falsifier, red on purpose.
#[test]
fn the_instrument_reproduces_the_printed_record() {
    let (Some(recs), Some(values)) = (run1100(), instrument("3502")) else {
        eprintln!("skipping: no {RUN1100} or no instrument values (RON_CHECKSUM_DIR)");
        return;
    };
    assert_eq!(values.len(), 15);
    let (_, end) = frame(&recs, CHECKSUM_PACKET.1);
    let end = end.unwrap();
    assert_eq!(end.number, CHECKSUM_PACKET.2);
    let parted = parting(end, &values);
    assert_eq!(
        (15 - parted.len(), 15),
        CHECKSUM_REPRODUCED,
        "the instrument parts from the printed record: {parted:?}"
    );
    let (next, _) = frame(&recs, CHECKSUM_PACKET.1 + 1);
    let parted: Vec<_> = parting(next.unwrap(), &values)
        .into_iter()
        .map(|(n, _, _)| n)
        .collect();
    assert_eq!(
        parted,
        ["units", "builds", "guys", "leaders", "goods", "world"]
    );
}
