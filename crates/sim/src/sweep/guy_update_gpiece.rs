//! `Guy::update_gpiece@005d8530` under the emulator against the port (item 3229; tools/sweep_gen.py, obj with windows).

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

/// `Guy::update_gpiece@005d8530`, run by `tools/emu/sweep_guy_update_gpiece.py` under unicorn
/// (tools/emu): 256 random bytes of object, 0 argument(s), `p` windows in order: p0..p22 the place(s), in flag order; the global cell 0x00c06214 points at 2048 random bytes; 4 random bytes at 0x00c06244 (each word in 1056964608..1082130432); callee 0x0060aa60 (0 args) stubbed; callee 0x0090c030 (7 args) stubbed; callee 0x008e4780 (1 args) stubbed; the word at `0xc0aec0` points at 16 random bytes less `-4` (placed); the word at `0xc0aedc` points at 16 random bytes less `-4` (placed); the word at `p0+0` points at 128 random bytes less `0` (placed); the word at `p0+4` points at 128 random bytes less `0` (placed); the word at `p0+8` points at 128 random bytes less `0` (placed); the word at `p1+0` points at 128 random bytes less `0` (placed); the word at `p1+4` points at 128 random bytes less `0` (placed); the word at `p1+8` points at 128 random bytes less `0` (placed); the word at `p2+0x18` points at 8 random bytes less `0` (placed); the word at `p3+0x18` points at 8 random bytes less `0` (placed); the word at `p4+0x18` points at 8 random bytes less `0` (placed); the word at `p5+0x18` points at 8 random bytes less `0` (placed); the word at `p6+0x18` points at 8 random bytes less `0` (placed); the word at `p7+0x18` points at 8 random bytes less `0` (placed); the word at `c0+0x728` points at 16 random bytes less `0` (placed); the word at `p14+0` points at 160 random bytes less `0` (placed); the word at `p14+4` points at 160 random bytes less `0` (placed); the word at `p14+8` points at 160 random bytes less `0` (placed); the word at `p14+12` points at 160 random bytes less `0` (placed); the word at `p15+0x94` points at 20 random bytes less `0` (placed, null a third of the rows); the word at `p16+0x94` points at 20 random bytes less `0` (placed, null a third of the rows); the word at `p17+0x94` points at 20 random bytes less `0` (placed, null a third of the rows); the word at `p18+0x94` points at 20 random bytes less `0` (placed, null a third of the rows); stub 0 answered from [0, 1] (90%); stub 1 answers 0..3 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_guy_update_gpiece.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (line, calls) = line.split_once(" # ").expect("`<row> # <calls>`");
        let want_log: Vec<&str> = calls.split_whitespace().collect();
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
        let mut p: Vec<Vec<u8>> = wins.split_off(2);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // Windows: m `this` (the guy); c1 `guy_scale`; p0/p1 the `units` rows 0 and 1 (the
            // pointer is the buffer + 4, so slot -1/0/1 are words 0/1/2), p2..p7 their unit data
            // (`+0x68` masks), p8..p13 their types (`+4`), p14 the pieces' array at `+0x728`,
            // p15..p18 its records (`+0x94` the art, `+0x88` a float), p19..p22 the art (+0xc, +0x10).
            // a[0] is `is_packing`'s answer, a[1] `get_unit_gpiece`'s (0..3).
            let idx = m[0xa1] as i8 as i32;
            let slot = i16::from_le_bytes([m[0x8c], m[0x8d]]) as i32;
            assert!(
                (0..=1).contains(&idx) && (-1..=1).contains(&slot),
                "drawn in range"
            );
            let k = (3 * idx + slot + 1) as usize;
            let (ud, ty) = (2 + k, 8 + k);
            let ty_id = rd(&p[ty], 4);
            let masks = rd(&p[ud], 0x68);
            let c2 = m[0xa2] as i8 as i32;
            let old = rd(m, 0x88);
            let mut s0 = "-".to_string();
            let flag = if ty_id != 0x3d && ty_id != 0x3e && ty_id != 400 {
                masks & 0x80000
            } else if masks & 0x80000 != 0 {
                1
            } else {
                s0 = format!("p{ud}+0x0");
                (a[0] != 0) as i32
            };
            let new = a[1];
            log.push(s0);
            log.push(format!("c0+0x0,{ty_id},{idx},{slot},{c2},{flag},0,0"));
            log.push(if old != new {
                format!("c0+0x0,{new}")
            } else {
                "-".into()
            });
            wr(m, 0x88, new);
            let rec = 15 + new as usize;
            let art = rd(&p[rec], 0x94) != 0;
            // cvttss2si: the indefinite integer for NaN and out of range
            let cvt = |x: f32| -> i32 {
                if x.is_nan() || x >= 2147483648.0 || x < -2147483648.0 {
                    i32::MIN
                } else {
                    x as i32
                }
            };
            if c2 != 0 && art {
                let fl = |b: &[u8], o: usize| f32::from_bits(rd(b, o) as u32);
                let (s, f1) = (fl(&c[1], 0), fl(&p[rec], 0x88));
                let (fa, fb) = (
                    fl(&p[19 + new as usize], 0xc),
                    fl(&p[19 + new as usize], 0x10),
                );
                wr(m, 0xd0, 0);
                wr(m, 0x54, cvt(fa * s * f1));
                wr(m, 0x58, cvt(s * fb * f1));
            } else {
                wr(m, 0x54, 0);
                wr(m, 0x58, 0);
                wr(m, 0xd0, 0);
            }
            0
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "update_gpiece — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "update_gpiece — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "update_gpiece — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[3 + i],
                "update_gpiece — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 1000, "only {n} rows");
    eprintln!("Guy::update_gpiece@005d8530: {n} rows agree");
}
