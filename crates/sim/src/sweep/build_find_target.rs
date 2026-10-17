//! `Build::find_target@00622c80` under the emulator against the port (item 2857; tools/sweep_gen.py, obj with windows).

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

/// `Build::find_target@00622c80`, run by `tools/emu/sweep_build_find_target.py` under unicorn
/// (tools/emu): 144 random bytes of object, 0 argument(s), vtable slot +0x130 (0 args) stubbed; callee 0x00648da0 (5 args) stubbed; the word at `m+0x18` points at 576 random bytes less `0` (placed); stub 0 answers 0..40 every row; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 writes a value in 0..255 through its argument 1, printed after the answers (`o` in a place); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_build_find_target.py") else {
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
                // The search radius is the larger of the type record's two ranges (`+0x234`, `+0x238`)
                // plus twice the slot's answer, in 0x60 units; the found target's low byte, written
                // through the out-pointer, lands at +0x81, the callee's answer is kept as a short
                // at +0x7c, and bit 2 of the short at +0x60 is cleared.
                let reach = rd(&p[0], 0x234).max(rd(&p[0], 0x238));
                let range = reach.wrapping_add(a[0].wrapping_mul(2)).wrapping_mul(0x60);
                log.push(format!("m+0x0,{range},sp-0xc,1,0,0"));
                let flags = u16::from_le_bytes([m[0x60], m[0x61]]) & 0xfffb;
                m[0x60..0x62].copy_from_slice(&flags.to_le_bytes());
                m[0x7c..0x7e].copy_from_slice(&(a[1] as i16).to_le_bytes());
                m[0x81] = a[2] as u8;
                0
            }
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "find_target — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "find_target — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "find_target — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "find_target — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Build::find_target@00622c80: {n} rows agree");
}
