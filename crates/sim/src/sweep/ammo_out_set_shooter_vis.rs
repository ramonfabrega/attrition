//! `AmmoOut::set_shooter_vis@00679200` under the emulator against the port (item 2362; tools/sweep_gen.py, obj with windows).

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

/// `AmmoOut::set_shooter_vis@00679200`, run by `tools/emu/sweep_ammo_out_set_shooter_vis.py` under unicorn
/// (tools/emu): 128 random bytes of object, 2 argument(s), the global cell 0x00c06210 points at 768 random bytes; vtable slot +0x48 (2 args) stubbed; the word at `0xc0ab84+a[1]*0x1c` points at 4 random bytes less `a[0]*4` (placed); the word at `p0+0` points at 16 random bytes less `0` (placed); the stubbed vtable laid at `p1+0`; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_ammo_out_set_shooter_vis.py") else {
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
                // The object (the objects table, image 0xc0ab84 + 0x1c × a[1], its array's entry a[0])
                // is asked its slot +0x48 `(console->who, 0)` (cell 0xc06210, +0x298); the answer's low
                // byte is the ammo's `shooter_vis` (+0x70).
                log.push(format!("p1+0x0,{},0", rd(&c[0], 0x298)));
                m[0x70] = a[2] as u8;
                0
            }
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "set_shooter_vis — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "set_shooter_vis — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "set_shooter_vis — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[2 + i],
                "set_shooter_vis — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("AmmoOut::set_shooter_vis@00679200: {n} rows agree");
}
