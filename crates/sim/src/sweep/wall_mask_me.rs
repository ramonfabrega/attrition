//! `Wall::mask_me@00642fc0` under the emulator against the port (item 3044; tools/sweep_gen.py, obj with windows).

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

/// `Wall::mask_me@00642fc0`, run by `tools/emu/sweep_wall_mask_me.py` under unicorn
/// (tools/emu): 32 random bytes of object, 2 argument(s), `p` windows in order: p0..p1 sparse table 0 (2 rows × 1 slice(s), row-major), p2..p15 the place(s), in flag order; the global cell 0x00c0618c points at 128 random bytes; vtable slot +0xac (0 args) stubbed; callee 0x006db790 (1 args) stubbed; callee 0x00738410 (0 args) stubbed; callee 0x006312a0 (8 args) stubbed; stub 0's answer points at 128 random bytes less `0` (placed); the word at `c0+0x14` points at 16 random bytes less `0` (placed); the word at `c0+0x30` points at 16 random bytes less `0` (placed); the word at `p3+0` points at 64 random bytes less `0` (placed); the word at `p3+4` points at 64 random bytes less `0` (placed); the word at `p4+0` points at 64 random bytes less `0` (placed); the word at `p4+4` points at 64 random bytes less `0` (placed); the word at `p0+0` points at 16 random bytes less `0` (placed); the word at `p1+0` points at 16 random bytes less `0` (placed); the word at `p9+0` points at 32 random bytes less `0` (placed); the word at `p9+4` points at 32 random bytes less `0` (placed); the word at `p10+0` points at 32 random bytes less `0` (placed); the word at `p10+4` points at 32 random bytes less `0` (placed); the word at `m+0x18` points at 16 random bytes less `0` (placed); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); 2 rows 0x1c apart at `@0xc09970`, slices +0x0:0x4 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 400 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_wall_mask_me.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (line, calls) = line.split_once(" # ").expect("`<row> # <calls>`");
        let want_log: Vec<&str> = calls.split_whitespace().collect();
        let (lhs, rhs) = line.split_once(" -> ").expect("a row is `<args…> | <windows> -> <eax> | <windows>`");
        let (args, before) = lhs.split_once(" | ").expect("`<args> | <before>`");
        let (eax, after) = rhs.split_once(" | ").expect("`<eax> | <after>`");
        let a: Vec<i32> = args.split_whitespace().map(|s| s.parse().expect("an integer")).collect();
        let mut wins: Vec<Vec<u8>> = before.trim().split(';').map(unhex).collect();
        let mut m = wins.remove(0);
        let mut p: Vec<Vec<u8>> = wins.split_off(1);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // `m[8] & 0x20` asks for a radius: the unit `p[5 + 2 * owner + index]`'s slot 0xac (stub 0, the
            // record `p[2]`) names a city by the short at `+0x72`; negative: the leader's radius (stub 1,
            // handed the type's `+4`), else the city's (stub 2, ecx the word `p[9 + owner]` holds at
            // that index, a placed city `p[11 + 2 * owner + city]` for 0 and 1). Then `mask_me` (stub 3)
            // on the type `p[15]`: the masked corner, the radius, the two arguments, the corner's y,
            // the object's index and owner, the second argument.
            const K: i32 = 0x63637;
            let (owner, index) = (usize::from(m[9]), i16::from_le_bytes([m[0xa], m[0xb]]) as usize);
            let mut tokens: Vec<String> = Vec::new();
            let mut radius = 0;
            if m[8] & 0x20 != 0 {
                tokens.push(format!("s0:p{}+0x0", 5 + 2 * owner + index));
                let city = i16::from_le_bytes([p[2][0x72], p[2][0x73]]);
                if city < 0 {
                    let ecx = 0xe3a390u32.wrapping_add(0x6eec_u32.wrapping_mul(owner as u32));
                    tokens.push(format!("s1:{},{}", ecx as i32, rd(&p[15], 4)));
                    radius = a[3];
                } else {
                    let word = rd(&p[9 + owner], 4 * city as usize);
                    tokens.push(if city < 2 {
                        format!("s2:p{}+0x0", 11 + 2 * owner + city as usize)
                    } else {
                        format!("s2:{word}")
                    });
                    radius = a[4];
                }
            }
            let (x, y) = (rd(m, 0x10) ^ K, rd(m, 0x14) ^ K);
            tokens.push(format!("s3:p15+0x0,{x},{y},{radius},{},{y},{index},{owner},{}", a[0], a[1]));
            log = tokens;
            0
        };
        let _ = (got, want_eax);
        assert_eq!(log, want_log, "mask_me — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "mask_me — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "mask_me — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[2 + i], "mask_me — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 400, "only {n} rows");
    eprintln!("Wall::mask_me@00642fc0: {n} rows agree");
}
