//! `GroupData::equals_group_const@0070cdf0` under the emulator against the port (item 2346; tools/sweep_gen.py, obj with windows).

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

/// `GroupData::equals_group_const@0070cdf0`, run by `tools/emu/sweep_group_data_equals_group_const.py` under unicorn
/// (tools/emu): 2268 random bytes of object, 1 argument(s), argument 0 names 2268 random bytes; callee 0x0070d000 (0 args) stubbed; stub 0 answered from [0, 1] (90%); stub 0 answers 2 values in call order, printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_group_data_equals_group_const.py") else {
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
            {
                // The owner byte (+0x4a) first; then `get_num_const` (stub 0) of the other group, then
                // of this one, equal, the building flag (+0x49) equal, and the first `num` (+0xc) unit
                // indices of the lists (+0x8cc, shorts) equal.
                let q = &p[0];
                if m[0x4a] != q[0x4a] {
                    log.push("-".into());
                    0
                } else {
                    log.push("p0+0x0/m+0x0".into());
                    let same = a[3] == a[2]
                        && m[0x49] == q[0x49]
                        && (0..rd(m, 0xc).max(0) as usize).all(|i| {
                            m[0x8cc + 2 * i..0x8ce + 2 * i] == q[0x8cc + 2 * i..0x8ce + 2 * i]
                        });
                    i64::from(same)
                }
            }
        };
        assert_eq!(got, want_eax, "equals_group_const — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "equals_group_const — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(
            m, want[0],
            "equals_group_const — this's bytes, row `{line}`"
        );
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "equals_group_const — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "equals_group_const — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("GroupData::equals_group_const@0070cdf0: {n} rows agree");
}
