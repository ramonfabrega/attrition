//! The scalar grammar shared by Rise of Nations' data tables.
//!
//! Every numeric field in the shipped data is a leading numeric literal
//! followed by free text the engine discards. The trailing text is a
//! designer's note, not data:
//!
//! ```text
//! <UNIT_MOVE_SPEED value="1/192 tile (granularity for unit movement speeds)"/>
//! <ATTRITION value="48 frames -> this is the baseline level for ..."/>
//! ```
//!
//! # Why this is not one parser
//!
//! A `/` does not mean the same thing everywhere, and getting that wrong
//! silently produces plausible nonsense. In `rules.xml`'s constants it is
//! division — `1/192 tile` is one hundred and ninety-second of a tile. In a
//! unit's `COST` or `SUPPORT` it is a *separator between resources* — `75g/40m`
//! is seventy-five gold **and** forty metal, not a ratio.
//!
//! The two are told apart by the resource letter: if the segments carry one,
//! it is a cost. Rather than guess per call site, each shape gets its own
//! parser and `docs/FORMATS.md` records which fields use which.
//!
//! # Evidence
//!
//! The grammar below is not inferred from a schema — the shipped DTDs are
//! stale and type everything as `CDATA`. It comes from classifying all 851
//! values under `<CONSTANTS>` and all 27,645 field values across
//! `unitrules.xml`, `buildingrules.xml`, and `techrules.xml`. Under
//! `<CONSTANTS>`: 560 plain integers, 252 percentages, 36 rationals, 3
//! multipliers, and nothing that fails to start with a number. Across the
//! record tables the only numeric suffixes that occur are the six resource
//! letters, `rng` (on `RANGE`), and `tsx` (on `JOB_EXTRA_TIME`).

use fixed::Fx;

/// One of the game's six resources.
///
/// The letters are the ones used in `COST` and `SUPPORT` fields. The ordering
/// is the ordering of the `entry0`..`entry5` slots in `STARTING_GOODS`, whose
/// trailing commentary names them: food, timber, gold, knowledge, metal, oil.
/// That ordering is the engine's, so it is worth keeping.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Resource {
    Food,
    Timber,
    Gold,
    Knowledge,
    Metal,
    Oil,
}

impl Resource {
    /// The letter used as a numeric suffix in `COST` and `SUPPORT`.
    pub const fn letter(self) -> char {
        match self {
            Resource::Food => 'f',
            Resource::Timber => 't',
            Resource::Gold => 'g',
            Resource::Knowledge => 'k',
            Resource::Metal => 'm',
            Resource::Oil => 'o',
        }
    }

    /// Parses a resource letter. Case-sensitive: the data only ever uses
    /// lowercase here, and accepting uppercase would collide with the flag
    /// letters used by `OBJ_MASK`.
    pub const fn from_letter(c: char) -> Option<Resource> {
        match c {
            'f' => Some(Resource::Food),
            't' => Some(Resource::Timber),
            'g' => Some(Resource::Gold),
            'k' => Some(Resource::Knowledge),
            'm' => Some(Resource::Metal),
            'o' => Some(Resource::Oil),
            _ => None,
        }
    }

    /// All six, in the engine's own order.
    pub const ALL: [Resource; 6] = [
        Resource::Food,
        Resource::Timber,
        Resource::Gold,
        Resource::Knowledge,
        Resource::Metal,
        Resource::Oil,
    ];
}

/// A single numeric value from a data file, in the form the file wrote it.
///
/// The distinction between the variants is deliberately preserved rather than
/// collapsed at parse time. `50%` and `1/2` are the same number but not the
/// same statement, and whether a percentage is a scale factor or an additive
/// bonus is a per-constant question the parser has no business answering.
/// [`Scalar::to_fx`] converts literally; the caller decides what it means.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Scalar {
    /// A bare integer: `48 frames`.
    Int(i32),
    /// A rational: `1/192 tile`. Never normalised — the numerator and
    /// denominator the designer wrote are more informative than their quotient.
    Ratio { num: i32, den: i32 },
    /// A percentage: `50% per level of flank`.
    Percent(i32),
    /// A multiplier: `2x (units turn faster when packed)`.
    Multiplier(i32),
}

