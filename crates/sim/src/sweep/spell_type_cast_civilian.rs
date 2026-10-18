//! `SpellType::cast_civilian@006704a0` under the emulator against the port (item 3150; tools/sweep_gen.py, obj with windows).

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

/// `SpellType::cast_civilian@006704a0`, run by `tools/emu/sweep_spell_type_cast_civilian.py` under unicorn
/// (tools/emu): 16 random bytes of object, 2 argument(s), `p` windows in order: p0..p5 the place(s), in flag order; the global cell 0x00e85ddc points at 256 random bytes; the global cell 0x00c06204 points at 256 random bytes; 64 random bytes at 0x00e3a390; callee 0x006d9380 (1 args) stubbed; vtable slot +0x68 (1 args) stubbed; callee 0x006e3140 (1 args) stubbed; vtable slot +0x80 (2 args) stubbed; vtable slot +0x84 (2 args) stubbed; vtable slot +0x11c (1 args) stubbed; the word at `0xc0aec0` points at 16 random bytes less `0` (placed); the word at `p0+0` points at 256 random bytes less `0` (placed); the word at `p0+4` points at 256 random bytes less `0` (placed); the word at `p0+8` points at 256 random bytes less `0` (placed); the word at `p0+12` points at 256 random bytes less `0` (placed); the word at `c0+0xc8` points at 16 random bytes less `0` (placed); slot +0x11c laid as 0x00610890 or its stub, a flag after the answers; slot +0x80 laid as 0x0046ce10 or its stub, a flag after the answers; stub 0 answered from [0, 1] (90%); stub 5 answers 2 values in call order, printed after the answers; stub 5 answers 1..50 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p1+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 600 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_spell_type_cast_civilian.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(3);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            let rd = |w: &[u8], o: usize| {
                i32::from_le_bytes(w[o..o + 4].try_into().expect("four bytes"))
            };
            let wr = |w: &mut [u8], o: usize, v: i32| w[o..o + 4].copy_from_slice(&v.to_le_bytes());
            // c1 the options cell, c2 the leaders table, p1..p4 the units, p5 the civilian type; a[2..8] the
            // stubs' answers (tribe_can_type, the type's slot +0x68, current_upgrade, slot +0x80, slot +0x84,
            // slot +0x11c), a[8..10] the two `hits` answers, a[10] / a[11] the flags laying the image's `hits`
            // and `set_type`.
            let u = 1 + a[0] as usize;
            let (hits1, hits2) = if a[10] != 0 {
                (rd(&p[u], 0x20), rd(&p[u], 0x20))
            } else {
                (a[8], a[9])
            };
            let ratio = rd(&p[u], 0x24).wrapping_shl(8).wrapping_div(hits1);
            let mut l: [Vec<String>; 5] = Default::default();
            if rd(&p[u], 0x54) == 0 {
                l[0].push("c2+0x0,p5+0x0".to_string());
                let t = if a[2] != 0 {
                    0x32
                } else {
                    l[1].push(format!("p5+0x0,{}", rd(&c[2], 8)));
                    a[3]
                };
                wr(&mut p[u], 0x54, t);
            }
            let up = rd(&p[u], 0x54);
            l[2].push(format!("c2+0x0,{up}"));
            if a[11] != 0 {
                l[4].push(format!("p{u}+0x0,{},0", a[4]));
            } else {
                l[3].push(format!("p{u}+0x0,{},0", a[4]));
            }
            let v = hits2.wrapping_mul(ratio);
            wr(&mut p[u], 0x24, v.wrapping_add((v >> 31) & 0xff) >> 8);
            p[u][0xaa] = 9;
            wr(&mut c[1], 0x90, 1);
            for x in l {
                log.push(if x.is_empty() {
                    "-".to_string()
                } else {
                    x.join("/")
                });
            }
            0
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "cast_civilian — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "cast_civilian — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "cast_civilian — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[4 + i],
                "cast_civilian — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 600, "only {n} rows");
    eprintln!("SpellType::cast_civilian@006704a0: {n} rows agree");
}
