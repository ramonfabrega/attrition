//! Original game ranges for the window-input combat recordings (RECGAME §10).
use std::path::PathBuf;

fn lab() -> PathBuf {
    PathBuf::from(std::env::var_os("HOME").unwrap_or_default())
        .join("ron-data/lab-experiments/2026-11-18-codex-combat-replays")
}

#[test]
fn both_occupied_sprite_pools_match_every_record_in_window_a() {
    let Some(root) = crate::testenv::install_root() else {
        return;
    };
    let dir = lab();
    let game = crate::testenv::experiment(&dir, "window-a.rcx");
    let ranges = crate::testenv::experiment(&dir, "oracle-a/map-14/rontrace.ranges");
    let (Some(game), Some(ranges)) = (game, ranges) else {
        return;
    };
    let rec = crate::recgame::read(&game.to_string_lossy()).unwrap();
    let loaded = crate::load::load(&crate::Install::new(root)).unwrap();
    let mut built = super::recording_start(&loaded, &rec);
    let mut commands = crate::input::Stream::pump(&rec);
    let mut original = crate::trace::ranges::Record::open(&ranges).unwrap();
    assert!(!original.truncated);
    assert_eq!(original.mask & 0x18, 0x18);
    let last = rec.packages.last().unwrap().frame;
    assert_eq!(last, 11720);
    let mut occupied = [0; 2];
    for frame in 0..=last {
        let set = original.first_of(i64::from(frame) - 1).unwrap().clone();
        assert_eq!(set.flags, 0, "original overflow at {frame}");
        let rows = original.ranges(&set).unwrap();
        for (k, line) in [built.sim.checksum_ammo(), built.sim.checksum_deaths()]
            .into_iter()
            .enumerate()
        {
            let index = k + 3;
            let line = line.unwrap_or_else(|| panic!("unemitted line {index} at {frame}"));
            let theirs: Vec<_> = rows
                .iter()
                .filter(|r| usize::from(r.line) == index)
                .collect();
            assert_eq!(
                line.ranges.len(),
                theirs.len(),
                "{} count at {frame}",
                line.name
            );
            occupied[k] += usize::from(!theirs.is_empty());
            for (a, b) in line.ranges.iter().zip(theirs) {
                assert_eq!(
                    a.bytes, b.bytes,
                    "{} record {} {} at {frame}",
                    line.name, a.record, a.field
                );
            }
            assert_eq!(line.adler(), set.lines[index], "{} at {frame}", line.name);
        }
        if frame < last {
            commands.apply(frame, &mut built);
            built.sim.command().tick();
        }
    }
    assert_eq!(occupied, [848, 1152]);
}