impl Scalar {
    /// Parses the leading numeric literal, discarding any trailing commentary.
    ///
    /// Returns `None` if the text does not begin with a number. Across the
    /// whole shipped constant pool that never happens, so a `None` here means
    /// either a field that is not a scalar or a file we have not seen.
    ///
    /// Note that this will read `0-0rng` as `Int(0)`. `RANGE` is a range, not
    /// a scalar; use [`Range::parse`] for it.
    pub fn parse(text: &str) -> Option<Scalar> {
        let s = text.trim_start();
        let (n, rest) = take_int(s)?;
        let mut chars = rest.chars();
        match chars.next() {
            Some('/') => {
                let (den, _) = take_int(chars.as_str())?;
                Some(Scalar::Ratio { num: n, den })
            }
            Some('%') => Some(Scalar::Percent(n)),
            Some('x' | 'X') => Some(Scalar::Multiplier(n)),
            _ => Some(Scalar::Int(n)),
        }
    }

    /// The literal value as fixed point.
    ///
    /// A [`Scalar::Percent`] becomes its face value over one hundred, so `50%`
    /// is `0.5` — *not* `1.5`. Constants that express an additive bonus are the
    /// caller's problem, because only the caller knows which those are.
    ///
    /// A [`Scalar::Ratio`] goes through [`Fx::ratio`], which is exact for the
    /// denominators the game uses. This is the whole reason the sim can stay
    /// off floating point: the source data is rational, so it converts without
    /// ever passing through a float.
    pub const fn to_fx(self) -> Fx {
        match self {
            Scalar::Int(n) | Scalar::Multiplier(n) => Fx::from_int(n),
            Scalar::Ratio { num, den } => Fx::ratio(num, den),
            Scalar::Percent(p) => Fx::ratio(p, 100),
        }
    }

    /// This scalar multiplied by `count`, evaluated in one step.
    ///
    /// **Prefer this to `scalar.to_fx() * Fx::from_int(count)`.** They are not
    /// the same number. The game's units are deliberately tiny — a movement
    /// speed is a granularity of `1/192` of a tile, and a unit's `MOVES` is how
    /// many of those it covers per frame — and `Fx` truncates toward zero on
    /// every operation, by design, because one predictable rounding rule beats
    /// two fast ones.
    ///
    /// So truncating the granularity first and scaling afterwards throws away
    /// the remainder before it can be used:
    ///
    /// ```
    /// # use rondata::Scalar;
    /// # use fixed::Fx;
    /// let step = Scalar::parse("1/192 tile").unwrap();
    /// assert_eq!(step.to_fx().raw(), 341);              // 65536/192, truncated
    /// assert_eq!((step.to_fx() * Fx::from_int(25)).raw(), 8525);
    /// assert_eq!(step.scaled_fx(25).raw(), 8533);       // 25*65536/192
    /// ```
    ///
    /// Eight raw units is a fifth of a thousandth of a tile, which sounds like
    /// nothing until a unit has taken ten thousand steps and is a tile and a
    /// half from where the other client thinks it is. Divergence in a lockstep
    /// sim is not proportional to the error; it is a cliff.
    pub const fn scaled_fx(self, count: i32) -> Fx {
        match self {
            Scalar::Int(n) | Scalar::Multiplier(n) => Fx::from_int(n * count),
            Scalar::Ratio { num, den } => Fx::ratio(num * count, den),
            Scalar::Percent(p) => Fx::ratio(p * count, 100),
        }
    }

