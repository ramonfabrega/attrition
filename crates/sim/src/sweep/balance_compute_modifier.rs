//! `Balance::compute_modifier@00581cc0` under the emulator against the port (item 3372;
//! tools/emu/sweep_balance_compute_modifier.py, written by hand beside tools/sweep_gen.py: the function
//! wants TLS statics, a struct-returning stub and a 318 KB table the generator cannot print sparsely).
//!
//! A row is `ta tb | (mode id word)x4 | td | pa pb | R n e.. C n e.. | idx:val,.. -> eax | R n e.. C n e..`:
//! the four type records `types.list` points at (the image's own `TypeData::is_gaia_type` for mode 0, a
//! stub answering `word != 0` for mode 1), the answer of the stubbed `type_damage`, the two five-word packs
//! `return_pack` is stubbed to write, the function-local `rows`/`cols` statics before, the filled cells of the
//! 399-wide `short` table (the rest 0), and the statics after. The port is [`compute_modifier_with`] and
//! [`pack_categories`]; the arrays are held to the categories it walks.

use sim::balance::{Pack, compute_modifier_with, pack_categories};
use std::collections::HashMap;

/// `ta tb | records | td | packs | before | cells` parsed.
struct Row {
    ta: usize,
    tb: usize,
    recs: Vec<(i32, i32, i32)>,
    td: i32,
    packs: [Pack; 2],
    before: (Vec<i32>, Vec<i32>),
    cells: HashMap<usize, i32>,
}

fn ints(s: &str) -> Vec<i64> {
    s.split_whitespace()
        .map(|t| t.parse().expect("an integer"))
        .collect()
}

/// `R n e.. C n e..` as the two arrays.
fn arrays(s: &str) -> (Vec<i32>, Vec<i32>) {
    let t: Vec<&str> = s.split_whitespace().collect();
    assert_eq!(t[0], "R", "{s}");
    let n: usize = t[1].parse().expect("count");
    let rows = t[2..2 + n]
        .iter()
        .map(|v| v.parse().expect("int"))
        .collect();
    assert_eq!(t[2 + n], "C", "{s}");
    let m: usize = t[3 + n].parse().expect("count");
    assert_eq!(t.len(), 4 + n + m, "{s}");
    let cols = t[4 + n..].iter().map(|v| v.parse().expect("int")).collect();
    (rows, cols)
}

fn parse(lhs: &str) -> Row {
    let f: Vec<&str> = lhs.split(" | ").collect();
    assert_eq!(f.len(), 6, "{lhs:.200}");
    let t = ints(f[0]);
    let r = ints(f[1]);
    let p = ints(f[3]);
    let pack = |w: &[i64]| Pack {
        ty: w[0] as i32,
        line: w[1] as i32,
        obj: w[2] as i32,
        age: w[3] as i32,
        flags: w[4] as u32,
    };
    let cells = if f[5].trim() == "-" {
        HashMap::new()
    } else {
        f[5].trim()
            .split(',')
            .map(|c| {
                let (i, v) = c.split_once(':').expect("idx:val");
                (i.parse().expect("idx"), v.parse().expect("val"))
            })
            .collect()
    };
    Row {
        ta: t[0] as usize,
        tb: t[1] as usize,
        recs: r
            .chunks(3)
            .map(|c| (c[0] as i32, c[1] as i32, c[2] as i32))
            .collect(),
        td: ints(f[2])[0] as i32,
        packs: [pack(&p[..5]), pack(&p[5..])],
        before: arrays(f[4]),
        cells,
    }
}

/// `TypeData::is_gaia_type` as the row's record answers it: mode 0 is the image's own function (the record's
/// `+4` in `0x192..0x19e`), mode 1 a stub answering the record's `+8` word.
fn gaia(rec: (i32, i32, i32)) -> bool {
    match rec.0 {
        0 => (0x192..0x19e).contains(&rec.1),
        _ => rec.2 != 0,
    }
}

#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_balance_compute_modifier.py") else {
        return;
    };
    let (mut n, mut early) = (0, 0);
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (lhs, rhs) = line
            .split_once(" -> ")
            .expect("a row is `<inputs> -> <eax> | <statics>`");
        let (eax, after) = rhs.split_once(" | ").expect("`<eax> | <after>`");
        let row = parse(lhs);
        let want: i32 = eax.trim().parse().expect("an integer");
        let early_out = gaia(row.recs[row.ta]) || gaia(row.recs[row.tb]);
        let cells = &row.cells;
        // `short *` table, 399 wide: the row's filled cells, 0 elsewhere.
        let xml = |r: usize, c: usize| cells.get(&(r * 399 + c)).copied().unwrap_or(0);
        let got = compute_modifier_with(early_out, &row.packs[0], &row.packs[1], || row.td, xml);
        assert_eq!(got, want, "compute_modifier — eax, row `{line:.200}`");
        // The statics: a gaia gate leaves them as they were; otherwise the rows are the first pack's
        // categories, the columns the second's, in the order walked.
        let (ra, ca) = arrays(after);
        let expect = if early_out {
            early += 1;
            row.before.clone()
        } else {
            let cat = |p: &Pack| pack_categories(p).into_iter().map(|v| v as i32).collect();
            (cat(&row.packs[0]), cat(&row.packs[1]))
        };
        assert_eq!(
            (ra, ca),
            expect,
            "compute_modifier — statics, row `{line:.200}`"
        );
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    assert!(early > 20 && early < n - 100, "{early} of {n} rows gated");
    eprintln!("Balance::compute_modifier@00581cc0: {n} rows agree ({early} gaia-gated)");
}
