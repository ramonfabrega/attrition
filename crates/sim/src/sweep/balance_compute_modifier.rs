//! `Balance::compute_modifier@00581cc0` under the emulator against the port (item 3377; tools/sweep_gen.py, obj with windows).

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

/// `Balance::compute_modifier@00581cc0`, run by `tools/emu/sweep_balance_compute_modifier.py` under unicorn
/// (tools/emu): 16 random bytes of object, 3 argument(s), `p` windows in order: p0..p5 the place(s), in flag order; the global cell 0x00e85ddc points at 16 random bytes; 24 random bytes at 0x00ee1418; 24 random bytes at 0x00ee1438; vtable slot +0x18 (0 args) stubbed; callee 0x005821c0 (2 args) stubbed; callee 0x0057fb50 (2 args) stubbed; the word at `c0+0` points at 16 random bytes less `0` (placed); the word at `c0+4` points at 16 random bytes less `0` (placed); the word at `c0+8` points at 16 random bytes less `0` (placed); the word at `c0+0xc` points at 16 random bytes less `0` (placed); the word at `c1+0x10` points at 256 random bytes less `0` (placed); the word at `c2+0x10` points at 256 random bytes less `0` (placed); slot +0x18 laid as 0x004707a0 or its stub, a flag after the answers; stub 0 answered from [0, 0, 0, 0, 1] (90%); stub 2 answered from [100, 0, 1, -100, 250, 1075, 100, 100] (90%); `fs:[0x2c]` -> a TLS array -> one block whose word +0x8 is 0 (a `_TSS` guard read as initialised); stub 1 returns a 5-word struct through its hidden argument 0 and answers it, 2 call(s) drawn a row (the last repeated), the words printed last on the row; argument 2 is a 399 x 399 table of 2-byte cells, zeroed, each cell the function reads filled from ([100, 100, 100, 100, 100, 100, 100, 100, 90, 110, 120, 80, 95, 105, 150, 50, -20, 300, 99, 101, 200, 32767, -32768],) when first read and printed `idx:val` after ` @ ` (`t`, every other cell 0); a write to it fails the row; the stubbed vtable laid at `p0+0` as well as at `this`; the stubbed vtable laid at `p1+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 600 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_balance_compute_modifier.py") else {
        return;
    };
    let mut n = 0;
    // hand: the gaia-gated rows
    let mut gated = 0;
    // end hand
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (line, tcells) = line.split_once(" @ ").expect("`<row> @ <table cells>`");
        let t: std::collections::BTreeMap<usize, i64> = if tcells.trim() == "-" {
            std::collections::BTreeMap::new()
        } else {
            tcells
                .trim()
                .split(',')
                .map(|c| {
                    let (i, v) = c.split_once(':').expect("idx:val");
                    (i.parse().expect("idx"), v.parse().expect("val"))
                })
                .collect()
        };
        let _ = &t;
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
        let mut p: Vec<Vec<u8>> = wins.split_off(3);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            use sim::balance::{Pack, compute_modifier_with, pack_categories};
            // `a`: ta tb table | the stubs' answers (is_gaia, return_pack, type_damage) | the slotbase flag |
            // the two five-word packs `return_pack` writes, in call order.
            let pack = |w: &[i32]| Pack {
                ty: w[0],
                line: w[1],
                obj: w[2],
                age: w[3],
                flags: w[4] as u32,
            };
            let (pa, pb) = (pack(&a[7..12]), pack(&a[12..17]));
            // Slot +0x18 the image's `TypeData::is_gaia_type` (the flag a[6]) reads the record's `+4`;
            // the stub answers a[3].
            let gaia = |k: i32| {
                if a[6] != 0 {
                    (0x192..0x19e).contains(&rd(&p[k as usize], 4))
                } else {
                    a[3] != 0
                }
            };
            let early = gaia(a[0]) || gaia(a[1]);
            gated += usize::from(early);
            let got = compute_modifier_with(
                early,
                &pa,
                &pb,
                || a[5],
                |r, col| t.get(&(r * 399 + col)).copied().unwrap_or(0) as i32,
            );
            // The statics `rows` (c1, buffer p4) and `cols` (c2, buffer p5): the categories walked.
            if !early {
                for (k, pk) in [(1usize, &pa), (2, &pb)] {
                    let cats = pack_categories(pk);
                    wr(&mut c[k], 4, cats.len() as i32);
                    for (i, v) in cats.iter().enumerate() {
                        wr(&mut p[3 + k], 4 * i, *v as i32);
                    }
                }
            }
            i64::from(got)
        };
        assert_eq!(got, want_eax, "compute_modifier — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "compute_modifier — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "compute_modifier — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[4 + i],
                "compute_modifier — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 600, "only {n} rows");
    // hand: both arms reached (the hand sweep of item 3372 held this)
    assert!(gated > 20 && gated < n - 100, "{gated} of {n} rows gated");
    // end hand
    eprintln!("Balance::compute_modifier@00581cc0: {n} rows agree");
}
