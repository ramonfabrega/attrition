//! `BuildQueue::set_job_counter@00630ab0` under the emulator against the port (item 1851).

/// `BuildQueue::set_job_counter@00630ab0`, run by `tools/emu/sweep_build_queue_set_job_counter.py` under unicorn
/// (tools/emu): `__thiscall`, `ecx` = the queue (size at +0, entries behind +4, 20 bytes each), stack `(index, value)` (`ret 8`): the dword at the entry's +0 is set when `index < size` (signed), else nothing. Row `<size> <idx> <v> <k> -> <k=0: the dword read back; k=1: how many of the entries -3..7 changed>`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_build_queue_set_job_counter.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let (size, idx, v) = (a[0], a[1], a[2]);
        let wrote = idx < size;
        let got = match a[3] {
            0 => {
                if wrote {
                    v
                } else {
                    0x0101_0101
                }
            }
            _ => i64::from(wrote && v != 0x0101_0101),
        };
        assert_eq!(got, want, "BuildQueue::set_job_counter — row `{line}`");
        n += 1;
    }
    assert!(n >= 1000, "only {n} rows");
    eprintln!("BuildQueue::set_job_counter@00630ab0: {n} rows agree");
}
