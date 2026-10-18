//! `Guy::set_all_pivots@005d8bc0` under the emulator against the port (item 3261; tools/sweep_gen.py, obj with windows).

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

/// `Guy::set_all_pivots@005d8bc0`, run by `tools/emu/sweep_guy_set_all_pivots.py` under unicorn
/// (tools/emu): 176 random bytes of object, 0 argument(s), `p` windows in order: p0..p17 the place(s), in flag order, p18 stub 2's answer window; the global cell 0x00c06214 points at 1824 random bytes; the global cell 0x00c0618c points at 96 random bytes; the global cell 0x00c06378 points at 32 random bytes; 8 random bytes at 0x00eb13a8 (each word in -3..3); callee 0x00902540 (1 args) stubbed; callee 0x00616e80 (0 args) stubbed; callee 0x0060b030 (0 args) stubbed; vtable slot +0xd0 (0 args) stubbed; callee 0x0090b680 (4 args) stubbed; callee 0x0090b750 (9 args) stubbed; callee 0x00a3ab00 (10 args) stubbed; the word at `c1+0x14` points at 12 random bytes less `0` (placed); the word at `p0+0` points at 32 random bytes less `0` (placed); the word at `p0+4` points at 32 random bytes less `0` (placed); the word at `p0+8` points at 32 random bytes less `0` (placed); the word at `c1+0x30` points at 12 random bytes less `0` (placed); the word at `p4+0` points at 32 random bytes less `0` (placed); the word at `p4+4` points at 32 random bytes less `0` (placed); the word at `p4+8` points at 32 random bytes less `0` (placed); the word at `0xc0aec0` points at 12 random bytes less `0` (placed); the word at `p8+0` points at 16 random bytes less `0` (placed); the word at `p8+4` points at 16 random bytes less `0` (placed); the word at `p8+8` points at 16 random bytes less `0` (placed); the word at `0xc0aedc` points at 12 random bytes less `0` (placed); the word at `p12+0` points at 16 random bytes less `0` (placed); the word at `p12+4` points at 16 random bytes less `0` (placed); the word at `p12+8` points at 16 random bytes less `0` (placed); the word at `c0+0x70c` points at 256 random bytes less `0` (placed); stub 3's answer points at 16 random bytes less `0` (placed); stub 1 answered from [23, 23, 23, 23, 23, 22, 0, 5] (90%); stub 2 answers the address of 32 random bytes, a `p` window after the places; the stubbed vtable laid at `p18+0`; stub 0 answers 50..57 every row; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 writes a value in [-180, -100, -46, -45, -44, -10, 0, 10, 44, 45, 46, 100, 180] through its argument 2, printed after the answers (`o` in a place); stub 4 writes a value in [-180, -100, -46, -45, -44, -10, 0, 10, 44, 45, 46, 100, 180] through its argument 3, printed after the answers (`o` in a place); stub 5 writes a value in [0, 1065353216, -1082130432, 1056964608, -1090519040, 1076887552, -1070596096, 1120403456, -1027080192, 1133887488, -1013596160, 1099563008, -1031798784, 1150966989, -996516659] through its argument 7, printed after the answers (`o` in a place); the stack 512 bytes under the call is zeroed before each row; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 4000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_guy_set_all_pivots.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(4);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // The port: the gates and the data fetches inline (the crate carries
            // no `GraphicPieces` table, no object-table windows and no log); the
            // node loop is the crate's `pivot_steps`, the body `Sim::set_all_pivots` runs.
            let u16at = |m: &[u8], o: usize| u16::from_le_bytes([m[o], m[o + 1]]);
            let (get_type, order_type) = (a[0], a[1]);
            let (lo, hi, vx_bits) = (a[7], a[8], a[9]);
            let idx = get_type.wrapping_sub(0x32);
            let mut said: Vec<String> = Vec::new();
            let ret = if u16at(m, 0x9a) & 0x100 == 0 || idx >= rd(&c[0], 0x700) {
                0
            } else {
                let nodes = rd(&p[16], idx as usize * 0x20 + 0x14);
                if nodes == 0 {
                    0
                } else {
                    wr(m, 0x96, 0);
                    let who = m[0x9f] as i8;
                    let obj = i16::from_le_bytes([m[0x8e], m[0x8f]]);
                    let owner = m[0xa1] as i8;
                    let unit = i16::from_le_bytes([m[0x8c], m[0x8d]]);
                    // objects[who][obj] is p[1 + 4 * who + obj] (the ptrtab layout).
                    let valid =
                        obj >= 0 && who >= 0 && p[1 + 4 * who as usize + obj as usize][8] & 1 != 0;
                    if !valid && order_type != 0x17 {
                        1
                    } else {
                        let (tx, ty) = if order_type == 0x17 {
                            (rd(&p[17], 4), rd(&p[17], 8))
                        } else {
                            let w = &p[1 + 4 * who as usize + obj as usize];
                            (rd(w, 0x10) ^ 0x63637, rd(w, 0x14) ^ 0x63637)
                        };
                        let w = &p[1 + 4 * owner as usize + unit as usize];
                        let (ux, uy) = (rd(w, 0x10) ^ 0x63637, rd(w, 0x14) ^ 0x63637);
                        let facing = crate::movement::Angle(rd(m, 0x18));
                        let mut t = crate::anim::Turret {
                            angles: std::array::from_fn(|k| rd(m, 0x20 + 4 * k)),
                            des: std::array::from_fn(|k| rd(m, 0x30 + 4 * k)),
                            node_flags: 0,
                            des_flags: 0,
                        };
                        let vx = f32::from_bits(vx_bits as u32) as i32;
                        let can = crate::fight::pivot_steps(
                            nodes as usize,
                            |_| (lo, hi),
                            |_| (vx, 0),
                            (
                                crate::world::Pos::new(ux, uy),
                                crate::world::Pos::new(tx, ty),
                            ),
                            facing,
                            &mut t,
                        );
                        for k in 0..4 {
                            wr(m, 0x30 + 4 * k, t.des[k]);
                        }
                        wr(
                            m,
                            0x96,
                            i32::from(t.node_flags) | (i32::from(t.des_flags) << 16),
                        );
                        // One `Log::say` a node whose range held the bearing, the
                        // detail restored to what it was unless that was negative.
                        for _ in 0..t.des_flags.count_ones() {
                            said.push(format!(
                                "15405920,{},15418236,{lo},{hi},{vx},0,{tx},{ty},0,6",
                                rd(&c[2], 0x10).wrapping_add(0x1340c)
                            ));
                        }
                        if !said.is_empty() && rd(&c[3], 4) < 0 {
                            wr(&mut c[3], 4, 10);
                        }
                        i64::from(can)
                    }
                }
            };
            log.push(if said.is_empty() {
                "-".to_string()
            } else {
                said.join("/")
            });
            ret
        };
        assert_eq!(got, want_eax, "set_all_pivots — eax, row `{line}`");
        assert_eq!(log, want_log, "set_all_pivots — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "set_all_pivots — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "set_all_pivots — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[5 + i], "set_all_pivots — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 4000, "only {n} rows");
    eprintln!("Guy::set_all_pivots@005d8bc0: {n} rows agree");
}
