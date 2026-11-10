use super::*;

/// zlib's own values: the empty sum is the seed, and "Wikipedia" is the
/// textbook 0x11E60398. And the sums of zeros the record prints for an
/// empty array (`docs/CHECKSUM.md` §8): 4 zero bytes are 262145, 8 are
/// 524289.
#[test]
fn adler32_is_zlib_s() {
    assert_eq!(adler32(1, b""), 1);
    assert_eq!(adler32(1, b"Wikipedia"), 0x11E6_0398);
    assert_eq!(adler32(1, &[0; 4]), 262_145);
    assert_eq!(adler32(1, &[0; 8]), 524_289);
    // Carried across ranges: the same bytes in two pieces.
    assert_eq!(adler32(adler32(1, b"Wiki"), b"pedia"), 0x11E6_0398);
}

/// One box makes five ranges and 22 bytes, its coordinates under the mask;
/// a taken box is not walked.
#[test]
fn an_item_is_five_ranges_and_twenty_two_bytes() {
    let mut sim = crate::Sim::new(crate::Tuning::RON, crate::world::World::new(60, 60), 2);
    sim.setup()
        .add_goody_item(crate::world::Pos::new(36480, 22656), true, 4);
    sim.setup()
        .add_goody_item(crate::world::Pos::new(1152, 21120), false, 0);
    let line = sim.checksum_items();
    assert_eq!(line.ranges.len(), 5);
    assert_eq!(line.bytes(), 22);
    let f: Vec<_> = line.ranges.iter().map(|r| r.field).collect();
    assert_eq!(
        f,
        ["ever_seen", "flags", "must_walk", "who..y_internal", "type"]
    );
    assert_eq!(line.ranges[0].bytes, [4]);
    let sub = &line.ranges[3].bytes;
    assert_eq!(sub[..3], [0xff, 0, 0]);
    let x = i32::from_le_bytes(sub[7..11].try_into().unwrap()) ^ COORD_MASK;
    assert_eq!(x, 36480);
    assert_eq!(line.ranges[4].bytes, 0x21f_i32.to_le_bytes());
    assert_eq!(Line::default().adler(), 1);
}

/// A good is the items record over the goods array: five ranges, 22
/// bytes, its own type as the local, and `ever_seen` the bits
/// `mark_good_seen` set; a dead good is not walked.
#[test]
fn a_good_is_the_item_record_with_its_own_type_and_seen_bits() {
    let mut sim = crate::Sim::new(crate::Tuning::RON, crate::world::World::new(60, 60), 2);
    let at = crate::world::Pos::new(40320, 14208);
    sim.world.add_good(crate::world::Good {
        pos: at,
        ty: 17,
        alive: true,
    });
    sim.world.add_good(crate::world::Good {
        pos: crate::world::Pos::new(1152, 1152),
        ty: 5,
        alive: false,
    });
    assert_eq!(sim.checksum_goods().ranges[0].bytes, [0]);
    sim.mark_good_seen(0, 1);
    let line = sim.checksum_goods();
    assert_eq!(line.ranges.len(), 5);
    assert_eq!(line.bytes(), 22);
    assert_eq!(line.ranges[0].bytes, [2]);
    assert_eq!(line.ranges[4].bytes, 17_i32.to_le_bytes());
    let sub = &line.ranges[3].bytes;
    let y = i32::from_le_bytes(sub[11..15].try_into().unwrap()) ^ COORD_MASK;
    assert_eq!(y, 14208);
}

/// The city record's table covers `+0x6..+0x72` exactly, in order, with
/// no gap and no overlap — the 108 bytes `check_cities` walks.
#[test]
fn the_city_table_tiles_the_walked_108_bytes() {
    let mut at = 0x6;
    for &(name, off, width) in CITY_FIELDS {
        assert_eq!(off, at, "{name} starts where the last field ended");
        at += width;
    }
    assert_eq!(at, 0x72);
    assert_eq!(
        crate::Sim::new(crate::Tuning::RON, crate::world::World::new(8, 8), 2)
            .checksum_cities()
            .ranges
            .len(),
        0
    );
}
