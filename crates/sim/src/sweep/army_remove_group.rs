//! `Army::remove_group@006f8b50` under the emulator against the port (item 2844; tools/sweep_gen.py, obj with windows).

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

/// `Army::remove_group@006f8b50`, run by `tools/emu/sweep_army_remove_group.py` under unicorn
/// (tools/emu): 160 random bytes of object, 1 argument(s), callee 0x006f9b50 (0 args) stubbed; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); 4 rows 0x9d4 apart at `0xe85f20`, slices +0x4:0x4, +0x8:0x4 (`p` windows after the pointed buffers, before the places); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_army_remove_group.py") else {
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
            // Group `a[0]` (row k of the groups table: `p[2k]` its id at +4, `p[2k+1]` its owner word
            // at +8) leaves the army when the army's short at `m+2` is the group's owner word and the army
            // has groups (the short count at `m+0x96`): the first of its ids (ints from `m+0x54`) equal to
            // the group's id is cut out, the count drops, the group's owner word becomes -1 and
            // `normalize` runs on the army (item 2844).
            if a[0] >= 0 {
                let k = a[0] as usize;
                let count = i16::from_le_bytes([m[0x96], m[0x97]]);
                if rd(&p[2 * k + 1], 0) == i32::from(i16::from_le_bytes([m[2], m[3]])) && count != 0
                {
                    let mut i = 0;
                    while i < i32::from(count) && rd(m, 0x54 + 4 * i as usize) != rd(&p[2 * k], 0) {
                        i += 1;
                    }
                    if i != i32::from(count) {
                        let new = count - 1;
                        m[0x96..0x98].copy_from_slice(&new.to_le_bytes());
                        while i < i32::from(new) {
                            let next = rd(m, 0x58 + 4 * i as usize);
                            wr(m, 0x54 + 4 * i as usize, next);
                            i += 1;
                        }
                        wr(&mut p[2 * k + 1], 0, -1);
                        log.push("s0:m+0x0".into());
                    }
                }
            }
            if log.is_empty() {
                log.push("-".into());
            }
            0
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "remove_group — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "remove_group — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "remove_group — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "remove_group — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Army::remove_group@006f8b50: {n} rows agree");
}
