//! Owned source observations for civilian comparisons; never simulation results.
use super::{FrameGood, cycle_rows, frame_goods, raw_near, visible_rows};
use crate::capture::owned_codec::*;
use crate::diff::ammo::Ammo;
use crate::gamelog::Log;
use std::collections::BTreeMap;

#[derive(Debug, Default, PartialEq, Eq)]
pub(in crate::diff) struct Civilian {
    pub(super) near: BTreeMap<(i64, i64), (i64, i64)>,
    pub(super) cycles: Vec<((i64, i64), [i64; 3])>,
    pub(super) visible: Vec<((i64, i64), i64)>,
    pub(super) worlds: Vec<World>,
    pub(super) goods: BTreeMap<i64, FrameGood>,
    pub(super) ammo: Option<BTreeMap<(i64, i64, i64), Ammo>>,
}
#[derive(Debug, Default, PartialEq, Eq)]
pub(super) struct World {
    pub(super) fog: Vec<u8>,
    pub(super) detect: Vec<u8>,
    pub(super) danger: Vec<i64>,
}
impl Civilian {
    pub(in crate::diff) fn parse(raw: &str, log: &Log<'_>, include_ammo: bool) -> Self {
        Self {
            near: raw_near(raw),
            cycles: log
                .frames()
                .into_iter()
                .flat_map(|(_, b)| cycle_rows(b))
                .collect(),
            visible: log
                .frames()
                .into_iter()
                .flat_map(|(_, b)| visible_rows(b))
                .collect(),
            worlds: worlds(log),
            goods: frame_goods(raw).into_iter().map(|g| (g.o, g)).collect(),
            ammo: include_ammo.then(|| ammo(raw)),
        }
    }
}
pub(super) fn worlds(log: &Log<'_>) -> Vec<World> {
    log.frames()
        .into_iter()
        .filter_map(|(_, frame)| {
            let world = frame.find("WORLD")?;
            let fog = crate::gamelog::world_fog(&world.fields().to_vec());
            // The original comparison skips both following readers without fog.
            if fog.is_empty() {
                return None;
            }
            Some(World {
                fog,
                detect: crate::gamelog::world_fog3(&world.fields().to_vec()),
                danger: crate::gamelog::world_danger(&world.fields().to_vec()),
            })
        })
        .collect()
}
pub(super) fn ammo(raw: &str) -> BTreeMap<(i64, i64, i64), Ammo> {
    crate::diff::ammo::blocks(raw)
        .into_iter()
        .filter(|(a, _)| a.flags & 2 != 0)
        .map(|(a, _)| ((a.who, a.o, a.index), a))
        .collect()
}
record!(Civilian {
    near: BTreeMap<(i64, i64), (i64, i64)>,
    cycles: Vec<((i64, i64), [i64; 3])>,
    visible: Vec<((i64, i64), i64)>,
    worlds: Vec<World>,
    goods: BTreeMap<i64, FrameGood>,
    ammo: Option<BTreeMap<(i64, i64, i64), Ammo>>
});
record!(World { fog: Vec<u8>, detect: Vec<u8>, danger: Vec<i64> });
record!(FrameGood {
    name: String,
    ever_seen: i64,
    flags: i64,
    o: i64,
    x: i64,
    y: i64
});
record!(Ammo {
    cur_time: i64,
    total_time: i64,
    who: i64,
    o: i64,
    whom: i64,
    ox: i64,
    sx: i64,
    sy: i64,
    sz: i64,
    ex: i64,
    ey: i64,
    ez: i64,
    angle: i64,
    traj: i64,
    v1z: i64,
    dx: i64,
    start_roll_angle: i64,
    bank_dx: i64,
    bank_dy: i64,
    flags: i64,
    rolling: i64,
    gpiece: i64,
    graph_index: i64,
    splash_area: i64,
    index: i64,
    num_guys: i64,
    accuracy: i64
});

#[cfg(test)]
mod tests {
    use super::*;

