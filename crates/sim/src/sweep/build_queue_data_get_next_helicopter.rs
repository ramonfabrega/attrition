//! `BuildQueueData::get_next_helicopter@00630bb0` under the emulator against the port (item 2655; tools/sweep_gen.py, obj with windows).

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

/// `BuildQueueData::get_next_helicopter@00630bb0`, run by `tools/emu/sweep_build_queue_data_get_next_helicopter.py` under unicorn
/// (tools/emu): 16 random bytes of object, 2 argument(s), the global cell 0x00e85ddc points at 16 random bytes; vtable slot +0x60 (2 args) stubbed; the word at `m+4` points at 104 random bytes less `0` (placed); the word at `c0+0` points at 16 random bytes less `0` (placed); the word at `c0+4` points at 16 random bytes less `0` (placed); the word at `c0+8` points at 16 random bytes less `0` (placed); the word at `c0+0xc` points at 16 random bytes less `0` (placed); stub 0 answered from [0, 0, 0, 5] (90%); the stubbed vtable laid at `p1+0`; stub 0 answers 4 values in call order, the last from [5], printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_build_queue_data_get_next_helicopter.py") else {
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
            {
                // Of the first min(a0, queue size) queue records (0x14 apiece, the short at +4 a type index into the types list), the first from record 1 whose type answers its slot 0x60 (0x136, 0) with non-zero; -1 when none (the stub'"'"'s answers a call in call order, the last repeated).
                let end = a[0].min(rd(m, 0));
                let mut calls = Vec::new();
                let mut r = -1;
                let mut k = 0;
                let mut i = 1;
                while i < end {
                    let at = 0x14 * i as usize + 4;
                    let ty = i16::from_le_bytes([p[0][at], p[0][at + 1]]);
                    calls.push(format!("p{}+0x0,310,0", 1 + ty));
                    if a[3 + k.min(3)] != 0 {
                        r = i;
                        break;
                    }
                    k += 1;
                    i += 1;
                }
                log.push(if calls.is_empty() {
                    "-".into()
                } else {
                    calls.join("/")
                });
                i64::from(r)
            }
        };
        assert_eq!(got, want_eax, "get_next_helicopter — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "get_next_helicopter — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(
            m, want[0],
            "get_next_helicopter — this's bytes, row `{line}`"
        );
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "get_next_helicopter — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[2 + i],
                "get_next_helicopter — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("BuildQueueData::get_next_helicopter@00630bb0: {n} rows agree");
}
