//! Field decoders shared by packet tests and recorded-game diagnostics.

use sim::checksum::COORD_MASK;

/// One range of `line` decoded into its fields, where the line has a
/// decoder for the range's `field` (item 4214's minted packets name a
/// parting by field through it). `None` for a line with none.
pub(crate) fn range_fields(line: &str, field: &str, b: &[u8]) -> Option<Vec<(String, i64)>> {
    if b.is_empty() || (b.len() == 3 && !(line == "groups" && matches!(field, "list" | "angles"))) {
        return None;
    }
    match line {
        "units" => Some(unit_range_fields(field, b)),
        "builds" => Some(build_range_fields(field, b)),
        "guys" => Some(guy_range_fields(field, b)),
        "groups" if b.len() % 4 != 3 || matches!(field, "list" | "angles") => {
            Some(group_range_fields(field, b))
        }
        "cities" if field == "city..ter" => Some(
            city_fields(b)
                .into_iter()
                .map(|(n, v)| (n.to_string(), v))
                .collect(),
        ),
        _ => None,
    }
}

/// Decode equal-length native and simulated ranges under the simulated range's schema.
/// A different size or an unknown layout is left to the byte comparison.
pub(crate) fn differing_fields(
    line: &str,
    field: &str,
    ours: &[u8],
    original: &[u8],
) -> Vec<(String, i64, i64)> {
    if ours.len() == original.len()
        && let (Some(a), Some(b)) = (
            range_fields(line, field, ours),
            range_fields(line, field, original),
        )
    {
        return a
            .into_iter()
            .zip(b)
            .filter_map(|((name, x), (_, y))| (x != y).then_some((name, x, y)))
            .collect();
    }
    Vec::new()
}

/// One city's 108 bytes decoded by the type record's table
/// ([`sim::checksum::CITY_FIELDS`]): `who`, `race` and `founder` are
/// `char`s, `traded_with` `int`s read unsigned, every other byte a
/// `uchar` and every wider field signed.
pub(crate) fn city_fields(b: &[u8]) -> Vec<(&'static str, i64)> {
    sim::checksum::CITY_FIELDS
        .iter()
        .map(|&(name, at, width)| {
            let at = at - 6;
            let v = match width {
                1 if matches!(name, "who" | "race" | "founder") => i64::from(b[at] as i8),
                1 => i64::from(b[at]),
                2 => i64::from(i16::from_le_bytes([b[at], b[at + 1]])),
                _ if name.starts_with("traded_with") => {
                    i64::from(u32::from_le_bytes(b[at..at + 4].try_into().unwrap()))
                }
                _ => i64::from(i32::from_le_bytes(b[at..at + 4].try_into().unwrap())),
            };
            (name, v)
        })
        .collect()
}

/// One fixed-width field of a builds record, decoded: `char`s and the
/// named signed bytes signed, every other byte unsigned, `ulong`s and
/// `ushort`s unsigned, every other word signed.
fn build_field(name: &str, b: &[u8], at: usize, width: usize) -> i64 {
    let signed_byte = matches!(
        name,
        "infiltrated"
            | "damage_frac"
            | "mylos"
            | "targeted"
            | "inside_down_who"
            | "up_who"
            | "visible"
            | "launch_frames"
            | "stance"
            | "founder"
            | "gather_max"
            | "attack_whom"
    );
    match width {
        1 if signed_byte => i64::from(b[at] as i8),
        1 => i64::from(b[at]),
        2 if matches!(name, "uid" | "hold_frames") => {
            i64::from(u16::from_le_bytes([b[at], b[at + 1]]))
        }
        2 => i64::from(i16::from_le_bytes([b[at], b[at + 1]])),
        _ if matches!(name, "job_counter" | "job_counter_2" | "constr_time") && at >= 0x48 => {
            i64::from(u32::from_le_bytes(b[at..at + 4].try_into().unwrap()))
        }
        _ => i64::from(i32::from_le_bytes(b[at..at + 4].try_into().unwrap())),
    }
}

