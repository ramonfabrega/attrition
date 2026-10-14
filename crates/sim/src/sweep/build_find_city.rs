//! `Build::find_city@00628180` under the emulator against the port (item 2314; tools/sweep_gen.py, obj with windows).

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

/// `Build::find_city@00628180`, run by `tools/emu/sweep_build_find_city.py` under unicorn
/// (tools/emu): 128 random bytes of object, 0 argument(s), the global cell 0x00cae5fc points at 4096 random bytes; callee 0x00622030 (0 args) stubbed; callee 0x00639b90 (3 args) stubbed; vtable slot +0xac (0 args) stubbed; callee 0x00622380 (1 args) stubbed; the word at `0xc0618c` points at 256 random bytes less `0` (placed); the word at `p0+0x14+0x1c*m[9]` points at 24 random bytes less `0` (placed); the word at `p1+0` points at 16 random bytes less `0` (placed); the word at `p1+4` points at 16 random bytes less `0` (placed); the word at `p1+8` points at 16 random bytes less `0` (placed); the word at `p1+12` points at 16 random bytes less `0` (placed); the word at `p1+16` points at 16 random bytes less `0` (placed); the word at `p1+20` points at 16 random bytes less `0` (placed); stub 2 answers the address of 128 random bytes, a `p` window after the places; stub 1 answers -3..5 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_build_find_city.py") else {
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
            // `this`: flags `+8`, owner `+9`, its own index `+0xa`, the
            // position `+0x10`/`+0x14` (keyed, through `div_3_table`, window
            // c0), the type record `+0x18` (`get_town`'s `ecx`), the city
            // `+0x72`. Stubs: `remove_from_city` a[0], `get_town` a[1], slot
            // `0xac` of `objects[who][r]` a[2] (window p8, whose `+0x72` is
            // the city), `add_to_city` a[3]. The log: `remove_from_city`'s
            // `ecx`, `get_town`'s `ecx,x,y,who`, `add_to_city`'s `ecx,city`.
            let who = i32::from(m[9]);
            let id = i32::from(i16::from_le_bytes([m[0xa], m[0xb]]));
            let city = i16::from_le_bytes([m[0x72], m[0x73]]);
            let div3 = |w: i32| rd(&c[0], (((w ^ 0x63637) >> 6) as usize) * 4);
            let (x, y, ecx) = (div3(rd(m, 0x10)), div3(rd(m, 0x14)), rd(m, 0x18));
            let mut rec: [Vec<String>; 3] = Default::default();
            let r = crate::city::find_city_steps(
                &mut rec,
                (m[8] & 0x20 != 0).then_some(id),
                city >= 0,
                |l| l[0].push("m+0x0".into()),
                |l| {
                    l[1].push(format!("{ecx},{x},{y},{who}"));
                    a[1]
                },
                |_, _| i32::from(i16::from_le_bytes([p[8][0x72], p[8][0x73]])),
                |l, cty| l[2].push(format!("m+0x0,{cty}")),
            );
            for l in rec {
                log.push(if l.is_empty() {
                    "-".into()
                } else {
                    l.join("/")
                });
            }
            i64::from(r)
        };
        assert_eq!(got, want_eax, "find_city — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "find_city — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "find_city — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "find_city — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[2 + i], "find_city — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Build::find_city@00628180: {n} rows agree");
}
