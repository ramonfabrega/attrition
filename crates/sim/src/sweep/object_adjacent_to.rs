//! `Object::adjacent_to@00651f40` under the emulator against the port (item 2368; tools/sweep_gen.py, obj with windows).

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

/// `Object::adjacent_to@00651f40`, run by `tools/emu/sweep_object_adjacent_to.py` under unicorn
/// (tools/emu): 32 random bytes of object, 2 argument(s), the global cell 0x00c0618c points at 256 random bytes; the global cell 0x00c061f0 points at 256 random bytes; vtable slot +0x18 (0 args) stubbed; callee 0x0064c880 (2 args) stubbed; callee 0x006488f0 (4 args) stubbed; the word at `c0+0x14+0x1c*a[1]` points at 16 random bytes less `0` (placed); the word at `p0+4*a[0]` points at 16 random bytes less `0` (placed); the word at `m+0x18` points at 544 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*m[9]` points at 32 random bytes less `0` (placed); the word at `p3+4*i16(m,0xa)` points at 32 random bytes less `0` (placed); the word at `p4+0x18` points at 696 random bytes less `0` (placed); stub 0 answered from [0, 1, 1] (90%); stub 1 answered from [0, 99, 100, 575, 576, 577, 483, 484, 485, 959, 960, 961, 2000] (90%); stub 2 answered from [0, 95, 96, 97, 1000] (90%); stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1500 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_object_adjacent_to.py") else {
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
            // `a[0]` the target's id, `a[1]` its player; the answers from
            // `a[2]`: `+0x18` (is a unit), the boat arm's `attack_dist`, the
            // generic arm's `attack_dist`. This's slot is `m` (`+8` in use,
            // `+0x10`/`+0x14` the masked position), the target's `p[1]`
            // (`+8`), this's type `p[2]` (`+0x218` the domain), the unit's
            // own type read back through the units table `p[5]` (`+0x2b4`
            // the flags, `+0x248` `coll_size`), the constants `c[1]`
            // (`+0x94`).
            let (x, y) = (rd(m, 0x10) ^ 0x63637, rd(m, 0x14) ^ 0x63637);
            let (p2, p5) = (&p[2], &p[5]);
            let ok = crate::orders::adjacent_steps(
                m[8] & 1 != 0,
                p[1][8] & 1 != 0,
                rd(p2, 0x218) == 1,
                || a[2] != 0,
                || (p5[0x2b4] & 0x10 != 0, rd(p5, 0x248)),
                rd(&c[1], 0x94),
                |boat| {
                    if boat {
                        log.push(format!("s1:m+0x0,{},{}", a[0], a[1]));
                        a[3]
                    } else {
                        log.push(format!("s2:m+0x0,{},{},{x},{y}", a[0], a[1]));
                        a[4]
                    }
                },
            );
            if log.is_empty() {
                log.push("-".into());
            }
            i64::from(ok)
        };
        assert_eq!(got, want_eax, "adjacent_to — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "adjacent_to — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "adjacent_to — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "adjacent_to — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[3 + i], "adjacent_to — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 1500, "only {n} rows");
    eprintln!("Object::adjacent_to@00651f40: {n} rows agree");
}
