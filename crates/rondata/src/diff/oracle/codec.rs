//! Bounded, exhaustive codec for owned oracle records; no simulation values.
use super::Oracle;
use crate::gamelog::*;
use std::collections::BTreeMap;

pub(super) const LIMIT: usize = 32 * 1024 * 1024;
type Result<T> = std::result::Result<T, &'static str>;
struct Encoder(Vec<u8>);
impl Encoder {
    fn put(&mut self, bytes: &[u8]) -> Result<()> {
        if bytes.len() > LIMIT.saturating_sub(self.0.len()) {
            return Err("encoded oracle too large");
        }
        self.0.extend_from_slice(bytes);
        Ok(())
    }
    fn len(&mut self, n: usize) -> Result<()> {
        self.put(
            &u32::try_from(n)
                .map_err(|_| "length overflow")?
                .to_le_bytes(),
        )
    }
}
struct Decoder<'a> {
    bytes: &'a [u8],
    allocation: usize,
}
impl<'a> Decoder<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8]> {
        if n > self.bytes.len() {
            return Err("truncated oracle");
        }
        let (head, tail) = self.bytes.split_at(n);
        self.bytes = tail;
        Ok(head)
    }
    fn len(&mut self) -> Result<usize> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()) as usize)
    }
    fn reserve(&mut self, bytes: usize) -> Result<()> {
        self.allocation = self
            .allocation
            .checked_sub(bytes)
            .ok_or("allocation limit")?;
        Ok(())
    }
}
trait Wire: Sized + Default {
    fn heap_bytes(&self) -> usize {
        0
    }
    const MIN: usize;
    fn put(&self, out: &mut Encoder) -> Result<()>;
    fn get(input: &mut Decoder<'_>) -> Result<Self>;
    fn read_into(&mut self, input: &mut Decoder<'_>) -> Result<()> {
        *self = Self::get(input)?;
        Ok(())
    }
}
impl Wire for i64 {
    const MIN: usize = 8;
    fn put(&self, out: &mut Encoder) -> Result<()> {
        out.put(&self.to_le_bytes())
    }
    fn get(input: &mut Decoder<'_>) -> Result<Self> {
        Ok(Self::from_le_bytes(input.take(8)?.try_into().unwrap()))
    }
}
impl<T: Wire> Wire for Option<T> {
    fn heap_bytes(&self) -> usize {
        self.as_ref().map_or(0, Wire::heap_bytes)
    }
    const MIN: usize = 1;
    fn put(&self, out: &mut Encoder) -> Result<()> {
        out.put(&[u8::from(self.is_some())])?;
        if let Some(value) = self {
            value.put(out)?;
        }
        Ok(())
    }
    fn get(input: &mut Decoder<'_>) -> Result<Self> {
        match input.take(1)?[0] {
            0 => Ok(None),
            1 => Ok(Some(T::get(input)?)),
            _ => Err("invalid option tag"),
        }
    }
}
impl<T: Wire> Wire for Vec<T> {
    fn heap_bytes(&self) -> usize {
        self.capacity() * std::mem::size_of::<T>()
            + self.iter().map(Wire::heap_bytes).sum::<usize>()
    }
    const MIN: usize = 4;
    fn put(&self, out: &mut Encoder) -> Result<()> {
        out.len(self.len())?;
        for value in self {
            value.put(out)?;
        }
        Ok(())
    }
    fn get(input: &mut Decoder<'_>) -> Result<Self> {
        let n = input.len()?;
        if n > input.bytes.len() / T::MIN.max(1) {
            return Err("invalid vector length");
        }
        input.reserve(
            n.checked_mul(std::mem::size_of::<T>())
                .ok_or("allocation overflow")?,
        )?;
        let mut values = Vec::with_capacity(n);
        for _ in 0..n {
            values.push(T::get(input)?);
        }
        Ok(values)
    }
    fn read_into(&mut self, input: &mut Decoder<'_>) -> Result<()> {
        let n = input.len()?;
        if n > input.bytes.len() / T::MIN.max(1) {
            return Err("invalid vector length");
        }
        input.reserve(
            n.checked_mul(std::mem::size_of::<T>())
                .ok_or("allocation overflow")?,
        )?;
        self.truncate(n);
        if n > self.capacity() {
            self.reserve_exact(n - self.len());
        }
        self.resize_with(n, T::default);
        for value in self {
            value.read_into(input)?;
        }
        Ok(())
    }
}
impl Wire for String {
    fn heap_bytes(&self) -> usize {
        self.capacity()
    }
    const MIN: usize = 4;
    fn put(&self, out: &mut Encoder) -> Result<()> {
        out.len(self.len())?;
        out.put(self.as_bytes())
    }
    fn get(input: &mut Decoder<'_>) -> Result<Self> {
        let n = input.len()?;
        input.reserve(n)?;
        Ok(std::str::from_utf8(input.take(n)?)
            .map_err(|_| "invalid UTF-8")?
            .to_owned())
    }
    fn read_into(&mut self, input: &mut Decoder<'_>) -> Result<()> {
        let n = input.len()?;
        input.reserve(n)?;
        let text = std::str::from_utf8(input.take(n)?).map_err(|_| "invalid UTF-8")?;
        self.clear();
        self.push_str(text);
        Ok(())
    }
}
impl<T: Wire, const N: usize> Wire for [T; N]
where
    [T; N]: Default,
{
    fn heap_bytes(&self) -> usize {
        self.iter().map(Wire::heap_bytes).sum()
    }
    const MIN: usize = T::MIN * N;
    fn put(&self, out: &mut Encoder) -> Result<()> {
        for value in self {
            value.put(out)?;
        }
        Ok(())
    }
    fn get(input: &mut Decoder<'_>) -> Result<Self> {
        let v: Result<Vec<_>> = (0..N).map(|_| T::get(input)).collect();
        v?.try_into().map_err(|_| "array length")
    }
}
impl Wire for (i64, i64) {
    const MIN: usize = 16;
    fn put(&self, out: &mut Encoder) -> Result<()> {
        self.0.put(out)?;
        self.1.put(out)
    }
    fn get(input: &mut Decoder<'_>) -> Result<Self> {
        Ok((i64::get(input)?, i64::get(input)?))
    }
}
impl Wire for (i64, i64, i64) {
    const MIN: usize = 24;
    fn put(&self, out: &mut Encoder) -> Result<()> {
        self.0.put(out)?;
        self.1.put(out)?;
        self.2.put(out)
    }
    fn get(input: &mut Decoder<'_>) -> Result<Self> {
        Ok((i64::get(input)?, i64::get(input)?, i64::get(input)?))
    }
}
impl Wire for BTreeMap<String, i64> {
    fn heap_bytes(&self) -> usize {
        self.len() * 128 + self.keys().map(String::capacity).sum::<usize>()
    }
    const MIN: usize = 4;
    fn put(&self, out: &mut Encoder) -> Result<()> {
        out.len(self.len())?;
        for (k, v) in self {
            k.put(out)?;
            v.put(out)?;
        }
        Ok(())
    }
    fn get(input: &mut Decoder<'_>) -> Result<Self> {
        let n = input.len()?;
        if n > input.bytes.len() / 12 {
            return Err("invalid map length");
        }
        input.reserve(n.checked_mul(128).ok_or("map allocation overflow")?)?;
        let mut result = BTreeMap::new();
        for _ in 0..n {
            if result
                .insert(String::get(input)?, i64::get(input)?)
                .is_some()
            {
                return Err("duplicate map key");
            }
        }
        Ok(result)
    }
}
// Both patterns are exhaustive: a new owned field cannot silently disappear.
macro_rules! record {
    ($ty:ident { $($field:ident: $kind:ty),* $(,)? }) => {
        impl Wire for $ty {
            fn heap_bytes(&self) -> usize {
                let Self { $($field),* } = self;
                0 $(+ $field.heap_bytes())*
            }
            const MIN: usize = 0 $(+ <$kind as Wire>::MIN)*;
            fn put(&self, out: &mut Encoder) -> Result<()> {
                let Self { $($field),* } = self;
                $($field.put(out)?;)*
                Ok(())
            }
            fn get(input: &mut Decoder<'_>) -> Result<Self> {
                Ok(Self { $($field: <$kind as Wire>::get(input)?),* })
            }
            fn read_into(&mut self, input: &mut Decoder<'_>) -> Result<()> {
                let Self { $($field),* } = self;
                $($field.read_into(input)?;)*
                Ok(())
            }
        }
    };
}

pub(super) fn encode(value: &Oracle) -> Result<Vec<u8>> {
    let mut out = Encoder(Vec::new());
    value.put(&mut out)?;
    Ok(out.0)
}
pub(super) fn decode(bytes: &[u8]) -> Result<Oracle> {
    let mut value = Oracle::default();
    decode_into(bytes, &mut value)?;
    Ok(value)
}
pub(super) fn decode_into(bytes: &[u8], value: &mut Oracle) -> Result<()> {
    if bytes.len() > LIMIT {
        return Err("oracle too large");
    }
    // Capacity retained at different nested vector slots must not ratchet
    // across frames. At most 16 MiB survives into the next bounded decode.
    if value.heap_bytes() > 16 * 1024 * 1024 {
        *value = Oracle::default();
    }
    let mut input = Decoder {
        bytes,
        allocation: 64 * 1024 * 1024,
    };
    value.read_into(&mut input)?;
    if !input.bytes.is_empty() {
        return Err("trailing oracle bytes");
    }
    Ok(())
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
record!(Guy {
    kind: Option<i64>,
    pos: Option<Pos>,
    angle: Option<i64>,
    cur_time: Option<i64>,
    end_time: Option<i64>,
    last_time: Option<i64>,
    cur_anim: Option<i64>,
    gpiece: Option<i64>,
    guy_flags: Option<i64>,
    stopped: Option<i64>,
    guy_num: Option<i64>,
    hold_attack: Option<i64>,
    queued_attack: Option<i64>,
    ox: Option<i64>,
    whom: Option<i64>,
    des: Option<Pos>,
    des_angle: Option<i64>,
    track: Option<(i64, i64)>,
    last_pos: Option<Pos>,
    last_speed: Option<i64>,
    avg_speed: Option<i64>,
    bank: Option<i64>,
    last_bank: Option<i64>,
    pitch: Option<i64>,
    last_pitch: Option<i64>,
    turret_angles: Option<[i64; 4]>,
    des_turret_angles: Option<[i64; 4]>,
    node_flags: Option<i64>,
    des_node_flags: Option<i64>,
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
record!(Pos {
    x: i64,
    y: i64,
    z: i64,
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
            bytes: &[2],
            allocation: LIMIT,
        };
        assert!(Option::<i64>::get(&mut input).is_err());
        let mut input = Decoder {
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
            bytes: &out.0,
            allocation: LIMIT,
        };
        assert!(<BTreeMap<String, i64> as Wire>::get(&mut input).is_err());
        let mut input = Decoder {
            bytes: &bytes,
            allocation: 0,
        };
        assert!(Oracle::get(&mut input).is_err());
    }
}
