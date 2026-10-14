//! `Unit::add_think_order@005e3df0` under the emulator against the port (item 2314; tools/sweep_gen.py, obj with windows).

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

/// `Unit::add_think_order@005e3df0`, run by `tools/emu/sweep_unit_add_think_order.py` under unicorn
/// (tools/emu): 256 random bytes of object, 1 argument(s), callee 0x00730ac0 (1 args) stubbed; vtable slot +0xb0 (0 args) stubbed; callee 0x0046d5a0 (1 args) stubbed; callee 0x005e3920 (0 args) stubbed; callee 0x0060a870 (0 args) stubbed; the word at `m+0xdc` points at 16 random bytes less `0` (placed); stub 0 answers the address of 192 random bytes, a `p` window after the places; stub 1 answers the address of 64 random bytes, a `p` window after the places; the stubbed vtable laid at `p1+0`; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_add_think_order.py") else {
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
            // The made order is `p[2]` (stub 1's answer, through `p[1]`'s
            // vtable, `p[1]` being stub 0's), carried as its flags byte
            // `+4`; the link, `clear_partial_path` and `update_action`
            // are recorded; the list's head node `+0xdc` is `p[0]`, and
            // the rotation `head = head->next` (`+0`) is the engine's
            // iterator, held here as bytes (the port's is `new_head`).
            let mut cx = (m, p, &mut log);
            crate::orders::add_think_steps(
                &mut cx,
                |(_, p, log)| {
                    log.push(format!("m+0x0,{}", crate::orders::index::THINK));
                    log.push("p1+0x0".into());
                    p[2][4]
                },
                |f| f,
                |(_, p, log), f| {
                    p[2][4] = f;
                    log.push("m+0xcc,p2+0x0".into());
                },
                |(_, _, log)| log.push("m+0x0".into()),
                |(m, p, _)| {
                    let next = rd(&p[0], 0);
                    wr(m, 0xdc, next);
                },
                |(_, _, log)| log.push("m+0x0".into()),
            );
            0
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "add_think_order — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "add_think_order — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "add_think_order — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "add_think_order — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Unit::add_think_order@005e3df0: {n} rows agree");
}