    /// The value as `String::fraction(s, scale)` loads it: `num * scale / den`.
    ///
    /// This is the engine's own scaling routine and the only faithful way to
    /// check a constant it loads that way, because the fixed-point route is
    /// not exact for every denominator the file uses. `6/5` through Q16.16 is
    /// 78643 raw, and `78643 * 100 / 65536` is 119 — one short of the 120 the
    /// original computes as `6 * 100 / 5`. A fifth is not a dyadic rational,
    /// so no binary fixed-point representation can carry it.
    ///
    /// ```
    /// # use rondata::Scalar;
    /// assert_eq!(Scalar::parse("6/5 base rate").unwrap().fraction(100), 120);
    /// assert_eq!(Scalar::parse("3/2").unwrap().fraction(0x100), 384);
    /// assert_eq!(Scalar::parse("10 resources").unwrap().fraction(0x100), 2560);
    /// ```
    ///
    /// A value written without a slash has an implicit denominator of one,
    /// which is why `PEASANT_RATE`'s plain `10` arrives scaled. See
    /// `docs/DECISIONS.md` entry 14.
    pub const fn fraction(self, scale: i32) -> i32 {
        match self {
            Scalar::Int(n) | Scalar::Multiplier(n) | Scalar::Percent(n) => n * scale,
            Scalar::Ratio { num, den } => {
                if den == 0 {
                    0
                } else {
                    (num * scale) / den
                }
            }
        }
    }

    /// The number as the designer wrote it, before any interpretation.
    ///
    /// `50% reduction` gives 50, not 0. This is the reading the engine's own
    /// code takes for the great majority of its constants: it stores a plain
    /// `int` and the percent sign lives only in the annotation. Use this when
    /// checking our transcription of a constant against the file; use
    /// [`Scalar::to_int`] or [`Scalar::to_fx`] when the value's *meaning* is
    /// what is wanted.
    ///
    /// A rational gives its numerator, which is the honest answer for a field
    /// whose written form the caller has already decided is a plain count.
    pub const fn written_int(self) -> i32 {
        match self {
            Scalar::Int(n) | Scalar::Multiplier(n) | Scalar::Percent(n) => n,
            Scalar::Ratio { num, .. } => num,
        }
    }

    /// The integer value, for fields that are plainly counts — frames, hit
    /// points, population. Rationals and percentages truncate toward zero.
    pub const fn to_int(self) -> i32 {
        match self {
            Scalar::Int(n) | Scalar::Multiplier(n) => n,
            Scalar::Ratio { num, den } => {
                if den == 0 {
                    0
                } else {
                    num / den
                }
            }
            Scalar::Percent(p) => p / 100,
        }
    }
}

/// A price: an amount of each of one or more resources.
///
/// Written as slash-separated `<amount><letter>` pairs, sometimes with a
/// trailing ` support` that is a unit annotation rather than a value — `2f`,
/// `75g/40m`, `1f support`. Amounts are stored ×10 against `UNIT_COST_FACTOR`,
/// `BUILD_COST_FACTOR`, and `TECH_COST_FACTOR`, all of which are 10, so a
/// Citizen's `2f` is twenty food. This type holds the raw stored amount;
/// scaling belongs with the constants that define it.
///
/// The grammar is shared by two columns that mean different things. `COST` is
/// what a thing costs once; `SUPPORT`, despite the word, is not upkeep but the
/// **ramp** — what the price rises by per one you already have. See
/// `docs/COSTS.md`, and [`Cost::support_slots`] for the two-slot rule the
/// engine reads a `SUPPORT` field under.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct Cost(pub Vec<(Resource, i32)>);

impl Cost {
    /// Parses a cost field. An empty or `none` field yields an empty cost.
    ///
    /// Returns `None` only if the text is non-empty and does not parse, which
    /// is worth surfacing rather than silently treating as free.
    pub fn parse(text: &str) -> Option<Cost> {
        let s = text.trim();
        // `support` is a unit annotation on the SUPPORT field, exactly like
        // `rng` on RANGE — every one of the 364 unit records carries it. It
        // is not always preceded by an amount: the thirteen engine-spawned
        // objects at the end of the table (Boadicea and the herd animals)
        // write the bare word, meaning no upkeep at all.
        let s = s.strip_suffix("support").unwrap_or(s).trim_end();
        if s.is_empty() || s.eq_ignore_ascii_case("none") {
            return Some(Cost(Vec::new()));
        }
        let mut out = Vec::new();
        for seg in s.split('/') {
            let seg = seg.trim();
            let (n, rest) = take_int(seg)?;
            let letter = rest.chars().next()?;
            out.push((Resource::from_letter(letter)?, n));
        }
        Some(Cost(out))
    }

