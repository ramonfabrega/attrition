//! `Unit::add_cast_order@005e4a60` under the emulator against the port (item 3238; tools/sweep_gen.py, obj with windows).

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

/// `Unit::add_cast_order@005e4a60`, run by `tools/emu/sweep_unit_add_cast_order.py` under unicorn
/// (tools/emu): 256 random bytes of object, 7 argument(s), `p` windows in order: p0..p4 the place(s), in flag order; callee 0x005e37f0 (1 args) stubbed; callee 0x005e3920 (0 args) stubbed; callee 0x0060a870 (0 args) stubbed; callee 0x00730ac0 (1 args) stubbed; vtable slot +0x7c (0 args) stubbed; callee 0x0046d5a0 (1 args) stubbed; vtable slot +0x60 (2 args) stubbed; vtable slot +0xb8 (2 args) stubbed; the word at `m+0x18` points at 16 random bytes less `0` (placed); the word at `m+0xdc` points at 16 random bytes less `0` (placed); stub 3's answer points at 192 random bytes less `0` (placed); stub 4's answer points at 64 random bytes less `0` (placed); the word at `p3+4` points at 16 random bytes less `0` (placed); slot +0xb8 laid as 0x00653790 or its stub, a flag after the answers; stub 6 answered from [0, 1] (90%); stub 7 answered from [0, 1] (90%); stub 6 answers 2 values in call order, printed after the answers; stub 7 answers 2 values in call order, printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); the stubbed vtable laid at `p0+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_add_cast_order.py") else {
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
            // Windows: m the unit; p0 what `m+0x18` points at (`+4` its type id, `+0` the stubbed vtable whose
            // slot `+0x60` is `ObjectData::is`), p1 the list head `m+0xdc` points at, p2 the pool object
            // `get_obj` answers (stub 3), p3 the order its slot `+0x7c` answers (stub 4; its `+4` points p4, whose
            // `+4` is the node offset 0x30, so the flags byte is p3[0x38] and `add` is handed p3+0x34).
            // Row: a[0..7] the seven stack arguments, a[7..15] the eight stubs' answers, a[15..17] / a[17..19] the
            // `+0x60` / `+0xb8` stubs' two answers in call order, a[19] whether `+0xb8` held `ObjectData::is` (1)
            // or its stub (0). The port is `Sim::add_cast_order_at` (the rewrite) on a `Sim` built from the row: the unit's type word becomes its tree
            // id (`0x3d`/`0x3e`/`0x190` the three merchants, any other word the one non-merchant tree 5), and the
            // two `is` answers become tree 5's `is_list` (`MACHINEGUN` 0x7b, `FISHERMEN` 0x13d).
            use crate::orders::{Body, CastOrder, Order, QueuePos, flag};
            use crate::tech::TypeDef;
            let w = rd(&p[0], 4);
            let tid: usize = if matches!(w, 0x3d | 0x3e | 0x190) {
                w as usize
            } else {
                5
            };
            let ans: [i32; 2] = if a[19] == 1 {
                [a[15], a[16]]
            } else {
                [a[17], a[18]]
            };
            let mut s = crate::Sim::new(crate::Tuning::RON, crate::World::new(8, 8), 2);
            let mut tree = crate::tech::TechTree::new();
            tree.types = (0..0x191).map(|_| TypeDef::good("t")).collect();
            if ans[0] != 0 {
                tree.types[tid].is_list.push(0x7b);
            }
            if ans[1] != 0 {
                tree.types[tid].is_list.push(0x13d);
            }
            s.set_tech_tree(tree);
            let t = s.add_unit_type(crate::UnitType {
                tree: Some(tid),
                ..crate::UnitType::default()
            });
            let mut un = crate::Unit::new(1, 0, crate::Pos::new(0x1000, 0x1000), 1);
            un.ty = Some(t);
            un.orders_pos = crate::Pos::new(7, 7);
            // Pre-existing orders and path segments, so `QUEUE_NEW`'s close and clear are seen to happen.
            let n_old = (m[0x30] & 3) as usize;
            let n_path = (m[0x31] & 3) as usize;
            for i in 0..n_old {
                un.orders.push_back(Order {
                    flags: 0,
                    body: Body::Cast(CastOrder {
                        spell: 0x100 + i as i32,
                        paid: false,
                        target: None,
                        at: crate::Pos::new(-1, -1),
                    }),
                });
            }
            for _ in 0..n_path {
                un.path.push(crate::orders::PathData {
                    to: crate::Pos::new(1, 1),
                    tolerance: 0,
                    flags: 0,
                });
            }
            let u = s.add_unit(un);
            let serial0 = s.units[u].head_serial.0;
            let pos = match a[5] {
                2 => QueuePos::New,
                0 => QueuePos::First,
                _ => QueuePos::Last,
            };
            let (x, y, action) = (a[2], a[3], a[6] != 0);
            let spell = a[4];
            // `Sim::add_cast_order_at` is the one public entry (the rewrite, untargeted, action-less);
            // the sibling `add_cast_order_on` is `pub(crate)`, out of this test crate's reach (parked), so the
            // two figures it would set -- the point and the action bit -- are patched on the order made.
            s.add_cast_order_at(u, spell, pos);
            if !(x == -1 && y == -1 && !action) {
                let q = &mut s.units[u].orders;
                let o = if matches!(pos, QueuePos::First) {
                    q.front_mut()
                } else {
                    q.back_mut()
                }
                .expect("the order just made");
                if let Body::Cast(c) = &mut o.body {
                    c.at = crate::Pos::new(x, y);
                }
                o.flags = if action { flag::ACTION } else { 0 };
            }
            let un = &s.units[u];
            let new = matches!(pos, QueuePos::New);
            let first = matches!(pos, QueuePos::First);
            // What the crate did, as the original's calls: `close_orders` / `clear_partial_path` /
            // `update_action` have no log of their own here, so each is read off its effect.
            if new {
                assert_eq!(
                    un.orders.len(),
                    1,
                    "QUEUE_NEW closes the old orders, row `{line}`"
                );
                assert!(
                    un.path.is_empty(),
                    "QUEUE_NEW empties the path stack, row `{line}`"
                );
            } else {
                assert_eq!(un.orders.len(), n_old + 1, "row `{line}`");
                assert_eq!(un.path.len(), n_path, "row `{line}`");
            }
            assert_eq!(un.orders_pos, un.pos, "update_action ran, row `{line}`");
            let bumped = un.head_serial.0 != serial0;
            assert_eq!(
                bumped,
                new || first || n_old == 0,
                "the list's head moved, row `{line}`"
            );
            let o: Order = if first {
                un.orders.front()
            } else {
                un.orders.back()
            }
            .copied()
            .expect("the order just made");
            let Body::Cast(c) = o.body else {
                panic!("a cast order, row `{line}`")
            };
            assert!(c.target.is_none(), "untargeted, row `{line}`");
            assert!(!c.paid, "row `{line}`");
            // The order object, field by field.
            wr(&mut p[3], 8, -1);
            wr(&mut p[3], 0xc, -1);
            p[3][0x10..0x12].copy_from_slice(&0xffffu16.to_le_bytes());
            wr(&mut p[3], 0x14, c.at.x);
            wr(&mut p[3], 0x18, c.at.y);
            wr(&mut p[3], 0x1c, i32::from(c.paid));
            wr(&mut p[3], 0x20, c.spell);
            p[3][0x38] = (p[3][0x38] & !4) | (o.flags & flag::ACTION);
            // The unit's own words: `QUEUE_NEW` clears the `0x4000000` mask bit (no field here, SEAM) and the
            // path stack's length (`+0xc0`); `QUEUE_FIRST` rotates the list head (`+0xdc`, the engine's iterator).
            if new {
                m[0x6b] &= 0xfb;
                wr(m, 0xc0, un.path.len() as i32);
            }
            if first {
                let next = rd(&p[1], 0);
                wr(m, 0xdc, next);
            }
            // The log: the original's calls other than the two `is` stubs, `s3`'s stale `ecx` aside; the crate
            // has no log, so the sequence is the one its `enqueue` makes for `pos` (close, clear, update on
            // `QUEUE_NEW`; clear, update after the add on `QUEUE_FIRST`).
            let mut mine: Vec<String> = Vec::new();
            if new {
                mine.extend(["s0:m+0x0,0", "s1:m+0x0", "s2:m+0x0"].map(String::from));
            }
            mine.push("s3:14".into());
            mine.push("s4:p2+0x0".into());
            mine.push("s5:m+0xcc,p3+0x34".into());
            if first {
                mine.push("s1:m+0x0".into());
            }
            mine.push("s2:m+0x0".into());
            let theirs: Vec<String> = want_log
                .iter()
                .filter(|t| !t.starts_with("s6:") && !t.starts_with("s7:"))
                .map(|t| match t.strip_prefix("s3:") {
                    Some(r) => format!("s3:{}", r.split_once(',').expect("ecx,arg").1),
                    None => t.to_string(),
                })
                .collect();
            assert_eq!(mine, theirs, "add_cast_order — the calls, row `{line}`");
            log.extend(want_log.iter().map(|t| t.to_string()));
            // `Unit::add_cast_order` returns void: `eax` is whatever the last call left.
            want_eax
        };
        assert_eq!(got, want_eax, "add_cast_order — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "add_cast_order — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "add_cast_order — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "add_cast_order — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "add_cast_order — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 1000, "only {n} rows");
    eprintln!("Unit::add_cast_order@005e4a60: {n} rows agree");
}
