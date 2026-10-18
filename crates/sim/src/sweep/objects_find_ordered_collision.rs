//! `Objects::find_ordered_collision@0065b440` under the emulator against the port (item 3261; tools/sweep_gen.py, obj with windows).

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

/// `Objects::find_ordered_collision@0065b440`, run by `tools/emu/sweep_objects_find_ordered_collision.py` under unicorn
/// (tools/emu): 520 random bytes of object, 4 argument(s), `p` windows in order: p0..p15 sparse table 0 (16 rows × 1 slice(s), row-major), p16..p49 the place(s), in flag order; the global cell 0x00c06188 points at 320 random bytes; the global cell 0x00cae5fc points at 1024 random bytes; the global cell 0x00e85f20 points at 7488 random bytes; vtable slot +0x8 (0 args) stubbed; vtable slot +0xbc (0 args) stubbed; the word at `m+0x14` points at 36 random bytes less `0` (placed); the word at `p16+0` points at 64 random bytes less `0` (placed); the word at `p16+4` points at 64 random bytes less `0` (placed); the word at `p16+8` points at 64 random bytes less `0` (placed); the word at `m+0x30` points at 36 random bytes less `0` (placed); the word at `p20+0` points at 64 random bytes less `0` (placed); the word at `p20+4` points at 64 random bytes less `0` (placed); the word at `p20+8` points at 64 random bytes less `0` (placed); the word at `m+0xf4` points at 36 random bytes less `0` (placed); the word at `p24+0` points at 64 random bytes less `0` (placed); the word at `p24+4` points at 64 random bytes less `0` (placed); the word at `p24+8` points at 64 random bytes less `0` (placed); the word at `0xc0aec0` points at 36 random bytes less `0` (placed); the word at `p28+0` points at 144 random bytes less `0` (placed); the word at `p28+4` points at 144 random bytes less `0` (placed); the word at `p28+8` points at 144 random bytes less `0` (placed); the word at `p29+0x18` points at 608 random bytes less `0` (placed); the word at `p30+0x18` points at 608 random bytes less `0` (placed); the word at `p31+0x18` points at 608 random bytes less `0` (placed); the word at `0xc0aedc` points at 36 random bytes less `0` (placed); the word at `p35+0` points at 144 random bytes less `0` (placed); the word at `p35+4` points at 144 random bytes less `0` (placed); the word at `p35+8` points at 144 random bytes less `0` (placed); the word at `p36+0x18` points at 608 random bytes less `0` (placed); the word at `p37+0x18` points at 608 random bytes less `0` (placed); the word at `p38+0x18` points at 608 random bytes less `0` (placed); the word at `0xc0ab84` points at 36 random bytes less `0` (placed); the word at `p42+0` points at 32 random bytes less `0` (placed); the word at `p42+4` points at 32 random bytes less `0` (placed); the word at `p42+8` points at 32 random bytes less `0` (placed); the word at `0xc0aba0` points at 36 random bytes less `0` (placed); the word at `p46+0` points at 32 random bytes less `0` (placed); the word at `p46+4` points at 32 random bytes less `0` (placed); the word at `p46+8` points at 32 random bytes less `0` (placed); stub 0 answered from [1, 1, 1, 0] (90%); stub 1 answered from [1, 1, 1, 0] (90%); the stubbed vtable laid at `p17+0`; stub 0 answers 8 values in call order, printed after the answers; stub 1 answers 8 values in call order, printed after the answers; the global cell 0x00c0618c holds `this`; 16 rows 0x1c apart at `c0+0x134`, slices +0x8:0x4 (`p` windows after the pointed buffers, before the places); figures p17..p19 link at +0x2c (4 bytes) below their own index or to -1, 65536, 65537, 65538, 524288, 524289, 524290; the stubbed vtable laid at `p17+0` as well as at `this`; the stubbed vtable laid at `p18+0` as well as at `this`; the stubbed vtable laid at `p19+0` as well as at `this`; the stubbed vtable laid at `p21+0` as well as at `this`; the stubbed vtable laid at `p22+0` as well as at `this`; the stubbed vtable laid at `p23+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 10000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_objects_find_ordered_collision.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (lhs, rhs) = line.split_once(" -> ").expect("a row is `<args…> | <windows> -> <eax> | <windows>`");
        let (args, before) = lhs.split_once(" | ").expect("`<args> | <before>`");
        let (eax, after) = rhs.split_once(" | ").expect("`<eax> | <after>`");
        let a: Vec<i32> = args.split_whitespace().map(|s| s.parse().expect("an integer")).collect();
        let mut wins: Vec<Vec<u8>> = before.trim().split(';').map(unhex).collect();
        let mut m = wins.remove(0);
        let mut p: Vec<Vec<u8>> = wins.split_off(3);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // `a`: x, y, o, who; then the two stubs' one answer each (unread: both are `--seq`), and
            // the call-order answers of slot +8 (8) and slot +0xbc (8). c0 the world record (`+0`
            // width, `+4` height), c1 `div_3_table`, c2 the groups (0x9d4 apiece); p0..p15 the world
            // cells' head words; p16/p20/p24 the objects' pointer arrays of who 0/1/8 over objects
            // p17..19 / p21..23 / p25..27; p28/p35 the units' pointer arrays of who 0/1 over records
            // p29..31 / p36..38 and types p32..34 / p39..41; p42/p46 the static `objects` arrays.
            let (x, y, o, who) = (a[0], a[1], a[2], a[3]);
            let (seq8, seq_bc) = (&a[6..14], &a[14..22]);
            let div3 = |k: i32| rd(&c[1], 4 * k as usize);
            let rd16 = |w: &[u8], off: usize| i32::from(i16::from_le_bytes([w[off], w[off + 1]]));
            let unit = |who: i32, idx: i32| &p[(if who == 0 { 29 } else { 36 }) + idx as usize];
            let ty = |who: i32, idx: i32| &p[(if who == 0 { 32 } else { 39 }) + idx as usize];
            let mine = rd(ty(who, o), 0x248);
            let (ux, uy) = (div3(x >> 4), div3(y >> 4));
            // `unit +0x70/+0x74` sar 4 into `div_3_table`: where the unit was ordered to.
            let at = |w: &[u8]| (div3(rd(w, 0x70) >> 4), div3(rd(w, 0x74) >> 4));
            let (mut t8, mut tbc) = (0usize, 0usize);
            let r = mine != 0 && {
                // `objects+0x200 = -1`, `+0x1fc = 99999999` (`65b496`, `65b4a2`).
                wr(m, 0x200, -1);
                wr(m, 0x1fc, 99_999_999);
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
                    |(idx, w)| {
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
                        Some((r, at(unit(w, idx))))
                    },
                )
            };
            // The second pass (`65b680`): the asker's `unit +0x80` group, a record of 0x9d4 in c2;
            // its owner byte `+0x4a` is `who`, the asker's static-`objects` entry has bit 0 of `+8`
            // (the crate carries neither that flag nor the group's `+0x49`, which stops the pass: a
            // SEAM, passed as the pass's own gates here), and the member list is `+0x8cc` shorts,
            // `+0xc` of them.
            let r = r || {
                let me = unit(who, o);
                let g = rd16(me, 0x80);
                let rec = (g.max(0) as usize) * 0x9d4;
                let grp = &c[2][rec..];
                let flag = p[(if who == 0 { 42 } else { 46 }) + 1 + o as usize][8] & 1 != 0;
                let count = (rd(grp, 0xc).max(0) as usize).min(8);
                let list: Vec<usize> = (0..count).map(|i| rd16(grp, 0x8cc + 2 * i) as usize).collect();
                let open = g >= 0 && flag && grp[0x49] == 0;
                crate::collide::ordered_group_steps(
                    open.then_some((grp[0x4a], &list[..])),
                    o as usize,
                    who as crate::Player,
                    mine,
                    (ux, uy),
                    |m| {
                        let w = unit(who, m as i32);
                        // `unit +8` bit 0 and `+0x82 < 0`: on the map, and the crate's `alive`.
                        if rd(w, 8) & 1 == 0 || rd16(w, 0x82) >= 0 {
                            return None;
                        }
                        let r = rd(ty(who, m as i32), 0x248);
                        (r != 0).then_some((r, at(w)))
                    },
                )
            };
            i64::from(r)
        };
        assert_eq!(got, want_eax, "find_ordered_collision — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "find_ordered_collision — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "find_ordered_collision — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[4 + i], "find_ordered_collision — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 10000, "only {n} rows");
    eprintln!("Objects::find_ordered_collision@0065b440: {n} rows agree");
}
