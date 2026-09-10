//! Opt-in, bounded observations of state overwritten by replay corrections.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CorrectionCounter {
    pub attempted: u64,
    pub changed: u64,
    /// Sim frame, before incrementing to the next tick.
    pub first_changed_frame: Option<i64>,
    /// First changed value pair only; no unbounded per-tick retention.
    pub first_witness: Option<String>,
}

impl CorrectionCounter {
    pub(crate) fn observe(&mut self, frame: i64, changed: bool, witness: impl FnOnce() -> String) {
        self.attempted += 1;
        if changed {
            self.changed += 1;
            if self.first_changed_frame.is_none() {
                self.first_changed_frame = Some(frame);
                self.first_witness = Some(witness());
            }
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CorrectionAudit {
    pub seed: CorrectionCounter,
    /// Complete Gaia Unit equality immediately around reseat_animal, before clocks.
    pub gaia_reseat: CorrectionCounter,
    /// Guy equality immediately around set_guy, including preserved follow state.
    pub clock: CorrectionCounter,
    pub unlinked_units: u64,
    pub predicate_skipped_units: u64,
}

pub(super) fn install_clock(
    sim: &mut sim::Sim,
    audit: &mut Option<CorrectionAudit>,
    frame: i64,
    u: usize,
    n: usize,
    guy: sim::anim::Guy,
) {
    let before = audit.as_ref().map(|_| sim.units[u].guys.get(n).copied());
    sim.set_guy(u, n, guy);
    if let (Some(audit), Some(before)) = (audit, before) {
        let unit = &sim.units[u];
        let after = unit.guys.get(n).copied();
        audit.clock.observe(frame, before != after, || {
            format!(
                "unit {}/{}, guy {n}: {before:?} -> {after:?}",
                unit.owner, unit.index
            )
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn clock_writes_distinguish_insert_noop_and_overwrite() {
        let mut sim = sim::Sim::new(sim::Tuning::RON, sim::World::new(4, 4), 2);
        sim.units
            .push(sim::Unit::new(0, 0, sim::Pos { x: 1, y: 2 }, 100));
        let mut audit = Some(CorrectionAudit::default());
        let mut guy = sim::anim::Guy::fresh(7);
        install_clock(&mut sim, &mut audit, 7, 0, 0, guy);
        install_clock(&mut sim, &mut audit, 8, 0, 0, guy);
        guy.cur_time += 1;
        install_clock(&mut sim, &mut audit, 9, 0, 0, guy);
        let counter = &audit.unwrap().clock;
        assert_eq!((counter.attempted, counter.changed), (3, 2));
        assert_eq!(counter.first_changed_frame, Some(7));
        assert!(
            counter
                .first_witness
                .as_ref()
                .unwrap()
                .contains("None -> Some")
        );
    }
}
