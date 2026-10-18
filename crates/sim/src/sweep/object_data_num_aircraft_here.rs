//! `ObjectData::num_aircraft_here@00645330` under the emulator against the port (item 3120; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

fn unhex(s: &str) -> Vec<u8> {
    (0..s.len() / 2)
        .map(|i| u8::from_str_radix(&s[2 * i..2 * i + 2], 16).expect("hex"))
        .collect()
}

/// `ObjectData::num_aircraft_here@00645330`, run by `tools/emu/sweep_object_data_num_aircraft_here.py` under unicorn
/// (tools/emu): 32 random bytes of object, 1 argument(s), `p` windows in order: p0..p13 the place(s), in flag order, p14..p15 the stubs' answer window(s); 8 random bytes at 0x00c0accc (each word in 0..4); callee 0x00609dc0 (1 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0x60 (0 args) stubbed; vtable slot +0x20 (0 args) stubbed; vtable slot +0x3c (0 args) stubbed; vtable slot +0x190 (2 args) stubbed; the word at `0xc0aec0` points at 16 random bytes less `0` (placed); the word at `p0+0` points at 164 random bytes less `0` (placed); the word at `p0+4` points at 164 random bytes less `0` (placed); the word at `p0+8` points at 164 random bytes less `0` (placed); the word at `p0+12` points at 164 random bytes less `0` (placed); the word at `p1+0x18` points at 544 random bytes less `0` (placed); the word at `p2+0x18` points at 544 random bytes less `0` (placed); the word at `p3+0x18` points at 544 random bytes less `0` (placed); the word at `p4+0x18` points at 544 random bytes less `0` (placed); the word at `0xc0ab84` points at 16 random bytes less `0` (placed); the word at `p9+0` points at 164 random bytes less `0` (placed); the word at `p9+4` points at 164 random bytes less `0` (placed); the word at `p9+8` points at 164 random bytes less `0` (placed); the word at `m+0x18` points at 256 random bytes less `0` (placed); slot +0xb8 laid as 0x00653790 or its stub, a flag after the answers; stub 0 answered from [0, 1, 2] (90%); stub 1 answered from [0, 1] (90%); stub 2 answered from [0, 1] (90%); stub 3 answered from [0, 1] (90%); stub 4 answers a window of its own to each call, in call order (164 bytes, ...); stub 5 answers 2 values in call order, printed after the answers; stub 0 writes a value in [0, 1] through its argument 0, printed after the answers (`o` in a place); the stubbed vtable laid at `p13+0` as well as at `this`; the stubbed vtable laid at `p14+0` as well as at `this`; the stubbed vtable laid at `p10+0` as well as at `this`; the stubbed vtable laid at `p11+0` as well as at `this`; the stubbed vtable laid at `p12+0` as well as at `this`; the stubbed vtable laid at `p15+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1200 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_object_data_num_aircraft_here.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (lhs, rhs) = line
            .split_once(" -> ")
            .expect("a row is `<args…> | <windows> -> <eax> | <windows>`");
        let (args, before) = lhs.split_once(" | ").expect("`<args> | <before>`");
        let (eax, after) = rhs.split_once(" | ").expect("`<eax> | <after>`");
        let a: Vec<i32> = args
            .split_whitespace()
            .map(|s| s.parse().expect("an integer"))
            .collect();
        let mut wins: Vec<Vec<u8>> = before.trim().split(';').map(unhex).collect();
        let mut m = wins.remove(0);
        let mut p: Vec<Vec<u8>> = wins.split_off(1);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // `a`: 0 the stack argument, 1 `home_base`'s answer, 2 slot +0xb8's, 3 slot +0x60's, 4 slot +0x20's, 5 stub 4's window (unused: its answers are p14, p15),
                // 6 slot +0x190's single (unused), 7 what `home_base` writes through its argument -- the running count's own cell --, 8..9 slot +0x190's run, 10 the +0xb8 base flag.
                // Windows: c0 the per-player counts, p0 the units array, p1..p4 the units, p5..p8 their types, p9 the objects array.
                let rd16 = |w: &[u8], o: usize| {
                    i16::from_le_bytes(w[o..o + 2].try_into().expect("two bytes")) as i32
                };
                let rd32 = |w: &[u8], o: usize| {
                    i32::from_le_bytes(w[o..o + 4].try_into().expect("four bytes"))
                };
                let player = i32::from(m[9]);
                let idx = rd16(m, 0xa);
                let count = rd32(&c[0], 4 * player as usize);
                let mut hits = 0i32;
                for i in 0..count.max(0) as usize {
                    let unit = &p[1 + i];
                    if unit[8] & 1 == 0 || rd32(&p[5 + i], 0x218) != 2 {
                        continue;
                    }
                    // `home_base` answers a[1] and writes a[7] through the count's own cell, which `this`'s player is compared with.
                    if idx == a[1] && player == a[7] {
                        hits += 1;
                    }
                }
                let r = if a[10] != 0 { a[3] } else { a[2] };
                if r != 0 {
                    i64::from(rd16(&p[1 + idx as usize], 0xa0) + hits)
                } else if a[4] != 0 && a[0] != 0 {
                    i64::from(a[9].wrapping_sub(a[8]).wrapping_add(hits))
                } else {
                    i64::from(hits)
                }
            }
        };
        assert_eq!(got, want_eax, "num_aircraft_here — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "num_aircraft_here — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "num_aircraft_here — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[2 + i],
                "num_aircraft_here — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 1200, "only {n} rows");
    eprintln!("ObjectData::num_aircraft_here@00645330: {n} rows agree");
}
