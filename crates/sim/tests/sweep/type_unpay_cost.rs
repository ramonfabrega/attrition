//! `Type::unpay_cost@006682f0` under the emulator against the port (item 2837; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Type::unpay_cost@006682f0`, run by `tools/emu/sweep_obj.py` on its row `type_unpay_cost` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 16 random bytes of object, 3 argument(s), the global cell 0x00c061ec points at 2112 random bytes; callee 0x006e33a0 (2 args) stubbed; vtable slot +0x78 (7 args) stubbed; the word at `p0+0` points at 24 random bytes less `0` (placed); the word at `p1+0` points at 24 random bytes less `0` (placed); the word at `p2+0` points at 24 random bytes less `0` (placed); the word at `p3+0` points at 24 random bytes less `0` (placed); the word at `p4+0` points at 24 random bytes less `0` (placed); the word at `p5+0` points at 24 random bytes less `0` (placed); the word at `p6+0` points at 24 random bytes less `0` (placed); the word at `p7+0` points at 24 random bytes less `0` (placed); stub 0 answered from [0, 1] (90%); stub 1 answered from [0, 1, 2, 5, 100, -7, 3, 250] (90%); stub 0 answers 6 values in call order, printed after the answers; stub 1 answers 6 values in call order, printed after the answers; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); 8 rows 0x6eec apart at `@0xe3a390`, slices +0x6eb8:0x4 (`p` windows after the pointed buffers, before the places); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("type_unpay_cost") else {
        return;
    };
    let obj = super::Obj {
        name: "unpay_cost",
        full: "Type::unpay_cost@006682f0",
        cells: 1,
        calls: true,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        // Unless the game record's byte `+0x821` has bit 3 set, each of the six goods the leader
        // (`a[0]`) has available (`type_avail` answers `a[5..11]`) is refunded the cost slot `+0x78`
        // answers (`a[11..17]`, in call order, handed `(good, who, a[1], a[2], 1, 1, -1)`), added
        // to the leader's bucket stored XOR 0x8221 (item 2837).
        if c[0][0x821] & 8 == 0 {
            let who = a[0] as usize;
            let mut asked = 11;
            for g in 0..6usize {
                if a[5 + g] != 0 {
                    log.push(format!("s1:m+0x0,{g},{},{},{},1,1,-1", a[0], a[1], a[2]));
                    let cost = a[asked];
                    asked += 1;
                    let bucket = &mut p[8 + who];
                    let cur = rd(bucket, 4 * g) ^ 0x8221;
                    wr(bucket, 4 * g, cur.wrapping_add(cost) ^ 0x8221);
                }
            }
        }
        if log.is_empty() {
            log.push("-".into());
        }
        0
    });
}
