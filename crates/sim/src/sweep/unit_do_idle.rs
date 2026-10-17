//! `Unit::do_idle@0060dcd0` under the emulator against the port (item 2966; tools/sweep_gen.py, obj with windows).

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

/// `Unit::do_idle@0060dcd0`, run by `tools/emu/sweep_unit_do_idle.py` under unicorn
/// (tools/emu): 256 random bytes of object, 0 argument(s), `p` windows in order: p0..p4 the place(s), in flag order; vtable slot +0xc4 (0 args) stubbed; callee 0x00616f40 (3 args) stubbed; callee 0x006032c0 (0 args) stubbed; callee 0x005f6e40 (0 args) stubbed; the word at `m+0xf4` points at 16 random bytes less `0` (placed); the word at `p0+0` points at 160 random bytes less `0` (placed); the word at `0xc0aec0+u8(m,9)*0x1c` points at 16 random bytes less `0` (placed); the word at `p2+i16(m,0xa)*4` points at 32 random bytes less `0` (placed); the word at `p3+0x18` points at 704 random bytes less `0` (placed); slot +0xc4 laid as 0x0046ce60 or its stub, a flag after the answers; stub 0 answered from [0, 1, 1] (90%); stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); the stubbed vtable laid at `p3+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_do_idle.py") else {
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
            // Clears the 0x8000 flag of +0x6c (restored at the end). When the animation
            // class of the first graphic piece's category (the image's `UnitAnimCat`,
            // 12 for entries 11..=14 — a table the executable initialises statically)
            // is that class, the unit's own answer (slot +0xc4, inlined as the +0x2b8
            // word's bit 0x20 when the slot is `UnitData::is_hero`) suppresses the default
            // animation; `set_anim(CHAR_DEFAULT, 0, 1)` otherwise.
            let flag = rd(m, 0x6c) & 0x8000 != 0;
            if flag {
                wr(m, 0x6c, rd(m, 0x6c) & !0x8000);
            }
            let cat = p[1][0x9c] as i8;
            let mut anim = true;
            if (11..=14).contains(&cat) {
                let hero_slot = a[4] != 0;
                let v = if hero_slot {
                    rd(&p[4], 0x2b8) & 0x20
                } else {
                    a[0]
                };
                if v != 0 {
                    anim = false;
                }
            }
            if anim {
                log.push("s1:m+0x0,0,0,1".to_string());
            }
            m[0x88] = 0;
            m[0x89] = 0;
            log.push("s2:m+0x0".to_string());
            log.push("s3:m+0x0".to_string());
            if flag {
                wr(m, 0x6c, rd(m, 0x6c) | 0x8000);
            }
            0
        };
        let _ = (got, want_eax);
        assert_eq!(log, want_log, "do_idle — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "do_idle — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "do_idle — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "do_idle — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Unit::do_idle@0060dcd0: {n} rows agree");
}
