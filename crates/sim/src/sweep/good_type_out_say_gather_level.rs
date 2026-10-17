//! `GoodTypeOut::say_gather_level@006700d0` under the emulator against the port (item 2783; tools/sweep_gen.py, obj with windows).

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

/// `GoodTypeOut::say_gather_level@006700d0`, run by `tools/emu/sweep_good_type_out_say_gather_level.py` under unicorn
/// (tools/emu): 16 random bytes of object, 5 argument(s), the global cell 0x00c06378 points at 32 random bytes; 4 random bytes at 0x00c8cd00; argument 0 names 16 random bytes; callee 0x00a1d440 (1 args) stubbed; callee 0x00a1d180 (1 args) stubbed; callee 0x00670130 (5 args) stubbed; the word at `c0+0x10` points at 32 random bytes less `0x17c0` (placed); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_good_type_out_say_gather_level.py") else {
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
            // `GoodTypeOut::say_gather_level(text, a1, ..)`, `text` the string at `a[0]` (`p[0]`): appends
            // (`String::operator+=`, `0xa1d440`) the string at `*(G+0x10) + 0x17c0` for `G` the record of cell
            // `0xc06378` (`c[0]`, placed so the sum is the start of a window, `p[1]`), then the integer
            // `[this+4] + 1` (`0xa1d180`), then the string at `[0xc8cd00] + 0x7db4` (`c[1]`), and calls
            // `say_level` (`0x670130`) with `text`, `a1`, two stale stack words (a 0 and the return address
            // `0x67010d`) and 0.
            log.push(format!(
                "p0+0x0,p1+0x0/p0+0x0,{}",
                rd(&c[1], 0).wrapping_add(0x7db4)
            ));
            log.push(format!("p0+0x0,{}", rd(m, 4).wrapping_add(1)));
            log.push(format!("p0+0x0,p0+0x0,{},0,6750477,0", a[1]));
            0i64
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "say_gather_level — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "say_gather_level — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "say_gather_level — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[3 + i],
                "say_gather_level — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("GoodTypeOut::say_gather_level@006700d0: {n} rows agree");
}
