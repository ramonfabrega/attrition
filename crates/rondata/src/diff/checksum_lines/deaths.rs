//! The first occupied pool of the long window recording, against the
//! original's own checksum stream on the same frame (CHECKSUM §8).
use super::stream_lines;
use crate::checksum::playback::recording_start;

#[test]
fn the_disbanded_scout_and_dog_match_the_whole_packet() {
    let Some(root) = crate::testenv::install_root() else {
        return;
    };
    let lab = std::path::PathBuf::from(std::env::var_os("HOME").unwrap_or_default())
        .join("ron-data/lab-experiments");
    let game = crate::testenv::experiment(
        &lab.join("2026-11-17-codex-live-recording"),
        "window-long.rcx",
    );
    let stream =
        crate::testenv::experiment(&lab.join("2026-11-17-codex-scout-deaths"), "stream.tsv");
    let (Some(game), Some(stream)) = (game, stream) else {
        return;
    };
    let rows = stream_lines(&std::fs::read_to_string(stream).unwrap(), |name| {
        name == "deaths"
    });
    let theirs: Vec<_> = rows["deaths"].iter().map(|r| r.2.as_slice()).collect();
    assert_eq!(
        theirs.iter().map(|r| r.len()).collect::<Vec<_>>(),
        [4, 71, 4, 71]
    );

    let rec = crate::recgame::read(&game.to_string_lossy()).unwrap();
    let loaded = crate::load::load(&crate::Install::new(root)).unwrap();
    let mut built = recording_start(&loaded, &rec);
    let mut commands = crate::input::Stream::pump(&rec);
    while built.sim.frame < 8408 {
        commands.apply(built.sim.frame as i32, &mut built);
        built.sim.command().tick();
    }
    let figures = &built.sim.combat.deaths[0].figures;
    assert_eq!(&figures.gpiece[..usize::from(figures.n)], &[371, 13043]);
    assert_eq!(&figures.life[..usize::from(figures.n)], &[687, 160]);
    let line = built.sim.checksum_deaths().unwrap();
    assert_eq!(
        line.ranges
            .iter()
            .map(|r| r.bytes.as_slice())
            .collect::<Vec<_>>(),
        theirs
    );
    assert_eq!(line.adler(), 826547225);

    // Reverse slots without changing any field. Both the range ordering
    // and Adler must now distinguish the two figures of the same unit.
    built.sim.combat.death_pool.swap(0, 1);
    let swapped = built.sim.checksum_deaths().unwrap();
    assert_eq!(swapped.ranges[1].bytes, theirs[3]);
    assert_eq!(swapped.ranges[3].bytes, theirs[1]);
    assert_ne!(swapped.adler(), line.adler());

    built.sim.combat.death_pool.swap(0, 1);
    built.sim.combat.deaths[0].figures.place[1].pos.x += 1;
    let moved = built.sim.checksum_deaths().unwrap();
    assert_eq!(moved.ranges[..3], line.ranges[..3]);
    assert_ne!(moved.ranges[3].bytes, theirs[3]);
    assert_ne!(moved.adler(), line.adler());

    built.sim.combat.deaths[0].impact = None;
    assert!(built.sim.checksum_deaths().is_none());
}

/// A different game's four combat deaths. A missing hit direction must keep
/// the line unknown even if the attacking piece is absent in every record.
#[test]
fn the_combat_packet_distinguishes_an_unknown_hit_from_a_disband() {
    use crate::diff::german_modern_pair::GERMAN_MODERN;
    let dir = std::env::var_os("RON_PACKETS_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::path::PathBuf::from(std::env::var_os("HOME").unwrap_or_default())
                .join("ron-data/lab-experiments/2026-11-12-item-4188")
        });
    let Some(path) = crate::testenv::experiment(&dir.join("run1420"), "stream.tsv") else {
        return;
    };
    let rows = stream_lines(&std::fs::read_to_string(path).unwrap(), |name| {
        name == "deaths"
    });
    let theirs = &rows["deaths"];
    let Some((mut built, _)) = crate::diff::third::stand_up_built(GERMAN_MODERN.long) else {
        return;
    };
    while built.sim.frame < 4586 {
        built.tick();
    }
    assert!(built.sim.checksum_deaths().is_none());
    assert_eq!(built.sim.combat.deaths.len(), 4);
    // Falsify extending the disband's impact to every death. This probe
    // operates only after the replay has reached the packet.
    for d in &mut built.sim.combat.deaths {
        d.impact = Some((-1, sim::num::single::Single::ZERO));
    }
    let trial = built.sim.checksum_deaths().unwrap();
    assert_eq!((trial.ranges.len(), trial.bytes()), (8, 300));
    assert_eq!(theirs.len(), trial.ranges.len());
    let mut differences = Vec::new();
    for (i, (ours, (_, _, original))) in trial.ranges.iter().zip(theirs).enumerate() {
        assert_eq!(ours.bytes.len(), original.len());
        for (j, (a, b)) in ours.bytes.iter().zip(original).enumerate() {
            if a != b {
                differences.push((i, j));
            }
        }
    }
    // Only the high bytes of ammo_angle in the first three bodies differ.
    // The fourth hit really had zero direction. Every other byte agrees.
    assert_eq!(
        differences,
        [(1, 38), (1, 39), (3, 38), (3, 39), (5, 38), (5, 39)]
    );
}