/// A builds range decoded field by field: by the type record's table where
/// the range is one, else as its local (`(name, value)` pairs).
pub(crate) fn build_range_fields(field: &str, b: &[u8]) -> Vec<(String, i64)> {
    use sim::checksum::builds::{BUILD_FIELDS, OBJECT_FIELDS, QUEUE_ITEM_FIELDS, WALL_FIELDS};
    let table = |t: &[(&'static str, usize, usize)], base: usize| -> Vec<(String, i64)> {
        t.iter()
            .map(|&(n, at, w)| (n.to_string(), build_field(n, b, at - base, w)))
            .collect()
    };
    match field {
        "myhits..launch_frames" => table(OBJECT_FIELDS, 0x20),
        "job_counter..demolition" => table(WALL_FIELDS, 0x48),
        "gather_down..infiltrate2" => table(BUILD_FIELDS, 0x70),
        "queue[k]" => table(QUEUE_ITEM_FIELDS, 0),
        "who..y_internal" => {
            let w = |at: usize| {
                i64::from(i32::from_le_bytes(b[at..at + 4].try_into().unwrap()) ^ COORD_MASK)
            };
            vec![
                ("who".into(), i64::from(b[0])),
                ("o".into(), i64::from(i16::from_le_bytes([b[1], b[2]]))),
                ("z".into(), w(3)),
                ("x".into(), w(7)),
                ("y".into(), w(11)),
            ]
        }
        "mtn..cliff" => vec![
            ("mtn".into(), i64::from(b[0] as i8)),
            ("cliff".into(), i64::from(b[1] as i8)),
        ],
        "gather_from[k]" => vec![
            (
                "tx".into(),
                i64::from(i32::from_le_bytes(b[0..4].try_into().unwrap())),
            ),
            (
                "ty".into(),
                i64::from(i32::from_le_bytes(b[4..8].try_into().unwrap())),
            ),
        ],
        _ => {
            let v = match b.len() {
                1 => i64::from(b[0]),
                2 => i64::from(i16::from_le_bytes([b[0], b[1]])),
                _ => i64::from(i32::from_le_bytes(b[..4].try_into().unwrap())),
            };
            vec![(field.to_string(), v)]
        }
    }
}

/// One guys-line range decoded field by field: the figure record by
/// [`sim::checksum::guys::GUY_FIELDS`] (each field little-endian and
/// signed at its width, a turret array as its four words), any other
/// range as its one value.
pub(crate) fn guy_range_fields(field: &str, b: &[u8]) -> Vec<(String, i64)> {
    let le = |s: &[u8]| -> i64 {
        match s.len() {
            1 => i64::from(s[0] as i8),
            2 => i64::from(i16::from_le_bytes([s[0], s[1]])),
            _ => i64::from(i32::from_le_bytes(s[..4].try_into().unwrap())),
        }
    };
    if field == "guy" {
        let mut out = Vec::new();
        for &(n, at, w) in sim::checksum::guys::GUY_FIELDS {
            let s = &b[at - 8..at - 8 + w];
            if w == 16 {
                for k in 0..4 {
                    out.push((format!("{n}[{k}]"), le(&s[4 * k..4 * k + 4])));
                }
            } else {
                out.push((n.to_string(), le(s)));
            }
        }
        return out;
    }
    if field == "guys.size..increment" {
        return vec![
            ("size".into(), le(&b[0..4])),
            ("increment".into(), le(&b[4..6])),
        ];
    }
    vec![(field.to_string(), le(b))]
}

/// One units-line range decoded field by field: a record by its table
/// (each field little-endian and signed at its width), the sub-object's
/// `Coord`s unmasked, a figure by [`guy_range_fields`], anything else as
/// its one value.
pub(crate) fn unit_range_fields(field: &str, b: &[u8]) -> Vec<(String, i64)> {
    use sim::checksum::units::{
        CAST_FIELDS, GATHER_FIELDS, GROUP_ORDER_FIELDS, GUARD_FIELDS, MOVE_FIELDS, TARGET_FIELDS,
        TRADE_FIELDS, UNIT_FIELDS,
    };
    let le = |s: &[u8]| -> i64 {
        match s.len() {
            1 => i64::from(s[0] as i8),
            2 => i64::from(i16::from_le_bytes([s[0], s[1]])),
            _ => i64::from(i32::from_le_bytes(s[..4].try_into().unwrap())),
        }
    };
    let table = |t: &[(&str, usize, usize)], base: usize| -> Vec<(String, i64)> {
        t.iter()
            .map(|&(n, at, w)| (n.to_string(), le(&b[at - base..at - base + w])))
            .collect()
    };
    match field {
        "myhits..launch_frames" => table(sim::checksum::builds::OBJECT_FIELDS, 0x20),
        "collide_frame..play" => table(UNIT_FIELDS, 0x48),
        "order.move" => table(MOVE_FIELDS, 0x4),
        "order.target" => table(TARGET_FIELDS, 0x8),
        "order.gather" => table(GATHER_FIELDS, 0x14),
        "order.attack" => table(sim::checksum::units::ATTACK_FIELDS, 0x14),
        "order.trade" => table(TRADE_FIELDS, 0x14),
        "order.cast" => table(CAST_FIELDS, 0x14),
        "order.guard" => table(GUARD_FIELDS, 0x14),
        "order.attack_ground" => table(sim::checksum::units::ATTACK_GROUND_FIELDS, 0x4),
        "order.air" => table(sim::checksum::units::AIR_FIELDS, 0x4),
        "order.air_attack_ground" => table(sim::checksum::units::AIR_ATTACK_GROUND_FIELDS, 0x30),
        "order.group" => table(GROUP_ORDER_FIELDS, 0x54),
        "who..y_internal" => {
            let c = |at: usize| le(&b[at..at + 4]) ^ i64::from(COORD_MASK);
            vec![
                ("who".into(), i64::from(b[0])),
                ("o".into(), le(&b[1..3])),
                ("z".into(), c(3)),
                ("x".into(), c(7)),
                ("y".into(), c(11)),
            ]
        }
        "path.size..increment" => vec![
            ("path.size".into(), le(&b[0..4])),
            ("path.length".into(), le(&b[4..8])),
            ("path.increment".into(), le(&b[8..9])),
        ],
        "path[k]" => vec![
            ("to_x".into(), le(&b[0..4])),
            ("to_y".into(), le(&b[4..8])),
            ("path.tolerance".into(), le(&b[8..12])),
            ("path.flags".into(), le(&b[12..16])),
        ],
        "ox..aid" => vec![
            ("animal.ox".into(), le(&b[0..2])),
            ("animal.whom".into(), le(&b[2..4])),
            ("animal.aid".into(), le(&b[4..5])),
        ],
        "guy" | "guys.size..increment" => guy_range_fields(field, b),
        _ => vec![(field.to_string(), le(b))],
    }
}

/// One groups-line range decoded field by field: a record by
/// [`sim::checksum::groups::GROUP_RECORD_FIELDS`] (each little-endian and signed
/// at its width), a member array entry by entry, `last_group` as its word.
pub(crate) fn group_range_fields(field: &str, b: &[u8]) -> Vec<(String, i64)> {
    let le = |s: &[u8]| -> i64 {
        match s.len() {
            1 => i64::from(s[0] as i8),
            2 => i64::from(i16::from_le_bytes([s[0], s[1]])),
            _ => i64::from(i32::from_le_bytes(s[..4].try_into().unwrap())),
        }
    };
    let width = match field {
        "id..march" => {
            return sim::checksum::groups::GROUP_RECORD_FIELDS
                .iter()
                .map(|&(n, at, w)| (n.to_string(), le(&b[at - 4..at - 4 + w])))
                .collect();
        }
        "list" => 2,
        "angles" => 1,
        _ => 4,
    };
    b.chunks(width)
        .enumerate()
        .map(|(k, c)| (format!("{field}[{k}]"), le(c)))
        .collect()
}
