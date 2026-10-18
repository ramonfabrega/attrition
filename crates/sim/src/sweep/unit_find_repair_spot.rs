//! `Unit::find_repair_spot@00604320` under the emulator against the port (item 3219; tools/sweep_gen.py, obj with windows).

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

/// `Unit::find_repair_spot@00604320`, run by `tools/emu/sweep_unit_find_repair_spot.py` under unicorn
/// (tools/emu): 320 random bytes of object, 0 argument(s), `p` windows in order: p0..p11 the place(s), in flag order; the global cell 0x00c0618c points at 128 random bytes; the global cell 0x00cae5fc points at 4096 random bytes; the global cell 0x00c06188 points at 320 random bytes; the global cell 0x00c061f0 points at 64 random bytes; the global cell 0x00c061e8 points at 2304 random bytes; the global cell 0x00c061e0 points at 131072 random bytes; 56792 random bytes at 0x00e3a390 (each word in -1..3); callee 0x00659ca0 (12 args) stubbed; callee 0x00713e80 (1 args) stubbed; callee 0x00714350 (4 args) stubbed; callee 0x0070fbe0 (5 args) stubbed; vtable slot +0xf8 (0 args) stubbed; vtable slot +0x104 (1 args) stubbed; vtable slot +0x134 (1 args) stubbed; the word at `c0+0x14` points at 16 random bytes less `0` (placed); the word at `p0+0` points at 32 random bytes less `0` (placed); the word at `p0+4` points at 32 random bytes less `0` (placed); the word at `p0+8` points at 32 random bytes less `0` (placed); the word at `p0+12` points at 32 random bytes less `0` (placed); the word at `c0+0x30` points at 16 random bytes less `0` (placed); the word at `p5+0` points at 32 random bytes less `0` (placed); the word at `p5+4` points at 32 random bytes less `0` (placed); the word at `p5+8` points at 32 random bytes less `0` (placed); the word at `p5+12` points at 32 random bytes less `0` (placed); the word at `c2+0x134` points at 12288 random bytes less `0` (placed); the word at `m+0x18` points at 32 random bytes less `0` (placed); slot +0xf8 laid as 0x006109f0 or its stub, a flag after the answers; slot +0x104 laid as 0x0046ced0 or its stub, a flag after the answers; stub 4 answered from [0, 1, 2] (90%); stub 5 answered from [0, 1] (90%); stub 6 answered from [0, 1] (90%); stub 4 answers 4 values in call order, printed after the answers; stub 5 answers 4 values in call order, printed after the answers; stub 6 answers 4 values in call order, printed after the answers; stub 0 answers -2..3 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the stubbed vtable laid at `p11+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1500 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_find_repair_spot.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(7);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // Stub answers: `a[0]` `find_any_building`; slot `+0xf8` (`a[7..11]` in call order), `+0x104`
                // (`a[11..15]`), the type's `+0x134` (`a[15..19]`); `a[19]`/`a[20]` say whether slots `+0xf8`
                // / `+0x104` held `UnitData::get_worker_stance` / `has_stance_type` (run, not stubbed) or the stub.
                // Windows: c1 `div_3_table`, c2 `GameAccess::world`, c3 `constants`, c4 `gamec`, c6 the
                // leaders; p0..p4 / p5..p9 the objects tables of players 0 / 1, p10 the world's tile records
                // (stride 0x1c, `owner` the signed byte `+0xf`), c5 `leadersc`.
                const K: i32 = 0x63637;
                let who = m[9] as usize;
                let lead = |o: usize| rd(&c[6], who * 0x6eec + o);
                let mut cnt = [0usize; 3];
                // k: 0 the `+0xf8` stub, 1 the `+0x104` stub, 2 the type's `+0x134` stub.
                let mut ans = |k: usize| {
                    cnt[k] += 1;
                    a[7 + 4 * k + (cnt[k] - 1).min(3)]
                };
                let stance = i32::from(m[0xb1] as i8);
                let mut toks = ["-".to_string(), "-".into(), "-".into(), "-".into()];
                let mut range = rd(&c[3], 0x24).wrapping_mul(0xc0);
                let mut go = true;
                if lead(0) & 4 != 0 {
                    // `get_worker_stance` returns `WORKER_STAND` (3) with no stance type.
                    let first = if a[19] != 0 {
                        let has = ans(if a[20] != 0 { 2 } else { 1 });
                        (has != 0).then_some(stance)
                    } else {
                        Some(ans(0))
                    };
                    let mut dbl = first == Some(1);
                    if !dbl {
                        let v = if a[19] != 0 {
                            let has = ans(if a[20] != 0 { 2 } else { 1 });
                            if has != 0 { stance } else { 3 }
                        } else {
                            ans(0)
                        };
                        dbl = v == 2;
                    }
                    if dbl {
                        range = range.wrapping_mul(2);
                    }
                } else {
                    let g = &c[4];
                    let diff = if g[0x820] & 4 != 0 {
                        lead(0x50)
                    } else if g[0x821] & 0x10 != 0 && g[0x822] & 2 == 0 {
                        let d = lead(0x50);
                        if d < 0 { i32::from(g[0x2b]) } else { d }
                    } else {
                        i32::from(g[0x2b])
                    };
                    go = diff > 1;
                }
                let mut ret = 0;
                if go {
                    let (x, y) = (rd(m, 0x10) ^ K, rd(m, 0x14) ^ K);
                    toks[0] = format!("c0+0x0,{x},{y},1,{who},{range},512,7,-1,-1,8,0,0");
                    let b = a[0];
                    if b >= 0 {
                        let rec = &p[1 + 5 * who + b as usize];
                        let tile = |o: usize| rd(&c[1], 4 * ((rd(rec, o) ^ K) >> 8) as usize);
                        let idx = tile(0x14)
                            .wrapping_mul(rd(&c[2], 0))
                            .wrapping_add(tile(0x10));
                        let owner = i32::from(p[10][(idx as usize).wrapping_mul(0x1c) + 0xf] as i8);
                        let mut ok = true;
                        if owner >= 0 {
                            let me = lead(8);
                            if owner != me {
                                ok = lead(0x74 + owner as usize * 4) == 2
                                    && rd(&c[5], ((owner * 0x1bbb + me) * 4 + 0x74) as usize) == 2;
                            }
                        }
                        if ok {
                            let g = "sp-0x9e8";
                            toks[1] = format!("{g},-1/{g},-1");
                            let ty = i16::from_le_bytes([m[0xa], m[0xb]]);
                            toks[2] = format!("{g},{ty},{who},0,0");
                            toks[3] = format!("{g},{b},{who},1,13,0");
                            ret = 1;
                        }
                    }
                }
                log.extend(toks);
                ret
            }
        };
        assert_eq!(got, want_eax, "find_repair_spot — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "find_repair_spot — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "find_repair_spot — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "find_repair_spot — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[8 + i],
                "find_repair_spot — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 1500, "only {n} rows");
    eprintln!("Unit::find_repair_spot@00604320: {n} rows agree");
}
