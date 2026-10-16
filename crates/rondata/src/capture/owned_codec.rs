//! Bounded, exhaustive codec for owned oracle records; no simulation values.
use crate::gamelog::{Guy, Pos};
use std::collections::BTreeMap;

pub(crate) const LIMIT: usize = 32 * 1024 * 1024;
pub(crate) type Result<T> = std::result::Result<T, &'static str>;
pub(crate) struct Encoder(pub(crate) Vec<u8>);
impl Encoder {
    fn put(&mut self, bytes: &[u8]) -> Result<()> {
        if bytes.len() > LIMIT.saturating_sub(self.0.len()) {
            return Err("encoded oracle too large");
        }
        self.0.extend_from_slice(bytes);
        Ok(())
    }
    pub(crate) fn len(&mut self, n: usize) -> Result<()> {
        self.put(
            &u32::try_from(n)
                .map_err(|_| "length overflow")?
                .to_le_bytes(),
        )
    }
}
#[derive(Clone, Copy)]
pub(crate) struct Decoder<'a> {
    pub(crate) bytes: &'a [u8],
    pub(crate) allocation: usize,
    pub(crate) reuse_containers: bool,
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
pub(crate) trait Wire: Sized + Default {
    fn heap_bytes(&self) -> usize {
        0
    }
    const MIN: usize;
    fn put(&self, out: &mut Encoder) -> Result<()>;
    fn get(input: &mut Decoder<'_>) -> Result<Self>;
    fn read_into(&mut self, input: &mut Decoder<'_>) -> Result<()> {
        *self = <Self as Wire>::get(input)?;
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
    fn read_into(&mut self, input: &mut Decoder<'_>) -> Result<()> {
        if !input.reuse_containers || !std::mem::needs_drop::<T>() {
            *self = <Self as Wire>::get(input)?;
            return Ok(());
        }
        match input.take(1)?[0] {
            0 => *self = None,
            1 => self.get_or_insert_with(T::default).read_into(input)?,
            _ => return Err("invalid option tag"),
        }
        Ok(())
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
    fn read_into(&mut self, input: &mut Decoder<'_>) -> Result<()> {
        if !input.reuse_containers {
            *self = <Self as Wire>::get(input)?;
        } else {
            for value in self {
                value.read_into(input)?;
            }
        }
        Ok(())
    }
}
impl<A: Wire, B: Wire> Wire for (A, B) {
    fn heap_bytes(&self) -> usize {
        self.0.heap_bytes() + self.1.heap_bytes()
    }
    const MIN: usize = A::MIN + B::MIN;
    fn put(&self, out: &mut Encoder) -> Result<()> {
        self.0.put(out)?;
        self.1.put(out)
    }
    fn get(input: &mut Decoder<'_>) -> Result<Self> {
        Ok((A::get(input)?, B::get(input)?))
    }
    fn read_into(&mut self, input: &mut Decoder<'_>) -> Result<()> {
        if !input.reuse_containers {
            *self = <Self as Wire>::get(input)?;
        } else {
            self.0.read_into(input)?;
            self.1.read_into(input)?;
        }
        Ok(())
    }
}
impl Wire for u32 {
    const MIN: usize = 4;
    fn put(&self, out: &mut Encoder) -> Result<()> {
        out.put(&self.to_le_bytes())
    }
    fn get(input: &mut Decoder<'_>) -> Result<Self> {
        Ok(Self::from_le_bytes(input.take(4)?.try_into().unwrap()))
    }
}
impl Wire for bool {
    const MIN: usize = 1;
    fn put(&self, out: &mut Encoder) -> Result<()> {
        out.put(&[u8::from(*self)])
    }
    fn get(input: &mut Decoder<'_>) -> Result<Self> {
        match input.take(1)?[0] {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err("invalid bool"),
        }
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
    fn read_into(&mut self, input: &mut Decoder<'_>) -> Result<()> {
        if input.reuse_containers {
            // A stable schema is common across adjacent frames. Verify every
            // key before reusing its String and tree node; no value is cached.
            // Trial consumption is committed only on a complete key match.
            let mut trial = *input;
            let n = trial.len()?;
            if n > trial.bytes.len() / 12 {
                return Err("invalid map length");
            }
            trial.reserve(n.checked_mul(128).ok_or("map allocation overflow")?)?;
            if n == self.len() {
                let mut matched = true;
                for (key, value) in self.iter_mut() {
                    let len = trial.len()?;
                    trial.reserve(len)?;
                    let text =
                        std::str::from_utf8(trial.take(len)?).map_err(|_| "invalid UTF-8")?;
                    if text != key {
                        matched = false;
                        break;
                    }
                    *value = i64::get(&mut trial)?;
                }
                if matched {
                    *input = trial;
                    return Ok(());
                }
            }
        }
        // Changed, reordered or duplicate keys use the original decoder, with
        // its original allocation budget and duplicate-key rejection.
        *self = <Self as Wire>::get(input)?;
        Ok(())
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

pub(crate) use record;
pub(crate) fn encode<T: Wire>(value: &T) -> Result<Vec<u8>> {
    let mut out = Encoder(Vec::new());
    value.put(&mut out)?;
    Ok(out.0)
}
pub(crate) fn decode<T: Wire>(bytes: &[u8]) -> Result<T> {
    let mut value = T::default();
    decode_into(bytes, &mut value)?;
    Ok(value)
}
pub(crate) fn decode_into<T: Wire>(bytes: &[u8], value: &mut T) -> Result<()> {
    if bytes.len() > LIMIT {
        return Err("oracle too large");
    }
    // Capacity retained at different nested vector slots must not ratchet
    // across frames. At most 16 MiB survives into the next bounded decode.
    if value.heap_bytes() > 16 * 1024 * 1024 {
        *value = T::default();
    }
    let mut input = Decoder {
        bytes,
        allocation: 64 * 1024 * 1024,
        reuse_containers: std::env::var_os("RON_DISABLE_DECODE_REUSE").is_none(),
    };
    value.read_into(&mut input)?;
    if !input.bytes.is_empty() {
        return Err("trailing oracle bytes");
    }
    Ok(())
}
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
record!(Pos {
    x: i64,
    y: i64,
    z: i64,
});

#[cfg(test)]
mod reuse_tests {
    use super::*;
    fn read<T: Wire>(bytes: &[u8], value: &mut T, reuse: bool, budget: usize) -> Result<usize> {
        let mut input = Decoder {
            bytes,
            allocation: budget,
            reuse_containers: reuse,
        };
        value.read_into(&mut input)?;
        if !input.bytes.is_empty() {
            return Err("trailing oracle bytes");
        }
        Ok(input.allocation)
    }
    #[test]
    fn stable_nested_maps_reuse_keys_and_match_fresh_decodes() {
        type Maps = Option<[BTreeMap<String, i64>; 2]>;
        let map = |v: i64| BTreeMap::from([("first".into(), v), ("second".into(), -v)]);
        let mut reused: Maps = Some([map(1), map(2)]);
        let ptr = reused.as_ref().unwrap()[0]
            .first_key_value()
            .unwrap()
            .0
            .as_ptr();
        let next = Some([map(3), map(4)]);
        let bytes = encode(&next).unwrap();
        let budget = read(&bytes, &mut reused, true, 4096).unwrap();
        assert_eq!(reused, next);
        assert_eq!(
            ptr,
            reused.as_ref().unwrap()[0]
                .first_key_value()
                .unwrap()
                .0
                .as_ptr()
        );
        let mut legacy = None;
        assert_eq!(
            read::<Maps>(&bytes, &mut legacy, false, 4096).unwrap(),
            budget
        );
        for next in [
            None,
            Some([BTreeMap::new(), map(8)]),
            Some([map(9), map(10)]),
        ] {
            let bytes = encode(&next).unwrap();
            read(&bytes, &mut reused, true, 4096).unwrap();
            read(&bytes, &mut legacy, false, 4096).unwrap();
            assert_eq!(reused, legacy);
            assert_eq!(reused, next);
        }
    }
    #[test]
    fn changed_reordered_invalid_and_bounded_maps_match_original_decoder() {
        let original = BTreeMap::from([("a".to_string(), 1), ("b".to_string(), 2)]);
        let cases = [
            vec![("a", 3), ("b", 4)],
            vec![("a", 3), ("c", 4)],
            vec![("b", 4), ("a", 3)],
            vec![("a", 3), ("a", 4)],
            vec![],
            vec![("z", 9)],
        ];
        for pairs in cases {
            let mut out = Encoder(Vec::new());
            out.len(pairs.len()).unwrap();
            for (key, value) in pairs {
                key.to_string().put(&mut out).unwrap();
                (value as i64).put(&mut out).unwrap();
            }
            let bytes = out.0;
            for end in 0..=bytes.len() {
                for budget in [0, 128, 257, 4096] {
                    let mut fast = original.clone();
                    let mut legacy = original.clone();
                    let a = read(&bytes[..end], &mut fast, true, budget);
                    let b = read(&bytes[..end], &mut legacy, false, budget);
                    assert_eq!(a, b, "acceptance/budget at {end}/{budget}");
                    if a.is_ok() {
                        assert_eq!(fast, legacy);
                    }
                }
            }
        }
        let bytes = [
            1u32.to_le_bytes().as_slice(),
            1u32.to_le_bytes().as_slice(),
            &[255],
            &0i64.to_le_bytes(),
        ]
        .concat();
        for reuse in [false, true] {
            assert_eq!(
                read(&bytes, &mut BTreeMap::from([("a".into(), 0)]), reuse, 4096),
                Err("invalid UTF-8")
            );
        }
    }
    #[test]
    fn nested_tuple_buffers_preserve_values_across_growth_and_shrink() {
        let mut fast: (String, Vec<(String, Vec<i64>)>) = Default::default();
        let mut legacy = fast.clone();
        for n in [10, 2, 0, 20, 1] {
            let value = (
                format!("n={n}"),
                (0..n)
                    .map(|i| (format!("key{i}"), vec![i as i64; i]))
                    .collect(),
            );
            let bytes = encode(&value).unwrap();
            read(&bytes, &mut fast, true, 65536).unwrap();
            read(&bytes, &mut legacy, false, 65536).unwrap();
            assert_eq!(fast, legacy);
            assert_eq!(fast, value);
        }
    }
}
