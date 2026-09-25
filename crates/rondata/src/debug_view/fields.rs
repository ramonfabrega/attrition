//! Presentation of the comparator's typed disagreements, never a second diff.
use crate::diff::{FrameResult, OrderMismatch};

#[derive(Debug, PartialEq, Eq)]
pub(super) struct Difference {
    pub entity: String,
    pub field: String,
    pub original: String,
    pub rust: String,
    pub assessment: &'static str,
    pub weight: usize,
}
impl Difference {
    pub fn json(&self) -> String {
        let q = super::quoted;
        format!(
            "{{\"entity\":{},\"field\":{},\"original\":{},\"rust\":{},\"assessment\":{},\"weight\":{}}}",
            q(&self.entity),
            q(&self.field),
            q(&self.original),
            q(&self.rust),
            q(self.assessment),
            self.weight
        )
    }
}

pub(super) fn differences(r: &FrameResult) -> Vec<Difference> {
    let mut rows = Vec::new();
    let mut add =
        |entity: String, field: String, original: String, rust: String, assessment, weight| {
            rows.push(Difference {
                entity,
                field,
                original,
                rust,
                assessment,
                weight,
            });
        };
    for d in &r.diverged {
        add(
            format!("unit:{}/{}", d.who, d.o),
            "position (x, y)".into(),
            format!("({}, {})", d.theirs.x, d.theirs.y),
            format!("({}, {})", d.ours.x, d.ours.y),
            "position comparator",
            1,
        );
    }
    for &(who, o) in &r.unlinked_units {
        add(
            format!("unit:{who}/{o}"),
            "presence".into(),
            "present".into(),
            "not linked".into(),
            "unit coverage",
            1,
        );
    }
    // Older/synthetic reports may carry only the aggregate count.
    if r.unlinked > r.unlinked_units.len() {
        add(
            "frame".into(),
            "unidentified unlinked units".into(),
            (r.unlinked - r.unlinked_units.len()).to_string(),
            "not linked".into(),
            "unit coverage",
            r.unlinked - r.unlinked_units.len(),
        );
    }
    for &(who, o) in &r.extra_units {
        add(
            format!("unit:{who}/{o}"),
            "presence".into(),
            "not logged".into(),
            "present".into(),
            "unit coverage",
            1,
        );
    }
    for d in &r.angle_diverged {
        add(
            format!("unit:{}/{}", d.who, d.o),
            format!("angle.{:?}", d.which),
            d.theirs.to_string(),
            d.ours.to_string(),
            "angle comparator",
            1,
        );
    }
    for d in &r.los_diverged {
        add(
            format!("unit:{}/{}", d.who, d.o),
            "mylos".into(),
            d.theirs.to_string(),
            d.ours.to_string(),
            "sight comparator",
            1,
        );
    }
    for d in &r.packed_diverged {
        add(
            format!("unit:{}/{}", d.who, d.o),
            "packed".into(),
            (!d.ours).to_string(),
            d.ours.to_string(),
            "packed comparator",
            1,
        );
    }
    for d in &r.collide_diverged {
        add(
            format!("unit:{}/{}", d.who, d.o),
            d.field.into(),
            d.theirs.to_string(),
            d.ours.to_string(),
            "collision comparator",
            1,
        );
    }
    for d in &r.build_diverged {
        add(
            format!("building:{}/{}", d.who, d.o),
            d.field.into(),
            d.theirs.to_string(),
            d.ours.to_string(),
            "building comparator",
            1,
        );
    }
    for d in &r.gather_diverged {
        add(
            format!("building:{}/{}", d.who, d.o),
            format!("{} [entry {}]", d.field, d.at),
            d.theirs.to_string(),
            d.ours.to_string(),
            "gather comparator",
            1,
        );
    }
    for d in &r.queue_diverged {
        add(
            format!("building:{}/{}", d.who, d.o),
            d.field.clone(),
            d.theirs.to_string(),
            d.ours.to_string(),
            "production comparator",
            1,
        );
    }
    for d in &r.city_diverged {
        add(
            format!("city:{}/{}", d.who, d.o),
            d.field.clone(),
            d.theirs.to_string(),
            d.ours.to_string(),
            "city comparator",
            1,
        );
    }
    for (label, count) in [
        ("unlinked buildings", r.build_unlinked),
        ("unlinked cities", r.city_unlinked),
    ] {
        if count > 0 {
            add(
                "frame".into(),
                label.into(),
                count.to_string(),
                "not linked (identities unavailable in report)".into(),
                "coverage count",
                count,
            );
        }
    }
    for d in &r.order_diverged {
        let (field, original, rust) = order_values(d.what);
        let assessment = match d.what {
            OrderMismatch::Header { .. } => "source header inconsistency; order score",
            OrderMismatch::Unspellable { .. } => "unmodelled kind; order score",
            _ if d.what.scores() => "order score",
            _ => "reported only; excluded from order score",
        };
        let field = if d.what.is_path() {
            field
        } else {
            format!("order[{}].{field}", d.slot)
        };
        add(
            format!("unit:{}/{}", d.who, d.o),
            field,
            original,
            rust,
            assessment,
            1,
        );
    }
    rows
}