    /// The amount of one resource, or zero if this cost does not mention it.
    pub fn of(&self, r: Resource) -> i32 {
        self.0
            .iter()
            .find(|(res, _)| *res == r)
            .map_or(0, |(_, n)| *n)
    }

    /// Whether this cost is free.
    pub fn is_free(&self) -> bool {
        self.0.iter().all(|(_, n)| *n == 0)
    }

    /// The pairs a `SUPPORT` field actually reaches the engine as.
    ///
    /// `ObjectType::load_support` keeps two ordered `(resource, amount)`
    /// slots, not a six-slot array. It walks the written pairs in order, skips
    /// any whose amount is zero without consuming a slot, and stops after the
    /// second. Two consequences the engine really has: anything a designer
    /// wrote past the second pair is silently dropped, and a field naming the
    /// same resource twice fills both slots with it, so that resource ramps
    /// twice. The militia line — Militia, Minuteman, Partisan — writes
    /// `2f/2f support` and is the only place the second happens.
    pub fn support_slots(&self) -> Vec<(Resource, i32)> {
        self.0
            .iter()
            .filter(|(_, n)| *n != 0)
            .take(2)
            .copied()
            .collect()
    }
}

/// A weapon range: a minimum and a maximum, written `min-maxrng`.
///
/// Nearly always `0-N`; a non-zero minimum is a unit that cannot fire at point
/// blank. The `rng` suffix is a unit annotation and carries no information.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Range {
    pub min: i32,
    pub max: i32,
}

impl Range {
    /// Parses a range field.
    pub fn parse(text: &str) -> Option<Range> {
        let s = text.trim();
        let (min, rest) = take_int(s)?;
        let rest = rest.strip_prefix('-')?;
        let (max, _) = take_int(rest)?;
        Some(Range { min, max })
    }

    /// Whether this unit has no attack range at all.
    pub fn is_none(self) -> bool {
        self.max == 0
    }
}

