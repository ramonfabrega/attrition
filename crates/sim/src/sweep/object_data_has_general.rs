//! `ObjectData::has_general@00646b00` under the emulator against the port (item 3024; tools/sweep_gen.py, obj with windows).

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

/// `ObjectData::has_general@00646b00`, run by `tools/emu/sweep_object_data_has_general.py` under unicorn
/// (tools/emu): 32 random bytes of object, 2 argument(s), `p` windows in order: p0..p3 sparse table 0 (2 rows × 2 slice(s), row-major), p4..p10 the place(s), in flag order; vtable slot +0x18 (0 args) stubbed; vtable slot +0xc4 (0 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0x20 (0 args) stubbed; vtable slot +0x60 (2 args) stubbed; callee 0x0073a1b0 (6 args) stubbed; the word at `0xc0aec0` points at 8 random bytes less `0` (placed); the word at `0xc0aec0+0x1c` points at 8 random bytes less `0` (placed); the word at `p4+0` points at 112 random bytes less `0` (placed); the word at `p4+4` points at 112 random bytes less `0` (placed); the word at `p5+0` points at 112 random bytes less `0` (placed); the word at `p5+4` points at 112 random bytes less `0` (placed); the word at `m+0x18` points at 576 random bytes less `0` (placed); slot +0xb8 laid as 0x00653790 or its stub, a flag after the answers; stub 0 answered from [0, 1, 1] (90%); stub 1 answered from [0, 1, 1] (90%); stub 2 answered from [0, 1] (90%); stub 3 answered from [0, 1] (90%); stub 4 answered from [0, 1] (90%); stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; 2 rows 0x6eec apart at `0xc061e0`, slices +0x8:0x4, +0x56fe:0x20 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p10+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 500 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_object_data_has_general.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(0);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // `a[0]`, `a[1]` the mask and the type argument; `a[2..8]` the answers of slots
            // 0x18, 0xc4, 0xb8 (a flag `a[8]`: inline, the type record's slot 0x60 `a[6]`), 0x20,
            // that slot 0x60, and `find_hero` (the direct callee). The owner (`this+9`) and unit
            // index (`this+0xa`) pick the unit record p[6 + 2 * owner + index]; the leader row
            // `owner` has its `+8` in p[2 * owner] and its shorts from `+0x56fe` in p[2 * owner + 1].
            let (mask, kind) = (a[0], a[1]);
            let owner = m[9] as usize;
            let index = i16::from_le_bytes([m[0xa], m[0xb]]);
            let unit = &p[6 + 2 * owner + index as usize];
            let found = a[2] != 0 && a[3] != 0 && (mask == 0 || rd(unit, 0x68) & mask != 0) && {
                kind < 0 || (if a[8] != 0 { a[6] != 0 } else { a[4] != 0 })
            };
            log.push("-".into());
            if found {
                i64::from(index)
            } else {
                let rec = &p[10];
                let extra = if a[5] != 0 {
                    rd(rec, 0x238)
                        .wrapping_add(rd(rec, 0x234))
                        .wrapping_mul(0x60)
                } else {
                    0
                };
                let x10 = rd(m, 0x10) ^ 0x63637;
                let x14 = rd(m, 0x14) ^ 0x63637;
                let shorts = &p[2 * owner + 1];
                let short_at = (kind >= 0).then(|| {
                    let o = 2 * kind as usize;
                    i16::from_le_bytes([shorts[o], shorts[o + 1]])
                });
                if short_at == Some(0) {
                    -1
                } else {
                    log[0] = format!(
                        "{x10},{x10},{x14},{},{mask},{kind},{extra}",
                        rd(&p[2 * owner], 0)
                    );
                    i64::from(a[7])
                }
            }
        };
        assert_eq!(got, want_eax, "has_general — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "has_general — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "has_general — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "has_general — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "has_general — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 500, "only {n} rows");
    eprintln!("ObjectData::has_general@00646b00: {n} rows agree");
}