fn order_values(d: OrderMismatch) -> (String, String, String) {
    use OrderMismatch::*;
    // Exhaustive matching keeps newly added order variants from silently disappearing.
    match d {
        Length { ours, theirs } => ("length".into(), theirs.to_string(), ours.to_string()),
        Kind { ours, theirs } => ("kind".into(), theirs.to_string(), ours.to_string()),
        Unspellable { theirs } => ("kind".into(), theirs.to_string(), "not modelled".into()),
        Header { named, theirs } => (
            "header/type".into(),
            format!("header {named}; type {theirs}"),
            "not a Rust value comparison".into(),
        ),
        Group {
            field,
            ours,
            theirs,
        }
        | Gather {
            field,
            ours,
            theirs,
        }
        | Move {
            field,
            ours,
            theirs,
        }
        | Guard {
            field,
            ours,
            theirs,
        }
        | Cast {
            field,
            ours,
            theirs,
        }
        | Patrol {
            field,
            ours,
            theirs,
        }
        | Ground {
            field,
            ours,
            theirs,
        } => (
            format!("{}.{field}", d.name()),
            theirs.to_string(),
            ours.to_string(),
        ),
        Garrison { ours, theirs } => (
            "garrison.search".into(),
            theirs.to_string(),
            ours.to_string(),
        ),
        Action { ours, theirs } => ("action".into(), theirs.to_string(), ours.to_string()),
        Target { ours, theirs } => (
            "target (owner, object)".into(),
            format!("{theirs:?}"),
            format!("{ours:?}"),
        ),
        Flags { ours, theirs } => ("flags".into(), theirs.to_string(), ours.to_string()),
        Coll { ours, theirs } => (
            "collision point".into(),
            format!("{theirs:?}"),
            format!("{ours:?}"),
        ),
        PathLength { ours, theirs } => ("path.length".into(), theirs.to_string(), ours.to_string()),
        PathTo { slot, ours, theirs } => (
            format!("path[{slot}].to (bottom first)"),
            format!("{theirs:?}"),
            format!("{ours:?}"),
        ),
        PathField {
            slot,
            field,
            ours,
            theirs,
        } => (
            format!("path[{slot}].{field} (bottom first)"),
            theirs.to_string(),
            ours.to_string(),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diff::*;

    #[test]
    fn order_exclusions_and_source_errors_keep_their_meaning() {
        let report = FrameResult {
            order_diverged: vec![
                OrderDivergence {
                    frame: 9,
                    who: 1,
                    o: 7,
                    slot: 2,
                    what: OrderMismatch::Flags {
                        ours: 4,
                        theirs: 12,
                    },
                },
                OrderDivergence {
                    frame: 9,
                    who: 1,
                    o: 7,
                    slot: 2,
                    what: OrderMismatch::Move {
                        field: "dest_x",
                        ours: 99,
                        theirs: 100,
                    },
                },
                OrderDivergence {
                    frame: 9,
                    who: 1,
                    o: 7,
                    slot: 2,
                    what: OrderMismatch::Header {
                        named: 19,
                        theirs: 1,
                    },
                },
                OrderDivergence {
                    frame: 9,
                    who: 1,
                    o: 7,
                    slot: 0,
                    what: OrderMismatch::PathField {
                        slot: 3,
                        field: "tolerance",
                        ours: 48,
                        theirs: 96,
                    },
                },
            ],
            ..Default::default()
        };
        let rows = differences(&report);
        assert_eq!(
            (rows[0].original.as_str(), rows[0].rust.as_str()),
            ("12", "4")
        );
        assert!(rows[0].assessment.contains("excluded"));
        assert_eq!(rows[1].field, "order[2].move.dest_x");
        assert_eq!(rows[1].assessment, "order score");
        assert_eq!(rows[2].original, "header 19; type 1");
        assert_eq!(rows[2].rust, "not a Rust value comparison");
        assert_eq!(rows[3].field, "path[3].tolerance (bottom first)");
    }

    #[test]
    fn values_and_coverage_are_not_lost_in_presentation() {
        let report = FrameResult {
            unlinked: 3,
            unlinked_units: vec![(1, 8)],
            extra_units: vec![(0, 2)],
            build_unlinked: 4,
            city_unlinked: 2,
            queue_diverged: vec![QueueDivergence {
                frame: 1,
                who: 1,
                o: 2012,
                field: "queue[0].cost[1]".into(),
                ours: i64::MIN,
                theirs: i64::MAX,
            }],
            packed_diverged: vec![PackedDivergence {
                frame: 1,
                who: 0,
                o: 3,
                ours: true,
            }],
            ..Default::default()
        };
        let rows = differences(&report);
        assert_eq!(rows.iter().map(|r| r.weight).sum::<usize>(), 12);
        let queue = rows.iter().find(|r| r.entity == "building:1/2012").unwrap();
        assert_eq!(queue.original, "9223372036854775807");
        assert_eq!(queue.rust, "-9223372036854775808");
        assert!(
            queue
                .json()
                .contains("\"original\":\"9223372036854775807\"")
        );
        let packed = rows.iter().find(|r| r.field == "packed").unwrap();
        assert_eq!(
            (packed.original.as_str(), packed.rust.as_str()),
            ("false", "true")
        );
    }
}
