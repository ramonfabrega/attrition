//! `Unit::get_goody_box@005f7690` under the emulator against the port (item 2314; tools/sweep_gen.py, obj with windows).

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

/// `Unit::get_goody_box@005f7690`, run by `tools/emu/sweep_unit_get_goody_box.py` under unicorn
/// (tools/emu): 16 random bytes of object, 2 argument(s), the global cell 0x00e85f20 points at 2520 random bytes; callee 0x00713e80 (1 args) stubbed; callee 0x00714350 (4 args) stubbed; callee 0x0070f9e0 (3 args) stubbed; callee 0x0070fba0 (10 args) stubbed; stub 2 answers 0..1 every row; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_get_goody_box.py") else {
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
            // `a[0]`, `a[1]` the box's cell; `a[2..6]` the four stubs'
            // answers (`clear`, `add`, `push_group`, `action_move_to`), so
            // `a[4]` is the group's slot. `this+9` the owner byte, `this+0xa`
            // the unit's id. `GROUP` is the `Group` the function builds on
            // its own stack at `ebp-0x9d8`: the emulator's stack top
            // `0x8000_0000`, less the 0x100 the harness keeps, the two
            // arguments, the return and the saved `ebp`. `Group::add` and
            // `push_group` are entered with that `ecx` because the stub
            // before each leaves it standing. `c[0]` the groups table
            // (`0xe85f20`), slot stride `0x9d4`. `QUEUE_FIRST` is 0 and
            // `EXPLORE_TO` 3 (the listing's pushes at `005f76f7..0x5f7722`).
            const GROUP: i64 = 0x8000_0000 - 0x100 - 8 - 4 - 4 - 0x9d8;
            let o = crate::goody::goody_box_order(crate::world::Cell::new(a[0], a[1]));
            let who = m[9];
            let id = i16::from_le_bytes([m[0xa], m[0xb]]);
            let q = match o.queue {
                crate::orders::QueuePos::First => 0,
                crate::orders::QueuePos::Last => 1,
                crate::orders::QueuePos::New => 2,
            };
            let kind = match o.kind {
                crate::orders::MoveKind::MoveTo => 1,
                crate::orders::MoveKind::AttackTo => 2,
                crate::orders::MoveKind::ExploreTo => 3,
                crate::orders::MoveKind::FleeTo => 4,
            };
            log.push(format!("{GROUP},{id},{who},0,0"));
            log.push(format!("{GROUP},{who},{GROUP},{}", o.force as i32));
            log.push(format!(
                "c0+{:#x},{},{},{q},{},{},{kind},{},-1,-1,0",
                a[4] * 0x9d4,
                o.to.x,
                o.to.y,
                o.set_angle as i32,
                o.angle.0,
                o.action as i32,
            ));
            0
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "get_goody_box — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "get_goody_box — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "get_goody_box — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[2 + i],
                "get_goody_box — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Unit::get_goody_box@005f7690: {n} rows agree");
}