/// Reads a leading optionally-negative integer, returning it and the rest.
fn take_int(s: &str) -> Option<(i32, &str)> {
    let neg = s.starts_with('-');
    let digits = &s[usize::from(neg)..];
    let end = digits
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(digits.len());
    if end == 0 {
        return None;
    }
    let n: i32 = digits[..end].parse().ok()?;
    Some((if neg { -n } else { n }, &digits[end..]))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Every literal in these tests is a real value copied out of the shipped
    // data files, so a failure here means our reading of the format is wrong
    // rather than that a made-up example drifted.

    #[test]
    fn plain_integer_with_commentary() {
        // <ATTRITION value="48 frames -> this is the baseline level ...">
        assert_eq!(
            Scalar::parse("48 frames -> baseline"),
            Some(Scalar::Int(48))
        );
        assert_eq!(Scalar::parse("30"), Some(Scalar::Int(30)));
    }

    #[test]
    fn rationals_keep_their_terms() {
        // <UNIT_MOVE_SPEED value="1/192 tile (granularity ...)">
        assert_eq!(
            Scalar::parse("1/192 tile (granularity for unit movement speeds)"),
            Some(Scalar::Ratio { num: 1, den: 192 })
        );
        // 2/3 is not stored as 0 or as 0.666.
        assert_eq!(
            Scalar::parse("2/3 (light infantry in rocks)"),
            Some(Scalar::Ratio { num: 2, den: 3 })
        );
    }

    #[test]
    fn percent_and_multiplier() {
        // <HEIGHT_BONUS value="10% per increment">
        assert_eq!(
            Scalar::parse("10% per increment"),
            Some(Scalar::Percent(10))
        );
        // <UNIT_PACK_TURN_BONUS value="2x (units turn faster when packed)">
        assert_eq!(
            Scalar::parse("2x (units turn faster when packed)"),
            Some(Scalar::Multiplier(2))
        );
    }

    #[test]
    fn negative_values_parse() {
        assert_eq!(Scalar::parse("-1"), Some(Scalar::Int(-1)));
    }

    #[test]
    fn non_numeric_is_rejected() {
        assert_eq!(Scalar::parse("none"), None);
        assert_eq!(Scalar::parse(""), None);
    }

    #[test]
    fn rationals_reach_fx_without_a_float_existing() {
        // The point of the whole fixed-point constraint: 1/192 of a tile
        // survives the trip without a float existing at any moment.
        let speed = Scalar::parse("1/192 tile").unwrap();
        assert_eq!(speed.to_fx(), Fx::ratio(1, 192));
    }

    #[test]
    fn scaling_a_truncated_unit_is_not_the_same_as_scaling_the_ratio() {
        // Pinned because it is a live desync hazard, not a curiosity. A
        // Citizen's MOVES is 25 and UNIT_MOVE_SPEED is 1/192 of a tile.
        let speed = Scalar::parse("1/192 tile").unwrap();

        // Truncate the granularity, then scale: the remainder is gone.
        let naive = speed.to_fx() * Fx::from_int(25);
        // Scale inside the ratio: the remainder survives to the last step.
        let correct = speed.scaled_fx(25);

        assert_eq!(naive.raw(), 8525);
        assert_eq!(correct.raw(), 8533);
        assert_ne!(naive, correct);
        assert_eq!(correct, Fx::ratio(25, 192));
    }

    #[test]
    fn scaled_fx_agrees_with_to_fx_at_unit_count() {
        for s in [
            Scalar::Int(48),
            Scalar::Ratio { num: 1, den: 192 },
            Scalar::Percent(50),
            Scalar::Multiplier(2),
        ] {
            assert_eq!(s.scaled_fx(1), s.to_fx(), "{s:?}");
        }
    }

    #[test]
    fn percent_converts_to_its_face_value() {
        assert_eq!(Scalar::parse("50%").unwrap().to_fx(), Fx::ratio(50, 100));
    }

    #[test]
    fn single_resource_cost() {
        // Citizen: <COST>2f</COST>
        assert_eq!(Cost::parse("2f"), Some(Cost(vec![(Resource::Food, 2)])));
    }

    #[test]
    fn slash_in_a_cost_means_and_not_divided_by() {
        // This is the distinction that matters. In a cost, 75g/40m is two
        // resources; read as a rational it would be "one point eight".
        let c = Cost::parse("75g/40m").unwrap();
        assert_eq!(c.of(Resource::Gold), 75);
        assert_eq!(c.of(Resource::Metal), 40);
        assert_eq!(c.of(Resource::Food), 0);
    }

    #[test]
    fn trailing_support_is_ignored() {
        // Citizen: <SUPPORT>1f support</SUPPORT>
        assert_eq!(
            Cost::parse("1f support"),
            Some(Cost(vec![(Resource::Food, 1)]))
        );
        assert_eq!(
            Cost::parse("1t/2f support"),
            Some(Cost(vec![(Resource::Timber, 1), (Resource::Food, 2)]))
        );
    }

    #[test]
    fn free_and_absent_costs() {
        assert!(Cost::parse("").unwrap().is_free());
        assert!(Cost::parse("none").unwrap().is_free());
        assert!(Cost::parse("0f").unwrap().is_free());
    }

    #[test]
    fn a_bare_unit_word_means_no_upkeep() {
        // unitrules.xml records 351-363 — Boadicea and the herd animals —
        // carry <SUPPORT>support</SUPPORT> with no amount at all. These are
        // engine-spawned objects that cost the player nothing to keep.
        let c = Cost::parse("support").unwrap();
        assert!(c.is_free());
        assert_eq!(c.0.len(), 0);
    }

    #[test]
    fn bad_resource_letter_is_an_error_not_a_zero() {
        assert_eq!(Cost::parse("5z"), None);
    }

    #[test]
    fn ranges() {
        // Citizen: <RANGE>0-0rng</RANGE>
        assert_eq!(Range::parse("0-0rng"), Some(Range { min: 0, max: 0 }));
        assert!(Range::parse("0-0rng").unwrap().is_none());
        assert_eq!(Range::parse("0-12rng"), Some(Range { min: 0, max: 12 }));
    }

    #[test]
    fn every_resource_letter_round_trips() {
        for r in Resource::ALL {
            assert_eq!(Resource::from_letter(r.letter()), Some(r));
        }
    }
}
