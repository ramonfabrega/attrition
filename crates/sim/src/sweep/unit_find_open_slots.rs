//! `Unit::find_open_slots@00600e30` under the emulator against the port (item 2400; tools/sweep_gen.py, obj with windows).

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

/// `Unit::find_open_slots@00600e30`, run by `tools/emu/sweep_unit_find_open_slots.py` under unicorn
/// (tools/emu): 32 random bytes of object, 3 argument(s), the global cell 0x00c0618c points at 256 random bytes; the global cell 0x00c06188 points at 64 random bytes; the global cell 0x00cae5fc points at 1024 random bytes; argument 2 names 20 random bytes; vtable slot +0xb8 (2 args) stubbed; callee 0x00607c30 (8 args) stubbed; callee 0x0065b1b0 (5 args) stubbed; callee 0x0065b440 (4 args) stubbed; the word at `p0+0x10` points at 1024 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*a[1]` points at 32 random bytes less `0` (placed); the word at `p2+4*a[0]` points at 32 random bytes less `0` (placed); the word at `p3+0x18` points at 608 random bytes less `0` (placed); the word at `m+0x18` points at 608 random bytes less `0` (placed); the word at `c0+0x14+0x1c*a[1]` points at 32 random bytes less `0` (placed); the word at `p6+4*a[0]` points at 32 random bytes less `0` (placed); stub 0 answered from [0, 1] (90%); stub 1 answered from [0, 0, 0, 0, 5] (90%); stub 2 answered from [0, 0, 0, 0, 1] (90%); stub 3 answered from [0, 0, 0, 0, 1] (90%); stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_find_open_slots.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(3);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // `a[0]` the target's id, `a[1]` its list, `a[2]` the out array
            // (`p[0]`: `+4` count, `+0x10` data = `p[1]`); `a[3]` the `is`
            // slot's answer, `a[4]`/`a[5]`/`a[6]` the three predicates'.
            // The asker's type is `p[5]`, the target's `p[4]`, the target's
            // slot `p[7]` (`+0x10`/`+0x14` the masked position), the world
            // `c[1]` (`+0`, `+4` its size in cells).
            let centre = crate::collide::ucell(crate::world::Pos::new(
                rd(&p[7], 0x10) ^ 0x63637,
                rd(&p[7], 0x14) ^ 0x63637,
            ));
            let dims = (rd(&c[1], 0), rd(&c[1], 4));
            let slots = crate::attack_pos::find_open_slots_steps(
                rd(&p[5], 0x248),
                rd(&p[4], 0x248),
                a[3] != 0,
                centre,
                dims,
                |at, tl| {
                    log.push(format!("s1:m+0x0,{},{},0,0,0,0,1,{}", tl.x, tl.y, tl.y));
                    if a[4] != 0 {
                        return false;
                    }
                    let (sh, by) = (i16::from_le_bytes([m[0xa], m[0xb]]), m[9]);
                    log.push(format!("s2:{},{},{},{sh},{by},0", at.x, at.x, at.y));
                    if a[5] != 0 {
                        return false;
                    }
                    log.push(format!("s3:m+0x0,{},{},{sh},{by}", at.x, at.y));
                    a[6] == 0
                },
            );
            let mut n = rd(&p[0], 4);
            for s in &slots {
                wr(&mut p[1], 8 * n as usize, s.x);
                wr(&mut p[1], 8 * n as usize + 4, s.y);
                n += 1;
            }
            wr(&mut p[0], 4, n);
            if log.is_empty() {
                log.push("-".into());
            }
            i64::from(!slots.is_empty())
        };
        assert_eq!(got, want_eax, "find_open_slots — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "find_open_slots — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "find_open_slots — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "find_open_slots — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[4 + i],
                "find_open_slots — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 1000, "only {n} rows");
    eprintln!("Unit::find_open_slots@00600e30: {n} rows agree");
}
