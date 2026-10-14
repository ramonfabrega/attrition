//! `TypeData::can_pay_cost@00667570` under the emulator against the port (item 2247; tools/sweep_gen.py, obj with windows).

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

/// `TypeData::can_pay_cost@00667570`, run by `tools/emu/sweep_type_data_can_pay_cost.py` under unicorn
/// (tools/emu): 32 random bytes of object, 4 argument(s), the global cell 0x00c061e8 points at 2112 random bytes; callee 0x006e33a0 (2 args) stubbed; vtable slot +0x78 (7 args) stubbed; the word at `p1+0` points at 24 random bytes less `0` (placed); the word at `p3+0` points at 24 random bytes less `0` (placed); the word at `p5+0` points at 24 random bytes less `0` (placed); the word at `p7+0` points at 24 random bytes less `0` (placed); the word at `p9+0` points at 24 random bytes less `0` (placed); the word at `p11+0` points at 24 random bytes less `0` (placed); the word at `p13+0` points at 24 random bytes less `0` (placed); the word at `p15+0` points at 24 random bytes less `0` (placed); stub 0 answered from [0, 0, 0, 1] (90%); stub 1 answered from [0, 1, 2, 5, 100, -7, 3, 250] (90%); stub 0 answers 6 values in call order, printed after the answers; stub 1 answers 6 values in call order, printed after the answers; 8 rows 0x6eec apart at `0xc061e0`, slices +0x468:0x18, +0x6eb8:0x4 (`p` windows after the pointed buffers, before the places); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_type_data_can_pay_cost.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(1);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
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
        };
        assert_eq!(got, want_eax, "can_pay_cost — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "can_pay_cost — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "can_pay_cost — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[2 + i],
                "can_pay_cost — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("TypeData::can_pay_cost@00667570: {n} rows agree");
}
