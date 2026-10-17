//! `Doober::remove_hold_doobers@00846cd0` under the emulator against the port (item 3038; tools/sweep_gen.py, obj with windows).

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

/// `Doober::remove_hold_doobers@00846cd0`, run by `tools/emu/sweep_doober_remove_hold_doobers.py` under unicorn
/// (tools/emu): 512 random bytes of object, 1 argument(s), `p` windows in order: p0 the place; 4 random bytes at 0x00ee121c; the word at `m+0x110` points at 64 random bytes less `0` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 800 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_doober_remove_hold_doobers.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(1);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // Index under the count at `+0xcc`: the hold-fade array's slot (the pointer at `+0x110`) set to
            // 0xffffff01; otherwise the error report (muted by `ignore_always`, cell 0xee121c) and the same OK.
            if a[0] < rd(m, 0xcc) {
                wr(&mut p[0], 4 * a[0] as usize, 0xffffff01u32 as i32);
            }
            0
        };
        assert_eq!(got, want_eax, "remove_hold_doobers — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(
            m, want[0],
            "remove_hold_doobers — this's bytes, row `{line}`"
        );
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "remove_hold_doobers — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[2 + i],
                "remove_hold_doobers — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 800, "only {n} rows");
    eprintln!("Doober::remove_hold_doobers@00846cd0: {n} rows agree");
}
