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
pub(crate) struct Decoder<'a> {
    pub(crate) bytes: &'a [u8],
    pub(crate) allocation: usize,
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
