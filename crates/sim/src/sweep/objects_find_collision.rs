//! `Objects::find_collision@0065b1b0` under the emulator against the port (item 2400; tools/sweep_gen.py, obj with windows).

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

/// `Objects::find_collision@0065b1b0`, run by `tools/emu/sweep_objects_find_collision.py` under unicorn
/// (tools/emu): 520 random bytes of object, 5 argument(s), the global cell 0x00c06188 points at 320 random bytes; the global cell 0x00cae5fc points at 1024 random bytes; vtable slot +0x8 (0 args) stubbed; vtable slot +0xbc (0 args) stubbed; callee 0x00682540 (8 args) stubbed; the word at `m+0x14` points at 36 random bytes less `0` (placed); the word at `p16+0` points at 64 random bytes less `0` (placed); the word at `p16+4` points at 64 random bytes less `0` (placed); the word at `p16+8` points at 64 random bytes less `0` (placed); the word at `m+0x30` points at 36 random bytes less `0` (placed); the word at `p20+0` points at 64 random bytes less `0` (placed); the word at `p20+4` points at 64 random bytes less `0` (placed); the word at `p20+8` points at 64 random bytes less `0` (placed); the word at `m+0xf4` points at 36 random bytes less `0` (placed); the word at `p24+0` points at 64 random bytes less `0` (placed); the word at `p24+4` points at 64 random bytes less `0` (placed); the word at `p24+8` points at 64 random bytes less `0` (placed); the word at `0xc0aec0` points at 36 random bytes less `0` (placed); the word at `p28+0` points at 32 random bytes less `0` (placed); the word at `p28+4` points at 32 random bytes less `0` (placed); the word at `p28+8` points at 32 random bytes less `0` (placed); the word at `p29+0x18` points at 608 random bytes less `0` (placed); the word at `p30+0x18` points at 608 random bytes less `0` (placed); the word at `p31+0x18` points at 608 random bytes less `0` (placed); the word at `0xc0aedc` points at 36 random bytes less `0` (placed); the word at `p35+0` points at 32 random bytes less `0` (placed); the word at `p35+4` points at 32 random bytes less `0` (placed); the word at `p35+8` points at 32 random bytes less `0` (placed); the word at `p36+0x18` points at 608 random bytes less `0` (placed); the word at `p37+0x18` points at 608 random bytes less `0` (placed); the word at `p38+0x18` points at 608 random bytes less `0` (placed); stub 0 answered from [1, 1, 1, 0] (90%); stub 1 answered from [1, 1, 1, 0] (90%); stub 2 answered from [0, 1, 5, -3] (90%); the stubbed vtable laid at `p17+0`; stub 0 answers 8 values in call order, printed after the answers; stub 1 answers 8 values in call order, printed after the answers; stub 2 answers 1 values in call order, printed after the answers; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the global cell 0x00c0618c holds `this`; 16 rows 0x1c apart at `c0+0x134`, slices +0x8:0x4 (`p` windows after the pointed buffers, before the places); figures p17..p19 link at +0x2c (4 bytes) below their own index or to -1, 65536, 65537, 65538, 524288, 524289, 524290; the stubbed vtable laid at `p18+0` as well as at `this`; the stubbed vtable laid at `p19+0` as well as at `this`; the stubbed vtable laid at `p21+0` as well as at `this`; the stubbed vtable laid at `p22+0` as well as at `this`; the stubbed vtable laid at `p23+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 800 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_objects_find_collision.py") else {
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
            // `a`: x, y, o, who, the fifth argument; then the three stubs' one answer each (unread:
            // every stub is a `--seq`), and the call-order answers of slot +8 (8), slot +0xbc (8)
            // and `CollCheck::collide_here` (1). c0 the world record (`+0` width, `+4` height), c1
            // `div_3_table`; p0..p15 the world cells' head words (idx i16, who i16); p16/p20/p24 the
            // objects' pointer arrays of who 0/1/8 over objects p17..19 / p21..23 / p25..27; p29..31 /
            // p36..38 the units' records of who 0/1 and p32..34 / p39..41 their types.
            let (x, y, o, who, fifth) = (a[0], a[1], a[2], a[3], a[4]);
            let (seq8, seq_bc, seq_ch) = (&a[8..16], &a[16..24], a[24]);
            let div3 = |k: i32| rd(&c[1], 4 * k as usize);
            let rd16 = |w: &[u8], off: usize| i32::from(i16::from_le_bytes([w[off], w[off + 1]]));
            let ty = |who: i32, idx: i32| &p[(if who == 0 { 32 } else { 39 }) + idx as usize];
            let me_ty = ty(who, o);
            let (domain, mine) = (rd(me_ty, 0x218), rd(me_ty, 0x248));
            let (ux, uy) = (div3(x >> 4), div3(y >> 4));
            let (mut t8, mut tbc) = (0usize, 0usize);
            let mut scratch = false;
            let r = crate::collide::find_collision_steps(
                fifth != 0,
                domain == 0,
                mine,
                || {
                    log.push(format!("{},{o},{who},{ux},{uy},{mine},0,0,0", y >> 4));
                    seq_ch
                },
                || {
                    // `objects+0x200 = -1`, `+0x1fc = 99999999` (`65b252`, `65b25c`).
                    scratch = true;
                    let (wcx, wcy) = (div3(x >> 8), div3(y >> 8));
                    let (wide, high) = (rd(&c[0], 0), rd(&c[0], 4));
                    let obj = |(idx, who): (i32, i32)| {
                        &p[match who {
                            0 => 17,
                            1 => 21,
                            8 => 25,
                            _ => unreachable!("who {who}"),
                        } + idx as usize]
                    };
                    let link = |w: &[u8]| {
                        let (idx, who) = (rd16(w, 0x2c), rd16(w, 0x2e));
                        (idx >= 0).then_some((idx, who))
                    };
                    crate::collide::chain_hit_steps(
                        (ux, uy),
                        mine,
                        |dx, dy| {
                            let (cx, cy) = (wcx + dx, wcy + dy);
                            if cx < 0 || cy < 0 || cx >= wide || cy >= high {
                                return None;
                            }
                            let h = &p[(wide * cy + cx) as usize];
                            let (idx, who) = (rd16(h, 0), rd16(h, 2));
                            (idx >= 0).then_some((idx, who))
                        },
                        |n| link(obj(n)),
                        |n @ (idx, w)| {
                            if (idx, w) == (o, who) || w >= 8 {
                                return None;
                            }
                            // Slot +8, then — only if it answered — slot +0xbc.
                            let a8 = seq8[t8.min(7)];
                            t8 += 1;
                            if a8 == 0 {
                                return None;
                            }
                            let abc = seq_bc[tbc.min(7)];
                            tbc += 1;
                            if abc == 0 {
                                return None;
                            }
                            let r = rd(ty(w, idx), 0x248);
                            if r == 0 {
                                return None;
                            }
                            let ob = obj(n);
                            let at = |off| div3((rd(ob, off) ^ 0x63637) >> 4);
                            Some((r, (at(0x10), at(0x14))))
                        },
                    )
                },
            );
            if scratch {
                wr(m, 0x200, -1);
                wr(m, 0x1fc, 99_999_999);
            }
            if log.is_empty() {
                log.push("-".into());
            }
            i64::from(r)
        };
        assert_eq!(got, want_eax, "find_collision — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "find_collision — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "find_collision — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "find_collision — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[3 + i],
                "find_collision — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 800, "only {n} rows");
    eprintln!("Objects::find_collision@0065b1b0: {n} rows agree");
}
