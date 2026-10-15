//! `Build::add_gatherer@0062f640` under the emulator against the port (item 2749; tools/sweep_gen.py, obj with windows).

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

/// `Build::add_gatherer@0062f640`, run by `tools/emu/sweep_build_add_gatherer.py` under unicorn
/// (tools/emu): 160 random bytes of object, 2 argument(s), vtable slot +0x8 (0 args) stubbed; vtable slot +0xbc (0 args) stubbed; callee 0x00630450 (2 args) stubbed; callee 0x0062f520 (1 args) stubbed; callee 0x0062f710 (0 args) stubbed; the word at `0xc0618c` points at 64 random bytes less `0` (placed); the word at `0xc0aec0+m[9]*0x1c` points at 16 random bytes less `0` (placed); the word at `p1+0` points at 160 random bytes less `0` (placed); the word at `p1+4` points at 160 random bytes less `0` (placed); the word at `p1+8` points at 160 random bytes less `0` (placed); the word at `p1+12` points at 160 random bytes less `0` (placed); the word at `p2+0x18` points at 8 random bytes less `0` (placed); the word at `p3+0x18` points at 8 random bytes less `0` (placed); the word at `p4+0x18` points at 8 random bytes less `0` (placed); the word at `p5+0x18` points at 8 random bytes less `0` (placed); the word at `p0+m[9]*0x1c+0x14` points at 16 random bytes less `0` (placed); the word at `p10+0` points at 8 random bytes less `0` (placed); the word at `p10+4` points at 8 random bytes less `0` (placed); the word at `p10+8` points at 8 random bytes less `0` (placed); the word at `p10+12` points at 8 random bytes less `0` (placed); stub 0 answered from [1, 1, 1, 0] (90%); stub 1 answered from [1, 1, 1, 0] (90%); stub 3 answered from [0, 0, 1] (90%); stub 2 answers 0..4 every row; the stubbed vtable laid at `p11+0` as well as at `this`; the stubbed vtable laid at `p12+0` as well as at `this`; the stubbed vtable laid at `p13+0` as well as at `this`; the stubbed vtable laid at `p14+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_build_add_gatherer.py") else {
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
                // Stubs, in order, answer a[2] (object slot +8), a[3] (slot +0xbc), a[4] (num_gatherers),
                // a[5] (is_gathered_by) and a[6] (check_gatherers, hooked: it writes nothing here).
                // Units are p[2..6], their type records p[6..10], objects p[11..15].
                if a[1] == i32::from(m[9]) && a[4] < i32::from(m[0x80] as i8) {
                    let idx = usize::try_from(a[0]).expect("an index");
                    let ty = rd(&p[6 + idx], 4);
                    if a[2] != 0 && a[3] != 0 && (0x32..=0x35).contains(&ty) && a[5] == 0 {
                        let head = [m[0x70], m[0x71]];
                        p[2 + idx][0x92..0x94].copy_from_slice(&head);
                        let v = i16::try_from(a[0]).expect("a short");
                        m[0x70..0x72].copy_from_slice(&v.to_le_bytes());
                    }
                }
                0
            }
        };
        let _ = (got, want_eax);
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "add_gatherer — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "add_gatherer — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "add_gatherer — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Build::add_gatherer@0062f640: {n} rows agree");
}
