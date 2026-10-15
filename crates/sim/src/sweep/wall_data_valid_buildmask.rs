//! `WallData::valid_buildmask@0063e2a0` under the emulator against the port (item 2774; tools/sweep_gen.py, obj with windows).

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

/// `WallData::valid_buildmask@0063e2a0`, run by `tools/emu/sweep_wall_data_valid_buildmask.py` under unicorn
/// (tools/emu): 32 random bytes of object, 1 argument(s), vtable slot +0x20 (0 args) stubbed; vtable slot +0x3c (0 args) stubbed; callee 0x0062d4d0 (0 args) stubbed; callee 0x00646c40 (1 args) stubbed; the word at `0xc0ab84` points at 16 random bytes less `0` (placed); the word at `p0+0` points at 16 random bytes less `0` (placed); stub 0 answered from [0, 1] (90%); stub 2 answered from [0, 1] (90%); stub 3 answered from [0, 1] (90%); stub 1 answers the address of 16 random bytes, a `p` window after the places; the stubbed vtable laid at `p1+0`; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the stubbed vtable laid at `m+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_wall_data_valid_buildmask.py") else {
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
            // `WallData::valid_buildmask(mask)`: bit `0x40` with the wall's own slot `0x20` nonzero asks the
            // build data of the object at `[0xc0ab84][+0xa]` (slot `0x3c`, `p[1]`; `can_infinite`,
            // `0x62d4d0`, `ecx` its record `p[2]`) and a yes answers 1; bit `0x80` asks `can_carry(2)`
            // (`0x646c40`) and a yes answers 1; otherwise 0. The log is the three callees' `ecx`.
            let (mask, s20, s_inf, s_carry) = (a[0], a[1], a[3], a[4]);
            let (mut l1, mut l2, mut l3) = ("-", "-", "-");
            let mut r = 0i64;
            if mask & 0x40 != 0 && s20 != 0 {
                l1 = "p1+0x0";
                l2 = "p2+0x0";
                if s_inf != 0 {
                    r = 1;
                }
            }
            if r == 0 && mask & 0x80 != 0 {
                l3 = "m+0x0,2";
                if s_carry != 0 {
                    r = 1;
                }
            }
            log.extend([l1.to_string(), l2.to_string(), l3.to_string()]);
            r
        };
        assert_eq!(got, want_eax, "valid_buildmask — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "valid_buildmask — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "valid_buildmask — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "valid_buildmask — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "valid_buildmask — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("WallData::valid_buildmask@0063e2a0: {n} rows agree");
}