    pub(in crate::diff) const AUTHORED: &str = "BEGIN GAME\n BEGIN FRAME 3\n  BEGIN UNITDATA\n   who 0\n   o 7\n   near_o bad\n   near_o 8\n   near_o 9\n   near_who 1\n  BEGIN BUILDDATA\n   recharging 4\n   attack_ox 6\n   attack_whom 1\n   BEGIN WALLDATA\n    BEGIN OBJECT\n     visible -1\n     BEGIN SUBOBJECT\n      who 0\n      o 12\n  BEGIN WORLD\n   seen2[scan] 257\n   seen2[scan] bad\n   seen3[scan] 255\n   danger[who][scan] -7\n   danger[who][scan] bad\n  BEGIN GOOD\n   authored\n   ever_seen 2\n   BEGIN SUBOBJECT\n    o 5\n    flags 1\n    x_internal 9\n    y_internal 10\n  BEGIN GOOD\n   replacement\n   ever_seen 3\n   BEGIN SUBOBJECT\n    o 5\n    x_internal 11\n  BEGIN AMMO\n   who 0\n   o 7\n   index 2\n   flags 2\n   accuracy 17\n  BEGIN AMMO\n   who 0\n   o 7\n   index 3\n   flags 0\n";

    #[test]
    fn civilian_observations_preserve_auxiliary_values_and_codec_boundaries() {
        let log = Log::parse(AUTHORED);
        let value = Civilian::parse(AUTHORED, &log, true);
        assert_eq!(value.near, BTreeMap::from([((0, 7), (8, 1))]));
        assert_eq!(value.cycles, vec![((0, 12), [4, 6, 1])]);
        assert_eq!(value.visible, vec![((0, 12), -1)]);
        assert_eq!(
            value.worlds,
            vec![World {
                fog: vec![1, 0],
                detect: vec![255],
                danger: vec![-7]
            }]
        );
        assert_eq!(value.goods.len(), 1);
        assert_eq!(value.goods[&5].name, "replacement");
        assert_eq!(value.goods[&5].x, 11);
        assert_eq!(value.ammo.as_ref().unwrap().len(), 1);
        assert_eq!(value.ammo.as_ref().unwrap()[&(0, 7, 2)].accuracy, 17);
        assert!(Civilian::parse(AUTHORED, &log, false).ammo.is_none());
        let bytes = encode(&value).unwrap();
        assert_eq!(decode::<Civilian>(&bytes).unwrap(), value);
        let mut reused = Civilian::default();
        for v in [&value, &Civilian::default(), &value] {
            decode_into(&encode(v).unwrap(), &mut reused).unwrap();
            assert_eq!(&reused, v);
        }
        for end in 0..bytes.len() {
            assert!(decode::<Civilian>(&bytes[..end]).is_err());
        }
        let mut trailing = bytes;
        trailing.push(0);
        assert!(decode::<Civilian>(&trailing).is_err());
        let no_fog = Log::parse(
            "BEGIN GAME\n BEGIN FRAME 3\n  BEGIN WORLD\n   seen3[scan] 4\n   danger[who][scan] 7\n",
        );
        assert!(worlds(&no_fog).is_empty());
    }

    #[test]
    fn numeric_observation_maps_reject_duplicates_lengths_and_allocation_exhaustion() {
        let mut out = Encoder(Vec::new());
        out.len(2).unwrap();
        for value in [3i64, 4] {
            1i64.put(&mut out).unwrap();
            value.put(&mut out).unwrap();
        }
        assert_eq!(
            decode::<BTreeMap<i64, i64>>(&out.0),
            Err("duplicate map key")
        );
        assert!(decode::<BTreeMap<i64, i64>>(&u32::MAX.to_le_bytes()).is_err());
        let bytes = encode(&BTreeMap::from([((1i64, 2i64), 3i64)])).unwrap();
        let mut input = Decoder {
            bytes: &bytes,
            allocation: 0,
            reuse_containers: true,
        };
        assert!(<BTreeMap<(i64, i64), i64> as Wire>::get(&mut input).is_err());
        for value in [vec![], vec![0u8, 1, 255]] {
            assert_eq!(decode::<Vec<u8>>(&encode(&value).unwrap()).unwrap(), value);
        }
    }
}
