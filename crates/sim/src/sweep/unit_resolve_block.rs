//! `Unit::resolve_block@005fccc0` under the emulator against the port (item 3150; tools/sweep_gen.py, obj with windows).

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

/// `Unit::resolve_block@005fccc0`, run by `tools/emu/sweep_unit_resolve_block.py` under unicorn
/// (tools/emu): 256 random bytes of object, 0 argument(s), `p` windows in order: p0..p5 the place(s), in flag order, p6 stub 2's answer window; the global cell 0x00c0618c points at 1024 random bytes; the global cell 0x00cae5fc points at 1024 random bytes; 256 random bytes at 0x00e3a390; callee 0x00607c30 (8 args) stubbed; callee 0x00659180 (7 args) stubbed; vtable slot +0xb0 (0 args) stubbed; vtable slot +0x90 (0 args) stubbed; callee 0x006ebaa0 (1 args) stubbed; callee 0x005e5410 (5 args) stubbed; the word at `c0+0x14` points at 16 random bytes less `0` (placed); the word at `p0+0` points at 32 random bytes less `0` (placed); the word at `p0+4` points at 32 random bytes less `0` (placed); the word at `p0+8` points at 32 random bytes less `0` (placed); the word at `p0+12` points at 32 random bytes less `0` (placed); the word at `m+0xb8` points at 64 random bytes less `0` (placed); stub 0 answered from [1, 1, 1, 0] (90%); stub 3 answered from [0, 1] (90%); stub 4 answered from [0, 1] (90%); stub 2 answers the address of 128 random bytes, a `p` window after the places; the word at `p6+0x18` laid as the address of `p6` every row; stub 0 answers 2 values in call order, the last from [0, 0, 1], printed after the answers; stub 4 answers 2 values in call order, printed after the answers; stub 1 answers -1..3 every row; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p1+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1200 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_resolve_block.py") else {
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
            // c0 the objects cell, c1 the div_3_table, c2 the leaders; p1..p4 the units, p5 the orders' array, p6
            // the slot +0xb0 answer; a[1] the building search, a[3] slot +0x90, a[6..8] the two `invalid_loc`
            // answers, a[8..10] the two `is_enemy` answers.
            let mut l: [Vec<String>; 5] = Default::default();
            let e = (rd(m, 0xc0).max(1) - 1) as usize;
            let (tx, ty) = (rd(&p[5], 16 * e) >> 6, rd(&p[5], 16 * e + 4) >> 6);
            let (tx, ty) = (rd(&c[1], 4 * tx as usize), rd(&c[1], 4 * ty as usize));
            let mut eax = 0;
            if a[6] != 0 && a[7] == 0 {
                l[0].push(format!("c0+0x0,{tx},{ty},c1+0x0,{},0,0,c1+0x0", m[9]));
                let idx = a[1];
                if idx >= 0 {
                    let owner = rd(&c[0], 0x200);
                    if owner == i32::from(m[9]) {
                        l[1].push(format!("p{}+0x0", 1 + idx as usize));
                        l[2].push("p6+0x0".to_string());
                        if a[3] == 0 {
                            l[1].push(format!("p{}+0x0", 1 + idx as usize));
                            let w = u16::from_le_bytes([p[6][0x60], p[6][0x61]]) | 1;
                            p[6][0x60..0x62].copy_from_slice(&w.to_le_bytes());
                        }
                    } else {
                        l[3].push(format!("c2+0x0,{owner}"));
                        if a[8] == 0 {
                            let o = 0xb4 + 4 * owner as usize;
                            let w = rd(&c[2], o) | 2;
                            c[2][o..o + 4].copy_from_slice(&w.to_le_bytes());
                        } else {
                            l[3].push(format!("c2+0x0,{owner}"));
                            if a[9] != 0 {
                                l[4].push(format!("m+0x0,{idx},{owner},0,0,0"));
                                eax = 1;
                            }
                        }
                    }
                }
            }
            for x in l {
                log.push(if x.is_empty() {
                    "-".to_string()
                } else {
                    x.join("/")
                });
            }
            eax
        };
        assert_eq!(got, want_eax, "resolve_block — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "resolve_block — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "resolve_block — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "resolve_block — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[4 + i],
                "resolve_block — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 1200, "only {n} rows");
    eprintln!("Unit::resolve_block@005fccc0: {n} rows agree");
}
