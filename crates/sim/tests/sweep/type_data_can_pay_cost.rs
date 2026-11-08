//! `TypeData::can_pay_cost@00667570` under the emulator against the port (item 2247; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `TypeData::can_pay_cost@00667570`, run by `tools/emu/sweep_obj.py` on its row `type_data_can_pay_cost` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 32 random bytes of object, 4 argument(s), the global cell 0x00c061e8 points at 2112 random bytes; callee 0x006e33a0 (2 args) stubbed; vtable slot +0x78 (7 args) stubbed; the word at `p1+0` points at 24 random bytes less `0` (placed); the word at `p3+0` points at 24 random bytes less `0` (placed); the word at `p5+0` points at 24 random bytes less `0` (placed); the word at `p7+0` points at 24 random bytes less `0` (placed); the word at `p9+0` points at 24 random bytes less `0` (placed); the word at `p11+0` points at 24 random bytes less `0` (placed); the word at `p13+0` points at 24 random bytes less `0` (placed); the word at `p15+0` points at 24 random bytes less `0` (placed); stub 0 answered from [0, 0, 0, 1] (90%); stub 1 answered from [0, 1, 2, 5, 100, -7, 3, 250] (90%); stub 0 answers 6 values in call order, printed after the answers; stub 1 answers 6 values in call order, printed after the answers; 8 rows 0x6eec apart at `0xc061e0`, slices +0x468:0x18, +0x6eb8:0x4 (`p` windows after the pointed buffers, before the places); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("type_data_can_pay_cost") else {
        return;
    };
    let obj = super::Obj {
        name: "can_pay_cost",
        full: "TypeData::can_pay_cost@00667570",
        cells: 1,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            use crate::economy::{Ledger, RESOURCES};
            // `who` is `a[0]`, `a[3]` says the escrow is ignored; leader
            // `k`'s escrow is `p[2k]` (`+0x468`) and its bucket `p[16 + k]`,
            // stored XOR `0x8221`, through `+0x6eb8`. `type_avail` answers
            // `a[6..12]` a good, the cost slot `+0x78` `a[12..]` a call —
            // asked only for an available good.
            let who = a[0] as usize;
            let mut ledger = Ledger::default();
            let mut available = [false; RESOURCES];
            let mut charges = [0; RESOURCES];
            let mut asked = 12;
            for i in 0..RESOURCES {
                ledger.bucket[i] = rd(&p[16 + who], 4 * i) ^ 0x8221;
                ledger.escrow[i] = rd(&p[2 * who], 4 * i);
                available[i] = a[6 + i] != 0;
                if available[i] {
                    charges[i] = a[asked];
                    asked += 1;
                }
            }
            i64::from(crate::cost::affordable(
                &charges,
                &ledger,
                &available,
                a[3] != 0,
            ))
        }
    });
}
