//! `Group::can_activate@00708290` under the emulator against the port (item 3044; tools/sweep_gen.py, obj with windows).

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

/// `Group::can_activate@00708290`, run by `tools/emu/sweep_group_can_activate.py` under unicorn
/// (tools/emu): 96 random bytes of object, 2 argument(s), `p` windows in order: none; the global cell 0x00c06210 points at 768 random bytes; the global cell 0x00c061ec points at 2096 random bytes; 4 random bytes at 0x00ee10ec; vtable slot +0x0 (0 args) stubbed; vtable slot +0x4 (0 args) stubbed; callee 0x006ebd30 (2 args) stubbed; callee 0x00711720 (3 args) stubbed; stub 0 answered from [0, 1] (90%); stub 1 answered from [0, 1] (90%); stub 3 answered from [0, 1] (90%); stub 3 answers 2 values in call order, printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 500 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_group_can_activate.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(3);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // `a[0]` the check-members flag, `a[1]` the player (negative: ask slot 0, answer `a[2]`,
            // then the console's `who`, `c0+0x298`); stubs 0..3 answered `a[2..6]`, stub 3 also its
            // call-order sequence `a[6..8]`. `m[0x4a]` the owner, `c1+0x820` the game's semaphore byte.
            // A zero slot-0 answer reaches the error report, skipped by the `ignore_always` byte at `c2+3`.
            let mut calls: [Vec<String>; 4] = Default::default();
            let mut who = a[1];
            if who < 0 {
                calls[0].push("m+0x0".to_string());
                assert!(a[2] != 0 || c[2][3] != 0, "the error report arm is not drawn");
                who = rd(&c[0], 0x298);
            }
            let owner = u32::from(m[0x4a]);
            let got = 'g: {
                if owner != who as u32 {
                    let ecx = 0xe3a390u32.wrapping_add(0x6eec_u32.wrapping_mul(owner));
                    calls[2].push(format!("{ecx},{who},0"));
                    break 'g 0;
                }
                if c[1][0x820] & 0x40 != 0 {
                    break 'g 0;
                }
                calls[1].push("m+0x0".to_string());
                if a[3] == 0 {
                    break 'g 0;
                }
                if a[0] != 0 {
                    calls[3].push("m+0x0,10,0,0".to_string());
                    if a[6] == 0 {
                        calls[3].push("m+0x0,9,2,0".to_string());
                        if a[7] == 0 {
                            break 'g 0;
                        }
                    }
                }
                1
            };
            log = calls
                .iter()
                .map(|c| if c.is_empty() { "-".to_string() } else { c.join("/") })
                .collect();
            i64::from(got)
        };
        assert_eq!(got, want_eax, "can_activate — eax, row `{line}`");
        assert_eq!(log, want_log, "can_activate — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "can_activate — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "can_activate — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[4 + i], "can_activate — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 500, "only {n} rows");
    eprintln!("Group::can_activate@00708290: {n} rows agree");
}
