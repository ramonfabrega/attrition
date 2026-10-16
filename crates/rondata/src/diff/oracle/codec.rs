//! Oracle schema using the shared bounded owned-record codec.
use super::Oracle;
pub(super) use crate::capture::owned_codec::LIMIT;
use crate::capture::owned_codec::*;
use crate::gamelog::*;
use std::collections::BTreeMap;
pub(super) fn encode(value: &Oracle) -> Result<Vec<u8>> {
    crate::capture::owned_codec::encode(value)
}
pub(super) fn decode(bytes: &[u8]) -> Result<Oracle> {
    crate::capture::owned_codec::decode(bytes)
}
pub(super) fn decode_into(bytes: &[u8], value: &mut Oracle) -> Result<()> {
    crate::capture::owned_codec::decode_into(bytes, value)
}
record!(Oracle { frame: Frame, groups: Vec<GroupDump>, leaders: [Option<BTreeMap<String, i64>>; 2] });
record!(Frame {
    n: i64,
    units: Vec<UnitDump>,
    builds: Vec<BuildDump>,
    leaders: Vec<LeaderDump>,
    cities: Vec<CityDump>,
    deaths: Vec<DeathDump>,
});
record!(UnitDump {
    flags: i64,
    o: i64,
    who: i64,
    pos: Pos,
    angle: Option<i64>,
    guys: Vec<Guy>,
    orders: Vec<OrderDump>,
    path: Vec<PathDump>,
    group: Option<i64>,
    form: Option<i64>,
    form_mod: Option<i64>,
    stance: Option<i64>,
    orders_x: Option<i64>,
    orders_y: Option<i64>,
    dest_angle: Option<i64>,
    tolerance: Option<i64>,
    path_recursion: Option<i64>,
    idle: Option<i64>,
    unit_masks: Option<i64>,
    unit_masks2: Option<i64>,
    attrition: Option<i64>,
    myspeed: Option<i64>,
    o_up: Option<i64>,
    o_down: Option<i64>,
    inside_up: Option<i64>,
    myhits: Option<i64>,
    damage: Option<i64>,
    damage_frac: Option<i64>,
    mylos: Option<i64>,
    infiltrated: Option<i64>,
    visible: Option<i64>,
    hold_frames: Option<i64>,
    collide: Option<i64>,
    collide_frame: Option<i64>,
    collide_o: Option<i64>,
    collide_who: Option<i64>,
    collide_guy: Option<i64>,
    recharging: Option<i64>,
    cavarch_o: Option<i64>,
    cavarch_who: Option<i64>,
    damage_frame: Option<i64>,
    damage_o: Option<i64>,
    damage_who: Option<i64>,
    safe: Option<i64>,
    start_dist: Option<i64>,
    spell_time: Option<i64>,
    uid: Option<i64>,
    down: Option<i64>,
    down_who: Option<i64>,
    up: Option<i64>,
    up_who: Option<i64>,
});
record!(BuildDump {
    flags: i64,
    o: i64,
    who: i64,
    pos: Pos,
    damage: Option<i64>,
    damage_frac: Option<i64>,
    orig_type: Option<i64>,
    max_age: Option<i64>,
    build_masks: Option<i64>,
    gather_from: Vec<(i64, i64)>,
    mining_len: Option<i64>,
    mining_size: Option<i64>,
    gather_len: Option<i64>,
    gather: Vec<(i64, i64, i64)>,
    mtn: Option<i64>,
    cliff: Option<i64>,
    gather_down: Option<i64>,
    job_counter: Option<i64>,
    constr_time: Option<i64>,
    construct_hits: Option<i64>,
    myhits: Option<i64>,
    ever_seen: Option<i64>,
    ever_seen_completed: Option<i64>,
    queued: Option<i64>,
    city: Option<i64>,
    city_down: Option<i64>,
    queue: Vec<QueueItemDump>,
});
record!(CityDump {
    x: i64,
    y: i64,
    pop: i64,
    who: i64,
    race: i64,
    city_flags: i64,
    city: i64,
    attack_stamp: i64,
    raid_stamp: i64,
    reduce_stamp: i64,
    capture_stamp: i64,
    assimilation_timer: i64,
    capture_strength: i64,
    vans: Vec<(i64, i64)>,
    o: i64,
    reg: i64,
    scouted: i64,
    in_port: i64,
    peasant_dist: i64,
    trade_val: i64,
    free: i64,
    busy: i64,
    gatherers: i64,
    ocean: i64,
    land: i64,
    filled: i64,
    bordering: i64,
    ocean_filled: i64,
    dock_tile: i64,
    was_capital_flags: i64,
    space: [i64; 3],
    ter: [i64; 6],
});
record!(LeaderDump {
    who: i64,
    tribe: i64,
    defeated_by: i64,
    gov: i64,
    score: i64,
    leader_flags: i64,
    leader_flags2: i64,
    diplos: Vec<i64>,
});
record!(DeathDump {
    valid: Option<i64>,
    first_frame: Option<i64>,
    cur_anim: Option<i64>,
    who: Option<i64>,
    o: Option<i64>,
    gpiece: Option<i64>,
});
record!(OrderDump {
    index: i64,
    metric: i64,
    kind: String,
    flags: i64,
    ox: Option<i64>,
    whom: Option<i64>,
    uid: Option<i64>,
    tx: Option<i64>,
    ty: Option<i64>,
    build_type: Option<i64>,
    been_there: Option<i64>,
    goto_build: Option<i64>,
    non_flat_gather: Option<i64>,
    dist_mod: Option<i64>,
    wait: Option<i64>,
    x: Option<i64>,
    y: Option<i64>,
    dest: Option<i64>,
    angle: Option<i64>,
    tolerance: Option<i64>,
    pause: Option<i64>,
    retry: Option<i64>,
    attempts: Option<i64>,
    timer: Option<i64>,
    facing: Option<i64>,
    dest_x: Option<i64>,
    dest_y: Option<i64>,
    last_x: Option<i64>,
    last_y: Option<i64>,
    coll_x: Option<i64>,
    coll_y: Option<i64>,
    orig_x: Option<i64>,
    orig_y: Option<i64>,
    off_x: Option<i64>,
    off_y: Option<i64>,
    oxx: Option<i64>,
    whose: Option<i64>,
    group_angle: Option<i64>,
    group_id: Option<i64>,
    form_id: Option<i64>,
    in_group: Option<i64>,
    air_oxx: Option<i64>,
    air_whose: Option<i64>,
    cruising_alt: Option<i64>,
    sharp_turn: Option<i64>,
    air_old: Option<i64>,
    returning: Option<i64>,
    strafe_xx: Option<i64>,
    strafe_yy: Option<i64>,
    mandatory: Option<i64>,
    defensive: Option<i64>,
    in_range: Option<i64>,
    ever_in_range: Option<i64>,
    new_ord: Option<i64>,
    def_x: Option<i64>,
    def_y: Option<i64>,
    guard_dx: Option<i64>,
    guard_dy: Option<i64>,
    guard_x: Option<i64>,
    guard_y: Option<i64>,
    guard_idle: Option<i64>,
    guard_retry: Option<i64>,
    garrison_search: Option<i64>,
    cast_spell: Option<i64>,
    cast_paid: Option<i64>,
    ag_att_x: Option<i64>,
    ag_att_y: Option<i64>,
    ag_accuracy: Option<i64>,
    ag_attack_unit: Option<i64>,
    patrol_x: Vec<i64>,
    patrol_y: Vec<i64>,
    waypoint: Option<i64>,
});
record!(PathDump {
    to: (i64, i64),
    tolerance: i64,
    flags: i64,
});
record!(QueueItemDump {
    ty: i64,
    job_counter: i64,
    cost: [i64; 3],
    good: [i64; 3],
});
record!(GroupDump {
    id: i64,
    who: i64,
    num: i64,
    army: i64,
    ox: i64,
    oy: i64,
    o_dist: i64,
    o_angle: i64,
    buildings: i64,
    disband: i64,
    order_num: i64,
    priority: i64,
    stamp: i64,
    role: i64,
    form: i64,
    think_frame: i64,
    facing: i64,
    new_speed: i64,
    speed: i64,
    form_num: i64,
    members: Vec<GroupMemberDump>,
});
record!(GroupMemberDump {
    o: i64,
    off_x: i64,
    off_y: i64,
    curr_x: i64,
    curr_y: i64,
    angle: i64,
});

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn whole_record_roundtrip_and_invalid_inputs() {
        let value = Oracle {
            frame: Frame {
                n: i64::MIN,
                units: vec![UnitDump {
                    who: i64::MAX,
                    guys: vec![Guy {
                        pos: Some(Pos {
                            x: -1,
                            y: i64::MIN,
                            z: i64::MAX,
                        }),
                        ..Guy::default()
                    }],
                    orders: vec![OrderDump::default()],
                    ..UnitDump::default()
                }],
                ..Frame::default()
            },
            groups: vec![GroupDump {
                members: vec![GroupMemberDump::default()],
                ..GroupDump::default()
            }],
            leaders: [
                Some(BTreeMap::from([("Unicode: λ".into(), i64::MAX)])),
                None,
            ],
        };
        let bytes = encode(&value).unwrap();
        assert_eq!(decode(&bytes).unwrap(), value);
        let mut reused = decode(&bytes).unwrap();
        let empty = Oracle::default();
        decode_into(&encode(&empty).unwrap(), &mut reused).unwrap();
        assert_eq!(reused, empty);
        decode_into(&bytes, &mut reused).unwrap();
        assert_eq!(reused, value);
        for end in 0..bytes.len() {
            assert!(decode(&bytes[..end]).is_err(), "truncation {end}");
        }
        let mut extra = bytes.clone();
        extra.push(0);
        assert!(decode(&extra).is_err());
        let mut input = Decoder {
            reuse_containers: true,
            bytes: &[2],
            allocation: LIMIT,
        };
        assert!(Option::<i64>::get(&mut input).is_err());
        let mut input = Decoder {
            reuse_containers: true,
            bytes: &[255; 4],
            allocation: LIMIT,
        };
        assert!(Vec::<Frame>::get(&mut input).is_err());
        let mut out = Encoder(Vec::new());
        out.len(2).unwrap();
        for _ in 0..2 {
            "duplicate".to_owned().put(&mut out).unwrap();
            1i64.put(&mut out).unwrap();
        }
        let mut input = Decoder {
            reuse_containers: true,
            bytes: &out.0,
            allocation: LIMIT,
        };
        assert!(<BTreeMap<String, i64> as Wire>::get(&mut input).is_err());
        let mut input = Decoder {
            reuse_containers: true,
            bytes: &bytes,
            allocation: 0,
        };
        assert!(Oracle::get(&mut input).is_err());
    }
}
