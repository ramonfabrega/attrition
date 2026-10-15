//! `City::could_queue@00737fa0` under the emulator against the port (item 2743; tools/sweep_gen.py, obj with windows).

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

/// `City::could_queue@00737fa0`, run by `tools/emu/sweep_city_could_queue.py` under unicorn
/// (tools/emu): 96 random bytes of object, 1 argument(s), vtable slot +0xac (0 args) stubbed; callee 0x0062da50 (1 args) stubbed; the word at `0xc0618c` points at 64 random bytes less `0` (placed); the word at `p0+0x14` points at 12 random bytes less `0` (placed); the word at `p0+0x30` points at 12 random bytes less `0` (placed); the word at `p1+0` points at 16 random bytes less `0` (placed); the word at `p1+4` points at 16 random bytes less `0` (placed); the word at `p1+8` points at 16 random bytes less `0` (placed); the word at `p2+0` points at 16 random bytes less `0` (placed); the word at `p2+4` points at 16 random bytes less `0` (placed); the word at `p2+8` points at 16 random bytes less `0` (placed); stub 1 answered from [0, 1] (90%); stub 0 answers a window of its own to each call, in call order (128 bytes, ...); the stubbed vtable laid at `p3+0`; stub 1 answers 3 values in call order, printed after the answers; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_city_could_queue.py") else {
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
            // `City::could_queue`: walk the chain of types from the city's first building (`+8`, a negative
            // index ends it) — each link's type answered by slot 0xac, the next link the type's short at
            // +0x74 — and answer 1 at the first the callee (`0x62da50`, handed the argument) accepts. The
            // stubs answer in call order (windows 9.. for the types, `a[3..6]` for the callee).
            let seq = [a[3], a[4], a[5]];
            let mut calls: Vec<String> = Vec::new();
            let mut r = 0;
            if rd(m, 8) as i16 >= 0 {
                for j in 0..3 {
                    calls.push(format!("p{}+0x0,{}", 9 + 2 * j, a[0]));
                    if seq[j] != 0 {
                        r = 1;
                        break;
                    }
                    if i16::from_le_bytes([p[10 + 2 * j][0x74], p[10 + 2 * j][0x75]]) < 0 {
                        break;
                    }
                }
            }
            log.push(if calls.is_empty() {
                "-".into()
            } else {
                calls.join("/")
            });
            r
        };
        assert_eq!(got, want_eax, "could_queue — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "could_queue — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "could_queue — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "could_queue — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "could_queue — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("City::could_queue@00737fa0: {n} rows agree");
}
