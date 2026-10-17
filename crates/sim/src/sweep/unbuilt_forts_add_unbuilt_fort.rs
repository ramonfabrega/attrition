//! `UnbuiltForts::add_unbuilt_fort@0073bc00` under the emulator against the port (item 2783; tools/sweep_gen.py, obj with windows).

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

/// `UnbuiltForts::add_unbuilt_fort@0073bc00`, run by `tools/emu/sweep_unbuilt_forts_add_unbuilt_fort.py` under unicorn
/// (tools/emu): 32 random bytes of object, 2 argument(s), 112 random bytes at 0x00e39b20; vtable slot +0x0 (1 args) stubbed; the word at `c0+0x10` points at 64 random bytes less `0` (placed); the word at `c0+0x2c` points at 64 random bytes less `0` (placed); the word at `c0+0x48` points at 64 random bytes less `0` (placed); the word at `c0+0x64` points at 64 random bytes less `0` (placed); the stubbed vtable laid at `c0+0`; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the stubbed vtable laid at `c0+0x1c` as well as at `this`; the stubbed vtable laid at `c0+0x38` as well as at `this`; the stubbed vtable laid at `c0+0x54` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unbuilt_forts_add_unbuilt_fort.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(1);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // `UnbuiltForts::add_unbuilt_fort(packed, who)`: list `who` at `0xe39b20 + 0x1c * who` (`c[0]`)
            // grows through its vtable slot 0 (`ecx` the list, handed its short at `+0xc`) when `count >=
            // cap`; then `packed` with byte 2 replaced by `who` is stored at `data[count]` (`+0x10`, the
            // place `p[who]`) and `count` goes up by one.
            let (packed, who) = (a[0], a[1] as usize);
            let base = 0x1c * who;
            let count = rd(&c[0], base + 4);
            let call = if count >= rd(&c[0], base + 8) {
                let w = i16::from_le_bytes([c[0][base + 0xc], c[0][base + 0xd]]);
                format!("c0+{base:#x},{w}")
            } else {
                "-".to_string()
            };
            log.push(call);
            let v = (packed & !0x00ff_0000) | ((who as i32 & 0xff) << 16);
            wr(&mut p[who], 4 * count as usize, v);
            wr(&mut c[0], base + 4, count + 1);
            0i64
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "add_unbuilt_fort — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "add_unbuilt_fort — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "add_unbuilt_fort — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[2 + i],
                "add_unbuilt_fort — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("UnbuiltForts::add_unbuilt_fort@0073bc00: {n} rows agree");
}
