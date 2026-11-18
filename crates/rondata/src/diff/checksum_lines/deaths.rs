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

fn casualty_packet(run: &str) -> Option<super::StreamRows> {
    let dir = std::env::var_os("RON_PACKETS_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::path::PathBuf::from(std::env::var_os("HOME").unwrap_or_default())
                .join("ron-data/lab-experiments/2026-11-12-item-4188")
        });
    let path = crate::testenv::experiment(&dir.join(run), "stream.tsv")?;
    let mut rows = stream_lines(&std::fs::read_to_string(path).unwrap(), |name| {
        name == "deaths"
    });
    Some(rows.remove("deaths").expect("occupied casualty packet"))
}

#[test]
fn the_combat_packet_keeps_the_hit_direction_without_an_attachment() {
    use crate::diff::german_modern_pair::GERMAN_MODERN;
    let Some(theirs) = casualty_packet("run1420") else {
        return;
    };

    let Some((mut built, _)) = crate::diff::third::stand_up_built(GERMAN_MODERN.long) else {
        return;
    };
    while built.sim.frame < 4586 {
        built.tick();
    }
    let actual = built.sim.checksum_deaths().unwrap();
    assert_eq!(
        actual.ranges.iter().map(|r| &r.bytes).collect::<Vec<_>>(),
        theirs.iter().map(|r| &r.2).collect::<Vec<_>>()
    );
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
    for (i, (ours, (_, _, original))) in trial.ranges.iter().zip(&theirs).enumerate() {
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

#[test]
fn the_mongol_packet_keeps_the_attached_piece_and_relative_direction() {
    use crate::diff::mongol_enlightenment_pair::MONGOL_ENLIGHTENMENT;
    use sim::num::single::Single;
    let Some(theirs) = casualty_packet("run1361") else {
        return;
    };
    let Some((mut built, _)) = crate::diff::third::stand_up_built(MONGOL_ENLIGHTENMENT.long) else {
        return;
    };
    while built.sim.frame < 4840 {
        built.tick();
    }
    let actual = built.sim.checksum_deaths().unwrap();
    assert_eq!((actual.ranges.len(), actual.bytes()), (4, 150));
    assert_eq!(
        actual.ranges.iter().map(|r| &r.bytes).collect::<Vec<_>>(),
        theirs.iter().map(|r| &r.2).collect::<Vec<_>>()
    );
    let impacts = built
        .sim
        .combat
        .deaths
        .iter()
        .map(|d| (d.o, d.impact))
        .collect::<Vec<_>>();
    assert_eq!(
        impacts,
        [
            (8, Some((60157, Single::from_i32(343)))),
            (7, Some((-1, Single::ZERO)))
        ]
    );
}

#[test]
fn the_british_packet_turns_both_fallen_figures_before_measuring_the_hit() {
    use crate::diff::british_enlightenment_pair::BRITISH_ENLIGHTENMENT;
    let Some(theirs) = casualty_packet("run1185") else {
        return;
    };
    let Some((mut built, _)) = crate::diff::third::stand_up_built(BRITISH_ENLIGHTENMENT.long)
    else {
        return;
    };
    while built.sim.frame < 4582 {
        built.tick();
    }
    let actual = built.sim.checksum_deaths().unwrap();
    assert_eq!((actual.ranges.len(), actual.bytes()), (4, 150));
    assert_eq!(
        actual.ranges.iter().map(|r| &r.bytes).collect::<Vec<_>>(),
        theirs.iter().map(|r| &r.2).collect::<Vec<_>>()
    );
    let poses = built
        .sim
        .combat
        .deaths
        .iter()
        .map(|d| (d.figures.place[0].angle.0, d.impact))
        .collect::<Vec<_>>();
    use sim::num::single::Single;
    assert_eq!(
        poses,
        [
            (1_514_733_568, Some((-1, Single::from_i32(284)))),
            (221_642_752, Some((-1, Single::from_i32(46))))
        ]
    );
}

#[test]
fn the_sea_packet_keeps_the_attrition_figure_alongside_three_round_impacts() {
    use crate::diff::sea_melee::{MELEE_LONG, MELEE_START, melee_script};
    let Some(theirs) = casualty_packet("run1487") else {
        return;
    };
    crate::diff::setup::with_staged_script(melee_script(), || {
        let Some((mut built, _)) = crate::diff::third::stand_up_from(MELEE_START, MELEE_LONG)
        else {
            return;
        };
        while built.sim.frame < 1980 {
            built.tick();
        }
        let actual = built.sim.checksum_deaths().unwrap();
        assert_eq!((actual.ranges.len(), actual.bytes()), (8, 300));
        // Slot three holds the figure lost to attrition on 1822.
        assert_eq!(actual.ranges[7].bytes, theirs[7].2);
        let d = built
            .sim
            .combat
            .deaths
            .iter()
            .find(|d| d.who == 0 && d.o == 14)
            .unwrap();
        assert_eq!(d.first_frame, 1822);
        assert_eq!(d.impact, Some((-1, sim::num::single::Single::ZERO)));
    });
}
