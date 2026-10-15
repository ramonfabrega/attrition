//! `Unit::kill_garrison_order@005e2bd0` under the emulator against the port (item 2400; tools/sweep_gen.py, obj with windows).

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

/// `Unit::kill_garrison_order@005e2bd0`, run by `tools/emu/sweep_unit_kill_garrison_order.py` under unicorn
/// (tools/emu): 256 random bytes of object, 1 argument(s), the global cell 0x00c0618c points at 64 random bytes; 256 random bytes at 0x00e90000; 256 random bytes at 0x00e90200; 256 random bytes at 0x00e90400; 16 random bytes at 0x00e91000; vtable slot +0xe8 (0 args) stubbed; vtable slot +0xa8 (0 args) stubbed; callee 0x00608450 (0 args) stubbed; vtable slot +0x10 (0 args) stubbed; callee 0x005e29b0 (0 args) stubbed; callee 0x005e2cb0 (1 args) stubbed; slot +0xe8 laid as 0x0046ceb0 or its stub, a flag after the answers; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 0 answers from its own `ecx`: the word at `+0x0`; stub 1 answers from its own `ecx`: the word at `+0x0`, or `ecx+0x0` when the word at `+0x0` is set; stub 2 answers from its own `ecx`: the word at `+0x40`, or `ecx+0x40` when the word at `+0x3c` is set; stub 3 answers from its own `ecx`: the word at `+0x8`; the stubbed vtable laid at `m+0x40` as well as at `this`; the stubbed vtable laid at `c1+0` as well as at `this`; the stubbed vtable laid at `c2+0` as well as at `this`; the stubbed vtable laid at `c3+0` as well as at `this`; the stubbed vtable laid at `c1+0x40` as well as at `this`; the stubbed vtable laid at `c2+0x40` as well as at `this`; the stubbed vtable laid at `c3+0x40` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 600 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_kill_garrison_order.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(5);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // Node 0 is `this`, node k the unit window `c[k]` at
            // `UNITS + 0x200 (k - 1)`; `c[4]` is the object table
            // (`o_down` indexes it), `c[0]` the global cell.
            const UNITS: i32 = 0xe9_0000;
            fn win<'a>(m: &'a [u8], c: &'a [Vec<u8>], k: usize) -> &'a [u8] {
                if k == 0 { m } else { &c[k] }
            }
            let mut cx = (m, c, &mut log);
            let tag = |k: usize| {
                if k == 0 {
                    "m".to_string()
                } else {
                    format!("c{k}")
                }
            };
            crate::orders::kill_garrison_order_steps(
                &mut cx,
                0usize,
                16,
                |(m, c, _), k| {
                    let w = win(m, c, k);
                    // `get_action` answers the order at +0x40 when +0x3c is
                    // set; its type is the word at +0x48.
                    rd(w, 0x3c) != 0 && rd(w, 0x48) == 0x1a
                },
                |(_, _, log), k| log.push(format!("s4:{}+0x0", tag(k))),
                |(_, _, log), k| log.push(format!("s5:{}+0x0,0", tag(k))),
                |(m, c, _), k| {
                    let w = win(m, c, k);
                    let od = i32::from(rd(w, 0x90) as i16);
                    if od < 0 {
                        return None;
                    }
                    let addr = rd(&c[4], 4 * od as usize);
                    let next = ((addr - UNITS) / 0x200) as usize + 1;
                    (rd(win(m, c, next), 8) & 1 != 0).then_some(next)
                },
            );
            if log.is_empty() {
                log.push("-".into());
            }
            0
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "kill_garrison_order — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(
            m, want[0],
            "kill_garrison_order — this's bytes, row `{line}`"
        );
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "kill_garrison_order — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[6 + i],
                "kill_garrison_order — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 600, "only {n} rows");
    eprintln!("Unit::kill_garrison_order@005e2bd0: {n} rows agree");
}
