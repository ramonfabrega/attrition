//! The two-byte list slot in BuildData, checked in its whole original range.
use super::{build_site_field, packet_stream, packets_dir};
use std::{collections::BTreeMap, path::Path};

/// Key each 22-byte building body by the owner and object in its own walk.
fn bodies<'a>(rows: impl Iterator<Item = (&'a str, &'a [u8])>) -> BTreeMap<(u8, i16), &'a [u8]> {
    let mut result = BTreeMap::new();
    let mut key = None;
    for (field, bytes) in rows {
        match field {
            "who..y_internal" => {
                assert!(key.is_none());
                key = Some((bytes[0], i16::from_le_bytes([bytes[1], bytes[2]])));
            }
            "gather_down..infiltrate2" => {
                assert_eq!(bytes.len(), 22);
                assert!(result.insert(key.take().unwrap(), bytes).is_none());
            }
            _ => {}
        }
    }
    assert!(key.is_none());
    result
}

#[test]
fn building_slots_match_the_original_build_data() {
    use crate::diff::{
        american_industrial_pair::AMERICAN_INDUSTRIAL, british_classical_pair::BRITISH_CLASSICAL,
        british_enlightenment_pair::BRITISH_ENLIGHTENMENT, russian_modern_pair::RUSSIAN_MODERN,
    };

    for (game, run, frame, slots) in [
        (BRITISH_CLASSICAL, "run1326", 4567, &[(2017, 0)][..]),
        (
            BRITISH_ENLIGHTENMENT,
            "run1185",
            4582,
            &[(2031, 0), (2018, 0), (2020, 1), (2019, 2)][..],
        ),
        (
            AMERICAN_INDUSTRIAL,
            "run1359",
            5619,
            &[
                (2007, 0),
                (2014, 1),
                (2016, 2),
                (2035, 3),
                (2045, 4),
                (2034, 0),
                (2050, 1),
                (2061, 2),
                (2067, -1),
            ][..],
        ),
        (
            RUSSIAN_MODERN,
            "run1165",
            3719,
            &[(2029, 1), (2031, 2), (2044, 3)][..],
        ),
    ] {
        let dir = Path::new(&packets_dir()).join(run);
        if crate::testenv::experiment(&dir, "stream.tsv").is_none()
            || crate::testenv::experiment(&dir, "values.tsv").is_none()
        {
            return;
        }
        let (logged, original) = packet_stream(run).expect("the kept packet reads");
        assert_eq!(logged, frame, "{run}");
        let Some((built, _)) = crate::diff::third::stand_up_built(game.long) else {
            return;
        };
        let mut rider = crate::diff::harness::trunk::Rider::new(built);
        rider.advance(frame);
        // The graphic piece goes into WallData; these records are BuildData.
        let emitted = rider.state().sim.checksum_builds(&|_| -1);
        let ours = bodies(emitted.ranges.iter().map(|r| (r.field, r.bytes.as_slice())));
        let theirs = bodies(
            original["builds"]
                .iter()
                .map(|(_, site, bytes)| (build_site_field(site), bytes.as_slice())),
        );
        for &(object, slot) in slots {
            let native = theirs[&(1, object)];
            assert_eq!(
                i16::from_le_bytes([native[8], native[9]]),
                slot,
                "{run} object {object}"
            );
            assert_eq!(
                ours[&(1, object)],
                native,
                "{run} player 1 object {object}, slot {slot}"
            );
        }
    }
}
