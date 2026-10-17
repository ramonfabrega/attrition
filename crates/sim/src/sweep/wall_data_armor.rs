//! `WallData::armor@0063fa60` under the emulator against the port (item 3024; tools/sweep_gen.py, obj with windows).

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

/// `WallData::armor@0063fa60`, run by `tools/emu/sweep_wall_data_armor.py` under unicorn
/// (tools/emu): 32 random bytes of object, 0 argument(s), `p` windows in order: p0..p3 sparse table 0 (2 rows × 2 slice(s), row-major), p4 the place; the global cell 0x00c061e4 points at 3200 random bytes; callee 0x00647db0 (0 args) stubbed; vtable slot +0x18 (0 args) stubbed; vtable slot +0xc4 (0 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0x20 (0 args) stubbed; vtable slot +0x60 (2 args) stubbed; vtable slot +0x4c (0 args) stubbed; callee 0x0073a1b0 (6 args) stubbed; the word at `m+0x18` points at 576 random bytes less `0` (placed); slot +0xb8 laid as 0x00653790 or its stub, a flag after the answers; slot +0x4c laid as 0x00472350 or its stub, a flag after the answers; stub 1 answered from [0, 1, 1] (90%); stub 2 answered from [0, 1, 1] (90%); stub 3 answered from [0, 1] (90%); stub 4 answered from [0, 1] (90%); stub 5 answered from [0, 1] (90%); stub 6 answered from [0, 1] (90%); stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; 2 rows 0x6eec apart at `0xc061e0`, slices +0x8:0x4, +0x59c0:0x2 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p4+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 500 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_wall_data_armor.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(1);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // `a[0]` `ObjectData::armor`'s answer, `a[1..8]` the answers of slots 0x18, 0xc4,
            // 0xb8 (flag `a[8]`: inline, the type record's slot 0x60 `a[5]`), 0x20, 0x60, 0x4c
            // (flag `a[9]`: inline, bit 2 of `this+8`) and the direct callee `find_hero`, which
            // a general fails to be: `this+0xa` the unit index, else a hero search.
            let owner = m[9] as usize;
            let mut armor = a[0];
            log.push("-".into());
            let eax: i32 =
                if a[1] != 0 && a[2] != 0 && (if a[8] != 0 { a[5] != 0 } else { a[3] != 0 }) {
                    i32::from(i16::from_le_bytes([m[0xa], m[0xb]]))
                } else {
                    let rec = &p[4];
                    let extra = if a[4] != 0 {
                        rd(rec, 0x238)
                            .wrapping_add(rd(rec, 0x234))
                            .wrapping_mul(0x60)
                    } else {
                        0
                    };
                    let (x10, x14) = (rd(m, 0x10) ^ 0x63637, rd(m, 0x14) ^ 0x63637);
                    if p[2 * owner + 1][0] | p[2 * owner + 1][1] == 0 {
                        -1
                    } else {
                        log[0] =
                            format!("{x10},{x10},{x14},{},0,353,{extra}", rd(&p[2 * owner], 0));
                        a[7]
                    }
                };
            if eax >= 0 {
                armor = armor.wrapping_add(rd(&c[0], 0xc78));
            }
            let active = if a[9] != 0 { m[8] & 4 != 0 } else { a[6] != 0 };
            if !active {
                armor /= 2;
            }
            i64::from(armor)
        };
        assert_eq!(got, want_eax, "armor — eax, row `{line}`");
        assert_eq!(log, want_log, "armor — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "armor — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "armor — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[2 + i], "armor — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 500, "only {n} rows");
    eprintln!("WallData::armor@0063fa60: {n} rows agree");
}
