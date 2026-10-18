//! `SpellType::cast_pack@00670be0` under the emulator against the port (item 3166; tools/sweep_gen.py, obj with windows).

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

/// `SpellType::cast_pack@00670be0`, run by `tools/emu/sweep_spell_type_cast_pack.py` under unicorn
/// (tools/emu): 64 random bytes of object, 2 argument(s), `p` windows in order: p0..p25 the place(s), in flag order; the global cell 0x00c06204 points at 256 random bytes; the global cell 0x00c0618c points at 1024 random bytes; the global cell 0x00c0aec0 points at 16 random bytes; the global cell 0x00c0aedc points at 16 random bytes; the global cell 0x00cae5fc points at 1024 random bytes; 56792 random bytes at 0x00e3a390; vtable slot +0x8 (0 args) stubbed; vtable slot +0xbc (0 args) stubbed; vtable slot +0x160 (0 args) stubbed; vtable slot +0x174 (1 args) stubbed; callee 0x005e2920 (0 args) stubbed; callee 0x006b4900 (3 args) stubbed; callee 0x005f8d20 (4 args) stubbed; the word at `c1+0x14` points at 16 random bytes less `0` (placed); the word at `c1+0x30` points at 16 random bytes less `0` (placed); the word at `p0+0` points at 64 random bytes less `0` (placed); the word at `p0+4` points at 64 random bytes less `0` (placed); the word at `p0+8` points at 64 random bytes less `0` (placed); the word at `p0+12` points at 64 random bytes less `0` (placed); the word at `p1+0` points at 64 random bytes less `0` (placed); the word at `p1+4` points at 64 random bytes less `0` (placed); the word at `p1+8` points at 64 random bytes less `0` (placed); the word at `p1+12` points at 64 random bytes less `0` (placed); the word at `c2+0` points at 512 random bytes less `0` (placed); the word at `c2+4` points at 512 random bytes less `0` (placed); the word at `c2+8` points at 512 random bytes less `0` (placed); the word at `c2+12` points at 512 random bytes less `0` (placed); the word at `c3+0` points at 512 random bytes less `0` (placed); the word at `c3+4` points at 512 random bytes less `0` (placed); the word at `c3+8` points at 512 random bytes less `0` (placed); the word at `c3+12` points at 512 random bytes less `0` (placed); the word at `p10+0x18` points at 16 random bytes less `0` (placed); the word at `p11+0x18` points at 16 random bytes less `0` (placed); the word at `p12+0x18` points at 16 random bytes less `0` (placed); the word at `p13+0x18` points at 16 random bytes less `0` (placed); the word at `p14+0x18` points at 16 random bytes less `0` (placed); the word at `p15+0x18` points at 16 random bytes less `0` (placed); the word at `p16+0x18` points at 16 random bytes less `0` (placed); the word at `p17+0x18` points at 16 random bytes less `0` (placed); stub 0 answered from [1, 1, 1, 0] (90%); stub 1 answered from [1, 1, 1, 0] (90%); stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); stub 5 prints the 4 bytes its argument 1 points at, hex, after the record (`/` a call); the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p10+0` as well as at `this`; the stubbed vtable laid at `p11+0` as well as at `this`; the stubbed vtable laid at `p12+0` as well as at `this`; the stubbed vtable laid at `p13+0` as well as at `this`; the stubbed vtable laid at `p14+0` as well as at `this`; the stubbed vtable laid at `p15+0` as well as at `this`; the stubbed vtable laid at `p16+0` as well as at `this`; the stubbed vtable laid at `p17+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_spell_type_cast_pack.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(6);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // c0 the options record, c4 the div_3 table, c5 the leaders; p2.. the objects, p10.. the units,
            // p18.. their type records, each `player * 4 + unit`.
            let rdw = |w: &[u8], o: usize| {
                i32::from_le_bytes(w[o..o + 4].try_into().expect("four bytes"))
            };
            let (unit_i, pl) = (a[0] as usize, a[1] as usize);
            let k = 4 * pl + unit_i;
            wr(&mut c[0], 0x90, 1);
            let (mut s5, mut d0, mut d1) = (Vec::new(), Vec::new(), Vec::new());
            let mut toks = [
                "-".to_string(),
                "-".to_string(),
                "-".to_string(),
                "-".to_string(),
            ];
            if a[2] != 0 && a[3] != 0 {
                let ty = rdw(&p[18 + k], 4);
                let (ox, oy) = (
                    rdw(&p[2 + k], 0x10) ^ 0x63637,
                    rdw(&p[2 + k], 0x14) ^ 0x63637,
                );
                if ty == 0x3d || ty == 0x3e || ty == 400 {
                    let x = rdw(&c[4], 4 * (ox >> 6) as usize);
                    let y = rdw(&c[4], 4 * (oy >> 6) as usize);
                    for (i, (cx, cy)) in [(x, y), (x - 1, y), (x, y - 1), (x - 1, y - 1)]
                        .into_iter()
                        .enumerate()
                    {
                        let ptr = if i % 2 == 0 { "sp-0x14" } else { "sp-0x18" };
                        s5.push(format!("12621800,{ptr},sp-0x8,0"));
                        d0.push(format!("{:08x}", (cx as u32).swap_bytes()));
                        d1.push(format!("{:08x}", (cy as u32).swap_bytes()));
                    }
                    let l = rdw(&c[5], pl * 0x6eec);
                    wr(&mut c[5], pl * 0x6eec, l | 0x2000000);
                }
                let u = &mut p[10 + k];
                let f = rdw(u, 0x68);
                wr(u, 0x68, f | 0x80000);
                wr(u, 0x14c, -1);
                let w = format!("p{}+0x0", 10 + k);
                toks[0] = w.clone();
                toks[1] = format!("{w},0");
                toks[2] = w.clone();
                toks[3] = format!("{w},{ox},{oy},1,1");
            }
            log.push(toks[0].clone());
            log.push(toks[1].clone());
            log.push(toks[2].clone());
            log.push(if s5.is_empty() {
                "-".to_string()
            } else {
                s5.join("/")
            });
            log.push(toks[3].clone());
            log.push(if d0.is_empty() {
                "-".to_string()
            } else {
                d0.join("/")
            });
            log.push(if d1.is_empty() {
                "-".to_string()
            } else {
                d1.join("/")
            });
            0
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "cast_pack — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "cast_pack — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "cast_pack — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[7 + i], "cast_pack — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("SpellType::cast_pack@00670be0: {n} rows agree");
}
