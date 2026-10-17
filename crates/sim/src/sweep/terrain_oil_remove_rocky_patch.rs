//! `TerrainOil::remove_rocky_patch@008a2ac0` under the emulator against the port (item 2857; tools/sweep_gen.py, obj with windows).

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

/// `TerrainOil::remove_rocky_patch@008a2ac0`, run by `tools/emu/sweep_terrain_oil_remove_rocky_patch.py` under unicorn
/// (tools/emu): 32 random bytes of object, 2 argument(s), the global cell 0x00c06188 points at 16 random bytes; callee 0x0055d063 (4 args) stubbed; callee 0x0055d055 (2 args) stubbed; the word at `m+0x14` points at 48 random bytes less `0` (placed); the word at `p0+0` points at 64 random bytes less `-4` (placed, null a third of the rows); the word at `p0+4` points at 64 random bytes less `-4` (placed, null a third of the rows); the word at `p0+8` points at 64 random bytes less `-4` (placed, null a third of the rows); the word at `p0+12` points at 64 random bytes less `-4` (placed, null a third of the rows); the word at `p0+16` points at 64 random bytes less `-4` (placed, null a third of the rows); the word at `p0+20` points at 64 random bytes less `-4` (placed, null a third of the rows); the word at `p0+24` points at 64 random bytes less `-4` (placed, null a third of the rows); the word at `p0+28` points at 64 random bytes less `-4` (placed, null a third of the rows); the word at `p0+32` points at 64 random bytes less `-4` (placed, null a third of the rows); the word at `p0+36` points at 64 random bytes less `-4` (placed, null a third of the rows); the word at `p0+40` points at 64 random bytes less `-4` (placed, null a third of the rows); the word at `p0+44` points at 64 random bytes less `-4` (placed, null a third of the rows); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 is `__cdecl`: the caller pops its arguments; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_terrain_oil_remove_rocky_patch.py") else {
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
            // `TerrainOil::remove_rocky_patch(x, y)`: the slot `patch_data[w * y + x]` (the pointer table at
            // `this+0x14`, `w` the first word of the `GameAccess::world` record at 0xc06188). A null slot is left
            // alone; a record pointer `q` (an `OilPatchData[]` with its count at `q-4`) is destroyed through
            // `_eh_vector_destructor_iterator_(q, 0x3c, count, OilPatchData::~OilPatchData)` (stub 0), freed with
            // `operator delete[](q-4, (count*16 - count)*4 + 4)` (stub 1, `cdecl`), and the slot zeroed.
            // `eax` is not read (`--ret none`). Entry k names window p(1+k); the second call's `ecx` is the leftover `count * 15`.
            let idx = rd(&c[0], 0).wrapping_mul(a[1]).wrapping_add(a[0]);
            assert!(
                (0..12).contains(&idx),
                "the draw keeps the slot inside the table"
            );
            let at = idx as usize * 4;
            let q = rd(&p[0], at);
            if q != 0 {
                let w = 1 + idx as usize;
                let count = rd(&p[w], 0);
                let ecx = count.wrapping_mul(16).wrapping_sub(count);
                let size = ecx.wrapping_mul(4).wrapping_add(4);
                log.push(format!("m+0x0,p{w}+0x4,60,{count},4874304"));
                log.push(format!("{ecx},p{w}+0x0,{size}"));
                wr(&mut p[0], at, 0);
            } else {
                log.push("-".into());
                log.push("-".into());
            }
            0
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "remove_rocky_patch — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(
            m, want[0],
            "remove_rocky_patch — this's bytes, row `{line}`"
        );
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "remove_rocky_patch — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[2 + i],
                "remove_rocky_patch — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("TerrainOil::remove_rocky_patch@008a2ac0: {n} rows agree");
}
