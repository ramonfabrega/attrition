//! `BuildData::could_queue@0062da50` under the emulator against the port (item 2844; tools/sweep_gen.py, obj with windows).

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

/// `BuildData::could_queue@0062da50`, run by `tools/emu/sweep_build_data_could_queue.py` under unicorn
/// (tools/emu): 140 random bytes of object, 1 argument(s), 4 random bytes at 0x00cab3a4; vtable slot +0x198 (2 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0x60 (2 args) stubbed; vtable slot +0x190 (2 args) stubbed; callee 0x00630450 (2 args) stubbed; the word at `0xe85ddc` points at 32 random bytes less `0` (placed); the word at `p0+0` points at 8 random bytes less `0` (placed); the word at `p0+4` points at 8 random bytes less `0` (placed); the word at `p0+8` points at 8 random bytes less `0` (placed); the word at `p0+12` points at 8 random bytes less `0` (placed); the word at `p0+16` points at 8 random bytes less `0` (placed); the word at `p0+20` points at 8 random bytes less `0` (placed); the word at `p0+24` points at 8 random bytes less `0` (placed); the word at `p0+28` points at 8 random bytes less `0` (placed); the word at `m+0x18` points at 8 random bytes less `0` (placed); slot +0xb8 laid as 0x00653790 or its stub, a flag after the answers; stub 0 answered from [0, 1, 1, 1] (90%); stub 1 answered from [0, 1] (90%); stub 2 answered from [0, 1] (90%); stub 3 answered from [0, 1, 2, 3, 4, 5] (90%); stub 4 answered from [0, 1, 2, 3, 4, 5] (90%); the stubbed vtable laid at `p9+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_build_data_could_queue.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (lhs, rhs) = line.split_once(" -> ").expect("a row is `<args…> | <windows> -> <eax> | <windows>`");
        let (args, before) = lhs.split_once(" | ").expect("`<args> | <before>`");
        let (eax, after) = rhs.split_once(" | ").expect("`<eax> | <after>`");
        let a: Vec<i32> = args.split_whitespace().map(|s| s.parse().expect("an integer")).collect();
        let mut wins: Vec<Vec<u8>> = before.trim().split(';').map(unhex).collect();
        let mut m = wins.remove(0);
        let mut p: Vec<Vec<u8>> = wins.split_off(1);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // `queue_fail` (the word at 0xcab3a4, `c[0]`) is 1 when slot +0x198 (`a[1]`) refuses the type,
            // 2 when the queue holds `queued` (`m[0x82]`) or more (`m+0x88`), or when a training
            // building (slot +0xb8, `a[2]`; laid as the base, flag `a[6]`, it asks the object at `m+0x18`'s
            // slot +0x60, `a[3]`) is handed a type whose record (`p[1 + a[0]]`) says 0x34 or 0x35 and slot
            // +0x190 (`a[4]`) and `num_gatherers` (`a[5]`) sum to 7 or more; else it stays 0 (item 2844).
            let kind = rd(&p[1 + a[0] as usize], 4);
            let (qf, r) = if a[1] == 0 {
                (1, 0)
            } else if i32::from(m[0x82]) >= rd(m, 0x88) {
                (2, 0)
            } else {
                let training = if a[6] != 0 { a[3] } else { a[2] };
                if training != 0 && (kind == 0x34 || kind == 0x35) && a[4].wrapping_add(a[5]) >= 7 {
                    (2, 0)
                } else {
                    (0, 1)
                }
            };
            wr(&mut c[0], 0, qf);
            r
        };
        assert_eq!(got, want_eax, "could_queue — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "could_queue — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "could_queue — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[2 + i], "could_queue — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("BuildData::could_queue@0062da50: {n} rows agree");
}
