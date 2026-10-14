//! `Unit::add_move_facing_order@005e55c0` under the emulator against the port (item 2341; tools/sweep_gen.py, obj with windows).

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

/// `Unit::add_move_facing_order@005e55c0`, run by `tools/emu/sweep_unit_add_move_facing_order.py` under unicorn
/// (tools/emu): 256 random bytes of object, 11 argument(s), the global cell 0x00c0618c points at 128 random bytes; callee 0x00616e80 (0 args) stubbed; callee 0x0060b030 (0 args) stubbed; vtable slot +0xcc (0 args) stubbed; vtable slot +0x148 (1 args) stubbed; callee 0x005e37f0 (1 args) stubbed; callee 0x005e3920 (0 args) stubbed; callee 0x0060a870 (0 args) stubbed; callee 0x00730ac0 (1 args) stubbed; vtable slot +0x40 (0 args) stubbed; vtable slot +0x44 (0 args) stubbed; vtable slot +0x48 (0 args) stubbed; vtable slot +0x4c (0 args) stubbed; callee 0x0046d5a0 (1 args) stubbed; callee 0x005e5410 (5 args) stubbed; stub 2's answer points at 16 random bytes less `0` (placed); the word at `c0+0x14+max(0,i32(p[0],0xc))*0x1c` points at 4 random bytes less `i32(p[0],8)*4` (placed); the word at `p1+0` points at 4 random bytes less `0x18` (placed); the word at `p2+0` points at 4 random bytes less `0x218` (placed); the word at `m+0x18` points at 4 random bytes less `0x2c8` (placed); the word at `m+0xdc` points at 16 random bytes less `0` (placed); stub 8's answer points at 100 random bytes less `0` (placed); the word at `p6+0` points at 8 random bytes less `0` (placed); stub 9's answer points at 100 random bytes less `0` (placed); the word at `p8+0` points at 8 random bytes less `0` (placed); stub 10's answer points at 100 random bytes less `0` (placed); the word at `p10+0` points at 8 random bytes less `0` (placed); stub 11's answer points at 100 random bytes less `0` (placed); the word at `p12+0` points at 8 random bytes less `0` (placed); stub 0 answered from [10, 10, 1] (90%); stub 3 answered from [0, 1] (90%); stub 1 answers the address of 16 random bytes, a `p` window after the places; stub 7 answers the address of 16 random bytes, a `p` window after the places; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); the stubbed vtable laid at `p14+0` as well as at `this`; the stubbed vtable laid at `p15+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_add_move_facing_order.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(1);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // The arguments `a[0..11]`, the stubs' answers after them
            // (`a[11]` `order_type`, `a[14]` `has_objmask`). The kept
            // attack's record is `p[0]` (`+8` who, `+0xc` o), the objects
            // chain `c[0]` → `p[1]` → `p[2]` → `p[3]` (the target type's
            // `+0x218`), the role word `p[4]`, the list head node `p[5]`;
            // the order a mode-`k` vslot makes is `p[4 + 2k]` (its
            // vbtable `p[5 + 2k]` puts the `UnitOrder` base at `+0x5c`, the
            // flags byte at `+0x60`). The made order is the port's own
            // (`new_move_order`, `fill_move_order`), laid into its window
            // by the `MoveOrder` record's offsets.
            use crate::orders::{
                MoveBits, QueuePos, add_move_facing_steps, fill_move_order, new_move_order,
            };
            use crate::{Pos, movement::Angle};
            let pos = match a[5] {
                0 => QueuePos::First,
                1 => QueuePos::Last,
                _ => QueuePos::New,
            };
            let facing = match a[7] {
                -1 => None,
                v => Some(v != 0),
            };
            let orig = (a[8] != -1 || a[9] != -1).then(|| Pos::new(a[8], a[9]));
            let bits = MoveBits {
                pathed: a[4] != 0,
                action: a[6] != 0,
                disembark: a[10] != 0,
            };
            let mut cx = (m, p, &mut log);
            add_move_facing_steps(
                &mut cx,
                (a[0], a[1]),
                a[3],
                pos,
                bits,
                |(_, p, log)| {
                    log.push("s0:m+0x0".into());
                    if a[11] != 10 {
                        return None;
                    }
                    for _ in 0..2 {
                        log.push("s1:m+0x0".into());
                        log.push("s2:p14+0x0".into());
                    }
                    let (who, o) = (rd(&p[0], 8), rd(&p[0], 0xc));
                    (who >= 0 && o >= 0).then_some((who, o))
                },
                |(_, p, _), _| rd(&p[3], 0) == 2,
                |(_, _, log)| {
                    log.push("s3:m+0x0,-2147483648".into());
                    a[14] != 0
                },
                |(_, p, _)| p[4][0] & 0x10 != 0,
                |(m, _, _)| {
                    let v = rd(m, 0x68) | 0x0400_0000;
                    wr(m, 0x68, v);
                },
                |(m, _, _)| {
                    let v = rd(m, 0x68) & !0x0400_0000;
                    wr(m, 0x68, v);
                    wr(m, 0xc0, 0);
                },
                |(_, _, log)| log.push("s4:m+0x0,0".into()),
                |(_, _, log)| log.push("s5:m+0x0".into()),
                |(_, _, log)| log.push("s6:m+0x0".into()),
                |(_, p, log), idx| {
                    log.push(format!("s7:m+0x0,{idx}"));
                    log.push(format!("s{}:p15+0x0", 7 + idx));
                    let w = 4 + 2 * idx as usize;
                    let mut o = new_move_order(idx, Angle(a[2]), facing, orig, None);
                    o.flags = p[w][0x60];
                    (o, w)
                },
                |(o, _), f| fill_move_order(o, f),
                |(_, p, log), (o, w)| {
                    let crate::orders::Body::Move(mo) = o.body else {
                        unreachable!("a move")
                    };
                    let q = &mut p[w];
                    wr(q, 0x4, mo.dest.x);
                    wr(q, 0x8, mo.dest.y);
                    wr(q, 0xc, mo.angle.0);
                    wr(q, 0x10, i32::from(mo.has_waypoint));
                    wr(q, 0x18, mo.pause);
                    wr(q, 0x1c, mo.retry);
                    wr(q, 0x24, mo.timer);
                    wr(q, 0x28, mo.facing.map_or(-1, i32::from));
                    wr(q, 0x2c, mo.waypoint.x);
                    wr(q, 0x30, mo.waypoint.y);
                    let last = mo.last.unwrap_or(Pos::new(-1, -1));
                    wr(q, 0x34, last.x);
                    wr(q, 0x38, last.y);
                    let orig = mo.orig.unwrap_or(Pos::new(-1, -1));
                    wr(q, 0x44, orig.x);
                    wr(q, 0x48, orig.y);
                    // `+0x4c`/`+0x4e`: the destination's offset in its
                    // 0x300 tile, as `crate::path` derives it.
                    q[0x4c..0x4e].copy_from_slice(&((mo.dest.x % 0x300) as i16).to_le_bytes());
                    q[0x4e..0x50].copy_from_slice(&((mo.dest.y % 0x300) as i16).to_le_bytes());
                    q[0x60] = o.flags;
                    log.push(format!("s12:m+0xcc,p{w}+0x5c"));
                },
                |(m, p, _)| {
                    let next = rd(&p[5], 0);
                    wr(m, 0xdc, next);
                },
                |(_, _, log), (who, o)| log.push(format!("s13:m+0x0,{who},{o},0,0,0")),
            );
            0
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "add_move_facing_order — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(
            m, want[0],
            "add_move_facing_order — this's bytes, row `{line}`"
        );
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "add_move_facing_order — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[2 + i],
                "add_move_facing_order — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 1000, "only {n} rows");
    eprintln!("Unit::add_move_facing_order@005e55c0: {n} rows agree");
}
