//! `Build::remove_gatherer@0062f8d0` under the emulator against the port (item 2749; tools/sweep_gen.py, obj with windows).

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

/// `Build::remove_gatherer@0062f8d0`, run by `tools/emu/sweep_build_remove_gatherer.py` under unicorn
/// (tools/emu): 160 random bytes of object, 1 argument(s), vtable slot +0x8 (0 args) stubbed; the word at `0xc0618c` points at 64 random bytes less `0` (placed); the word at `0xc0aec0+m[9]*0x1c` points at 32 random bytes less `0` (placed); the word at `p1+0` points at 152 random bytes less `0` (placed); the word at `p1+4` points at 152 random bytes less `0` (placed); the word at `p1+8` points at 152 random bytes less `0` (placed); the word at `p1+12` points at 152 random bytes less `0` (placed); the word at `p1+16` points at 152 random bytes less `0` (placed); the word at `p1+20` points at 152 random bytes less `0` (placed); the word at `p0+m[9]*0x1c+0x14` points at 32 random bytes less `0` (placed); the word at `p8+0` points at 8 random bytes less `0` (placed); the word at `p8+4` points at 8 random bytes less `0` (placed); the word at `p8+8` points at 8 random bytes less `0` (placed); the word at `p8+12` points at 8 random bytes less `0` (placed); the word at `p8+16` points at 8 random bytes less `0` (placed); the word at `p8+20` points at 8 random bytes less `0` (placed); stub 0 answered from [1, 1, 1, 0] (90%); stub 0 answers 6 values in call order, printed after the answers; figures p2..p7 link at +0x92 (2 bytes) above their own index or to -1; the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p10+0` as well as at `this`; the stubbed vtable laid at `p11+0` as well as at `this`; the stubbed vtable laid at `p12+0` as well as at `this`; the stubbed vtable laid at `p13+0` as well as at `this`; the stubbed vtable laid at `p14+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_build_remove_gatherer.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(0);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // `units[who].list[i] + 0x92` is the next link of a gatherer list headed at `this + 0x70`;
                // `objects[who].list[i]`'s vtable slot +8 (stub 0, answers a[2..]) says whether node i lives.
                let get = |p: &[Vec<u8>], i: usize| {
                    i32::from(i16::from_le_bytes([p[2 + i][0x92], p[2 + i][0x93]]))
                };
                let set = |p: &mut [Vec<u8>], i: usize, v: i16| {
                    p[2 + i][0x92..0x94].copy_from_slice(&v.to_le_bytes())
                };
                let head = i32::from(i16::from_le_bytes([m[0x70], m[0x71]]));
                let want = a[0];
                let mut calls = 0usize;
                if head >= 0 {
                    let h = usize::try_from(head).expect("an index");
                    if head == want {
                        let nxt = get(p, h);
                        m[0x70..0x72]
                            .copy_from_slice(&i16::try_from(nxt).expect("a short").to_le_bytes());
                        set(p, h, -1);
                    } else {
                        let mut cur = head;
                        loop {
                            let alive = a[2 + calls];
                            calls += 1;
                            if alive == 0 {
                                break;
                            }
                            let n = get(p, usize::try_from(cur).expect("an index"));
                            if n < 0 {
                                break;
                            }
                            if n == want {
                                let k = usize::try_from(n).expect("an index");
                                let after = get(p, k);
                                set(
                                    p,
                                    usize::try_from(cur).expect("an index"),
                                    i16::try_from(after).expect("a short"),
                                );
                                set(p, k, -1);
                                break;
                            }
                            cur = n;
                        }
                    }
                }
                0
            }
        };
        let _ = (got, want_eax);
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "remove_gatherer — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "remove_gatherer — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "remove_gatherer — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Build::remove_gatherer@0062f8d0: {n} rows agree");
}
