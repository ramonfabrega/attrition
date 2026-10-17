//! `SubObject::set_type@00662450` under the emulator against the port (item 2946; tools/sweep_gen.py, obj with windows).

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

/// `SubObject::set_type@00662450`, run by `tools/emu/sweep_sub_object_set_type.py` under unicorn
/// (tools/emu): 64 random bytes of object, 2 argument(s), vtable slot +0xb0 (0 args) stubbed; vtable slot +0x34 (0 args) stubbed; callee 0x00643070 (1 args) stubbed; callee 0x005e2920 (0 args) stubbed; callee 0x008e4780 (1 args) stubbed; the word at `0xc061f8` points at 32 random bytes less `0` (placed); the word at `p0+0x10` points at 2304 random bytes less `0` (placed); the word at `0xc0618c` points at 128 random bytes less `0` (placed); the word at `p2+0x14` points at 16 random bytes less `0` (placed); the word at `p2+0x30` points at 16 random bytes less `0` (placed); the word at `p3+0` points at 32 random bytes less `0` (placed); the word at `p3+4` points at 32 random bytes less `0` (placed); the word at `p3+8` points at 32 random bytes less `0` (placed); the word at `p4+0` points at 32 random bytes less `0` (placed); the word at `p4+4` points at 32 random bytes less `0` (placed); the word at `p4+8` points at 32 random bytes less `0` (placed); the word at `0xc0aec0` points at 16 random bytes less `0` (placed); the word at `0xc0aedc` points at 16 random bytes less `0` (placed); stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p10+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1200 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_sub_object_set_type.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (line, calls) = line.split_once(" # ").expect("`<row> # <calls>`");
        let want_log: Vec<&str> = calls.split_whitespace().collect();
        let (lhs, rhs) = line.split_once(" -> ").expect("a row is `<args…> | <windows> -> <eax> | <windows>`");
        let (args, before) = lhs.split_once(" | ").expect("`<args> | <before>`");
        let (eax, after) = rhs.split_once(" | ").expect("`<eax> | <after>`");
        let a: Vec<i32> = args.split_whitespace().map(|s| s.parse().expect("an integer")).collect();
        let mut wins: Vec<Vec<u8>> = before.trim().split(';').map(unhex).collect();
        let mut m = wins.remove(0);
        let mut p: Vec<Vec<u8>> = wins.split_off(0);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // set_type: this+0x18 takes the objecttypes entry; a build-class type (0x19e..=0x21e) hands the object's Wall record
            // (slot 0xb0's answer, with the record) to Wall::update_gpiece, a unit-class type (0x32..=0x191) the unit's own
            // pointer to Unit::update_gpiece; the graphic load takes slot 0x34's answer either way.
            log.extend(["-"; 3].map(String::from));
            m[0x18..0x1c].copy_from_slice(&p[1][4 * a[0] as usize..4 * a[0] as usize + 4]);
            let owner = m[9] as usize;
            let idx = i16::from_le_bytes([m[10], m[11]]) as usize;
            if ((a[0] - 0x19e) as u32) <= 0x80 {
                log[0] = format!("{},p{}+0x0", a[2], 5 + owner * 3 + idx);
            } else if ((a[0] - 0x32) as u32) <= 0x15f {
                log[1] = rd(&p[11 + owner], 4 * idx).to_string();
            }
            log[2] = format!("m+0x0,{}", a[3]);
            0
        };
        let _ = (got, want_eax);
        assert_eq!(log, want_log, "set_type — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "set_type — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "set_type — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "set_type — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 1200, "only {n} rows");
    eprintln!("SubObject::set_type@00662450: {n} rows agree");
}
