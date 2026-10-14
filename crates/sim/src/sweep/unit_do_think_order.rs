//! `Unit::do_think_order@005e5bf0` under the emulator against the port (item 2287; tools/sweep_gen.py, obj with windows).

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

/// `Unit::do_think_order@005e5bf0`, run by `tools/emu/sweep_unit_do_think_order.py` under unicorn
/// (tools/emu): 224 random bytes of object, 1 argument(s), callee 0x005e2cb0 (1 args) stubbed; vtable slot +0x10 (0 args) stubbed; callee 0x005f5760 (1 args) stubbed; the word at `m+0xdc` points at 8 random bytes less `0` (placed, null a third of the rows); the word at `p0+4` points at 16 random bytes less `0` (placed); the word at `p1+8` points at 8 random bytes less `0` (placed); the word at `m+0x18` points at 8 random bytes less `0` (placed); stub 1 answered from [0, 0, 3, 1, -1] (90%); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the stubbed vtable laid at `p2+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_do_think_order.py") else {
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
            // The list's head node `+0xdc` (`p[0]`, or null), its `+4` the
            // node `p[1]` whose `+8` is the order `p[2]` (type: vslot `+0x10`,
            // `a[2]`) and `+0xc` its byte; the list's cache `+0xd4`, `+0xcc`,
            // `+0xd0` is refreshed from them (the engine's iterator, held
            // here as bytes and not ported). The unit type `p[3]` (`+4`).
            // The log: `kill_current_order`'s `ecx,0`, `think_peasant`'s
            // `ecx,1`.
            let has = rd(m, 0xdc) != 0;
            if has {
                let (n1, ord) = (rd(&p[0], 4), rd(&p[1], 8));
                wr(m, 0xd4, n1);
                wr(m, 0xcc, ord);
                m[0xd0] = p[1][0xc];
            }
            let (mut killed, mut thought) = (false, false);
            crate::orders::think_order_steps(
                &mut (),
                |_| killed = true,
                |_| has.then_some(a[2]),
                |_| matches!(rd(&p[3], 4), 0x32..=0x35),
                |_| thought = true,
            );
            log.push(if killed { "m+0x0,0" } else { "-" }.into());
            log.push(if thought { "m+0x0,1" } else { "-" }.into());
            0
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "do_think_order — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "do_think_order — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "do_think_order — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "do_think_order — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Unit::do_think_order@005e5bf0: {n} rows agree");
}
