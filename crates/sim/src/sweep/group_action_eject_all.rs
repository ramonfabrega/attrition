//! `Group::action_eject_all@00710b40` under the emulator against the port (item 3261; tools/sweep_gen.py, obj with windows).

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

/// `Group::action_eject_all@00710b40`, run by `tools/emu/sweep_group_action_eject_all.py` under unicorn
/// (tools/emu): 2304 random bytes of object, 4 argument(s), `p` windows in order: p0..p11 the place(s), in flag order; the global cell 0x00c0618c points at 1024 random bytes; 4 random bytes at 0x00cc02f8 (each word in 0..1); 4 random bytes at 0x00ed6574 (each word in 0..6); 4 random bytes at 0x00ed6590 (each word in 0..6); 4 random bytes at 0x00ed65ac (each word in 0..6); 4 random bytes at 0x00ed65c8 (each word in 0..6); 4 random bytes at 0x00ed6638 (each word in 0..6); vtable slot +0x10 (4 args) stubbed; vtable slot +0x14 (0 args) stubbed; vtable slot +0x60 (2 args) stubbed; vtable slot +0x158 (3 args) stubbed; vtable slot +0xb8 (2 args) stubbed; callee 0x00646d50 (1 args) stubbed; callee 0x00646c40 (1 args) stubbed; callee 0x0064cd20 (4 args) stubbed; callee 0x00647080 (3 args) stubbed; the word at `c0+0x14+0x1c*m[0x4a]` points at 12 random bytes less `0` (placed); the word at `p0+0` points at 96 random bytes less `0` (placed); the word at `p0+4` points at 96 random bytes less `0` (placed); the word at `p0+8` points at 96 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*m[0x4a]` points at 12 random bytes less `0` (placed); the word at `p4+0` points at 32 random bytes less `0` (placed); the word at `p4+4` points at 32 random bytes less `0` (placed); the word at `p4+8` points at 32 random bytes less `0` (placed); the word at `0xed6580+0x1c*m[0x4a]` points at 32 random bytes less `0` (placed); the word at `p1+0x18` points at 32 random bytes less `0` (placed); the word at `p2+0x18` points at 32 random bytes less `0` (placed); the word at `p3+0x18` points at 32 random bytes less `0` (placed); slot +0xb8 laid as 0x00653790 or its stub, a flag after the answers; slot +0x158 laid as 0x0060eda0 or its stub, a flag after the answers; stub 5 answered from [-1, 0, 1, 1, 2] (90%); stub 6 answered from [0, 0, 0, 1, 2] (90%); stub 5 answers 6 values in call order, printed after the answers; stub 6 answers 3 values in call order, printed after the answers; stub 4 answers 6 values in call order, printed after the answers; stub 2 answers 6 values in call order, printed after the answers; stub 4 answers 0..1 every row; stub 2 answers 0..1 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p10+0` as well as at `this`; the stubbed vtable laid at `p11+0` as well as at `this`; the stubbed vtable laid at `p1+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1500 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_group_action_eject_all.py") else {
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
                // Row layout: a[0..4] the arguments (back_to_work, who, eject_o, eject_who);
                // a[4..13] the nine stubs' answers (unused here: every stub that matters
                // answers from a call-order list); then the call-order answers of
                // num_inside (6), can_carry (3), the `0x140` slot stub (6) and the inner
                // slot (6); a[34], a[35] the two "compared with the base" flags.
                use std::cell::{Cell, RefCell};
                let (back, who, eject_who) = (a[0], a[1], a[3]);
                let player = i32::from(m[0x4a]);
                let lg = RefCell::new(Vec::<String>::new());
                let rec = |name: &str, args: &[i32]| {
                    let args = args
                        .iter()
                        .map(|v| v.to_string())
                        .collect::<Vec<_>>()
                        .join(",");
                    lg.borrow_mut().push(format!("{name},{args}"));
                };
                // The scenario's ignore-orders filter: the per-player count and the
                // objects it names; each non-negative one goes to the group's own
                // slot 0x10 with the player.
                if rd(&c[1], 0) != 0 && (0..8).contains(&player) {
                    let slot = match player {
                        0 => Some(2),
                        1 => Some(3),
                        2 => Some(4),
                        3 => Some(5),
                        7 => Some(6),
                        _ => None,
                    };
                    let n = slot.map_or(0, |s| rd(&c[s], 0));
                    for i in 0..n.max(0) as usize {
                        let o = rd(&p[8], 4 * i);
                        if o >= 0 {
                            rec("s0:m+0x0", &[o, player, 0, 0]);
                        }
                    }
                }
                lg.borrow_mut().push("s1:m+0x0".to_string());
                if crate::group::eject_all_own_arm(player, eject_who, who) {
                    wr(m, 0x10, -1);
                    let count = rd(m, 0xc);
                    let (k5, k6, k4, k2) = (
                        Cell::new(0usize),
                        Cell::new(0usize),
                        Cell::new(0usize),
                        Cell::new(0usize),
                    );
                    let nxt = |k: &Cell<usize>, base: usize, n: usize| -> i32 {
                        let v = a[base + k.get().min(n - 1)];
                        k.set(k.get() + 1);
                        v
                    };
                    for i in (0..count.max(0) as usize).rev() {
                        let idx =
                            i16::from_le_bytes([m[0x8cc + 2 * i], m[0x8cc + 2 * i + 1]]) as usize;
                        let (o, inner, unit) = (1 + idx, 9 + idx, 5 + idx);
                        let alive = rd(&p[o], 8) & 1 != 0;
                        let ejects = crate::group::eject_all_wants(
                            alive,
                            || {
                                rec(&format!("s5:p{o}+0x0"), &[1]);
                                nxt(&k5, 13, 6) > 0
                            },
                            || {
                                rec(&format!("s6:p{o}+0x0"), &[2]);
                                nxt(&k6, 19, 3) != 0
                            },
                        );
                        if !ejects {
                            continue;
                        }
                        let (arg1, arg3) = if back == 0 { (-1, 1) } else { (0x32, 0) };
                        rec(&format!("s7:p{o}+0x0"), &[0, arg1, 0, arg3]);
                        let is = |ty: i32| -> bool {
                            if a[34] != 0 {
                                rec(&format!("s2:p{inner}+0x0"), &[ty, 0]);
                                nxt(&k2, 28, 6) != 0
                            } else {
                                rec(&format!("s4:p{o}+0x0"), &[ty, 0]);
                                nxt(&k4, 22, 6) != 0
                            }
                        };
                        if !is(0x140) && !is(0x13e) {
                            continue;
                        }
                        rec(&format!("s5:p{unit}+0x0"), &[0]);
                        if nxt(&k5, 13, 6) == 0 {
                            let name = if a[35] != 0 { "s8" } else { "s3" };
                            rec(&format!("{name}:p{unit}+0x0"), &[0, -1, 0]);
                        }
                    }
                }
                log.extend(lg.into_inner());
                0
            }
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "action_eject_all — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "action_eject_all — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "action_eject_all — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[8 + i],
                "action_eject_all — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 1500, "only {n} rows");
    eprintln!("Group::action_eject_all@00710b40: {n} rows agree");
}
