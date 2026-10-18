//! `Unit::find_new_air_target@005ebc70` under the emulator against the port (item 3250; tools/sweep_gen.py, obj with windows).

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

/// `Unit::find_new_air_target@005ebc70`, run by `tools/emu/sweep_unit_find_new_air_target.py` under unicorn
/// (tools/emu): 128 random bytes of object, 5 argument(s), `p` windows in order: p0 argument 4, p1..p20 the place(s), in flag order; the global cell 0x00c0618c points at 576 random bytes; the global cell 0x00c061f0 points at 64 random bytes; argument 4 names 4 random bytes (null a third of the rows); callee 0x0065a620 (11 args) stubbed; callee 0x00648ba0 (3 args) stubbed; callee 0x0064e5c0 (4 args) stubbed; the word at `c0+0x214` points at 16 random bytes less `0` (placed); the word at `p1+0` points at 16 random bytes less `0` (placed); the word at `p1+4` points at 16 random bytes less `0` (placed); the word at `p1+8` points at 16 random bytes less `0` (placed); the word at `p1+12` points at 16 random bytes less `0` (placed); the word at `c0+0x14` points at 16 random bytes less `0` (placed); the word at `p6+0` points at 32 random bytes less `0` (placed); the word at `p6+4` points at 32 random bytes less `0` (placed); the word at `p6+8` points at 32 random bytes less `0` (placed); the word at `p6+12` points at 32 random bytes less `0` (placed); the word at `c0+0x30` points at 16 random bytes less `0` (placed); the word at `p11+0` points at 32 random bytes less `0` (placed); the word at `p11+4` points at 32 random bytes less `0` (placed); the word at `p11+8` points at 32 random bytes less `0` (placed); the word at `p11+12` points at 32 random bytes less `0` (placed); the word at `c0+0x4c` points at 16 random bytes less `0` (placed); the word at `p16+0` points at 32 random bytes less `0` (placed); the word at `p16+4` points at 32 random bytes less `0` (placed); the word at `p16+8` points at 32 random bytes less `0` (placed); the word at `p16+12` points at 32 random bytes less `0` (placed); stub 1 answered from [0, 1, 1] (90%); stub 1 answers 8 values in call order, printed after the answers; stub 2 answers 8 values in call order, printed after the answers; stub 2 answers -4..30 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 3000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_find_new_air_target.py") else {
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
            {
                // Windows: c0 `GameAccess::objects` (count `+0x208`, the hit list `+0x214` -> p1 -> p2..p5, each a
                // record with the type byte `+9` and the signed index short `+10`; the type's position table
                // `+0x14 + 28 * type` -> p6 / p11 / p16, whose records p7..p10 / p12..p15 / p17..p20 hold the
                // xor-0x63637 position at `+0x10` / `+0x14`), c1 `constants` (`+0x2c` `aircraft_respond_range`),
                // p0 `param_5`. Stubs: 0 `find_units` (11 args, the hit list is the window, the same for
                // every call), 1 `valid_target` (`a[8..16]` in call order), 2 `compare_target` (`a[16..24]`).
                // Drives `crate::air::pick_air_target`, the selection `Sim::find_new_air_target` runs, with
                // the stubs' answers handed in; `param_3 >= 0` (the arm between the loops) is drawn off
                // (`--argrange 2:-1:-1`): the crate keeps it a SEAM. The `unit_masks & 0x40000` swap of the
                // point is the one line the port keeps inline (the crate reads `Sim::ai_driven(owner)`).
                use std::cell::{Cell, RefCell};
                const K: i32 = 0x63637;
                let range = rd(&c[1], 0x2c);
                let (ux, uy) = (rd(m, 0x10) ^ K, rd(m, 0x14) ^ K);
                let (mut x, mut y) = (a[0], a[1]);
                if rd(m, 0x68) & 0x40000 != 0 {
                    (x, y) = (ux, uy);
                }
                assert!(a[2] < 0, "the second arm is not drawn");
                let who = m[9];
                let n = rd(&c[0], 0x208).max(0) as usize;
                let cands: Vec<(i32, usize)> = (0..n)
                    .map(|i| {
                        let cand = &p[2 + i];
                        (
                            i16::from_le_bytes([cand[10], cand[11]]) as i32,
                            cand[9] as usize,
                        )
                    })
                    .collect();
                let pos_of = |&(id, ty): &(i32, usize)| {
                    let r = &p[7 + 5 * ty + id as usize];
                    crate::Pos::new(rd(r, 0x10) ^ K, rd(r, 0x14) ^ K)
                };
                let sep = |t: &mut String, s: String| {
                    if t != "-" {
                        t.push('/');
                        t.push_str(&s);
                    } else {
                        *t = s;
                    }
                };
                let toks = RefCell::new(["-".to_string(), "-".into(), "-".into()]);
                let (vn, cn) = (Cell::new(0usize), Cell::new(0usize));
                let r3c0 = range * 0x3c0;
                let list = || cands.iter().map(|cd| (*cd, pos_of(cd))).collect::<Vec<_>>();
                let found = crate::air::pick_air_target(
                    range,
                    crate::Pos::new(ux, uy),
                    crate::Pos::new(x, y),
                    || {
                        let s = format!("{uy},{ux},{uy},3,{who},{},1,13,2,0,{uy},1", range * 0xc0);
                        sep(&mut toks.borrow_mut()[0], s);
                        list()
                    },
                    || {
                        let s = format!("{r3c0},{x},{y},3,{who},{},1,0,0,0,{r3c0},1", range * 0xc0);
                        sep(&mut toks.borrow_mut()[0], s);
                        list()
                    },
                    |(id, ty)| {
                        sep(&mut toks.borrow_mut()[1], format!("m+0x0,{id},{ty},0"));
                        vn.set(vn.get() + 1);
                        a[7 + vn.get()] != 0
                    },
                    |(id, ty)| {
                        sep(&mut toks.borrow_mut()[2], format!("m+0x0,{id},{ty},1,0"));
                        cn.set(cn.get() + 1);
                        a[15 + cn.get()]
                    },
                );
                log.extend(toks.into_inner());
                match found {
                    Some((id, ty)) => {
                        if a[4] != 0 {
                            wr(&mut p[0], 0, ty as i32);
                        }
                        i64::from(id)
                    }
                    None => -1,
                }
            }
        };
        assert_eq!(got, want_eax, "find_new_air_target — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "find_new_air_target — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(
            m, want[0],
            "find_new_air_target — this's bytes, row `{line}`"
        );
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "find_new_air_target — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[3 + i],
                "find_new_air_target — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 3000, "only {n} rows");
    eprintln!("Unit::find_new_air_target@005ebc70: {n} rows agree");
}
