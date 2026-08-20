//! Reads the original's own per-frame state dump, `Logs\gamelog.txt`.
//!
//! With `AllowLogs=1` in `rise.ini` the 2003 engine's `Log` system writes a
//! nested text dump of whichever subsystems `gamelog.ini` enables — once at
//! start (`[Start Game]`, plus the `InitialDump=1` state) and then **every
//! frame** (`[End Frame]`). Each object writes itself through its own
//! `log_data` virtual. `docs/ORACLE.md` has how it is switched on and what it
//! costs; this module is what reads the result back.
//!
//! # The grammar, as observed
//!
//! There is no schema. The file is a flat sequence of lines, each indented by
//! one space per nesting level:
//!
//! ```text
//! BEGIN GAME
//!  BEGIN WORLD
//!   seed 7236
//!   xs 60
//!  BEGIN UNITDATA
//!   BEGIN OBJECT
//!    BEGIN SUBOBJECT
//!     flags 65
//!     o 0
//!   BEGIN GUY
//!    type 69
//! ```
//!
//! A `BEGIN <name>` line opens a block; there is no `END`. A block closes
//! when a later `BEGIN` appears at the same or a shallower indent. Any other
//! line is a field: the first token is the key, the rest is the value, and it
//! belongs to the **innermost open block regardless of its own indent** —
//! because the writers are not consistent about it. `LeaderData::log_data`
//! writes `leader_flags` one level shallower than the `who`/`tribe` lines
//! that precede it inside the same `BEGIN LEADERDATA`, and the block is still
//! the right home for it. Keys repeat: an array constant is written as one
//! line per element, all under the same key (`fort_upgrade_terr[scan] 2`,
//! `… 4`, `… 6`, `… 9`), in index order. Lines before the first `BEGIN` — the
//! `init_teams:` chatter and the splash-screen timing — are preamble and are
//! kept as fields with no block.
//!
//! Everything borrows from the input text: a 114 MB initial dump parses into
//! a tree of slices rather than a tree of copies.
//!
//! # What the dump contains, at detail level 0
//!
//! Established by reading two runs of this install (`docs/ORACLE.md`, last
//! section). Per unit: the `Object` base — `flags`, the object number `o`,
//! the owner `who`, `x_internal`/`y_internal`/`z_internal` in position units
//! — and, **only in the start-of-game dump**, one `GUY` per member with its
//! `type` (the unit type id), position and `angle`; per frame the `GUY`
//! blocks are written empty. Per leader: `who`, `tribe`, `defeated_by`,
//! `gov`, `score`, `leader_flags`, `leader_flags2` — no goods. The richer
//! fields (`UnitData::log_data` goes on to fifty more) wait on a detail-level
//! argument that `DUMP_ALL=1` presumably raises; untried.
//!
//! The `CONSTANTS` block is the loaded `Constants` struct, field by field, in
//! memory representation, under the **lowercased `rules.xml` tag** — 718 of
//! the 753 keys written match a tag that way; the rest are the struct's
//! non-XML members (camera, editor and scenario state). That is what makes it
//! a direct oracle for every representation claim `sim::tuning::Slot` makes.

use std::fmt;

/// One `BEGIN` block: its name, its fields in file order, and its children.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Block<'a> {
    /// The text after `BEGIN `, e.g. `UNITDATA`, `FRAME 100`, `GAME INFO`.
    pub name: &'a str,
    /// Leading spaces on the `BEGIN` line.
    pub indent: usize,
    /// `(key, value)` in file order. The value may be empty and keys repeat.
    pub fields: Vec<(&'a str, &'a str)>,
    pub children: Vec<Block<'a>>,
}

impl<'a> Block<'a> {
    /// The first value under `key`.
    pub fn get(&self, key: &str) -> Option<&'a str> {
        self.fields.iter().find(|(k, _)| *k == key).map(|(_, v)| *v)
    }

    /// The first value under `key`, parsed as an integer.
    pub fn int(&self, key: &str) -> Option<i64> {
        self.get(key)?.trim().parse().ok()
    }

    /// Every value under `key`, in order — the array-constant shape.
    pub fn all(&self, key: &str) -> Vec<&'a str> {
        self.fields
            .iter()
            .filter(|(k, _)| *k == key)
            .map(|(_, v)| *v)
            .collect()
    }

    /// The child blocks whose name is exactly `name`.
    pub fn kids(&self, name: &str) -> impl Iterator<Item = &Block<'a>> {
        self.children.iter().filter(move |b| b.name == name)
    }

    /// The first child block named `name`.
    pub fn kid(&self, name: &str) -> Option<&Block<'a>> {
        self.kids(name).next()
    }

    /// The first child named `name`, searching depth-first through the
    /// whole subtree.
    pub fn find(&self, name: &str) -> Option<&Block<'a>> {
        for c in &self.children {
            if c.name == name {
                return Some(c);
            }
            if let Some(f) = c.find(name) {
                return Some(f);
            }
        }
        None
    }
}

/// A parsed log: the preamble fields and the top-level blocks.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Log<'a> {
    /// Lines before the first `BEGIN`, as `(first token, rest)`.
    pub preamble: Vec<(&'a str, &'a str)>,
    pub roots: Vec<Block<'a>>,
}

impl<'a> Log<'a> {
    /// Parses the whole text. Never fails: there is nothing in the grammar
    /// that can be malformed, only unexpected.
    pub fn parse(text: &'a str) -> Log<'a> {
        let mut log = Log::default();
        // The open-block stack, as paths of child indices from the roots, so
        // the tree can be built in place without parent pointers.
        let mut stack: Vec<(usize, usize)> = Vec::new(); // (indent, index in parent's children)

        fn open_mut<'b, 'a>(log: &'b mut Log<'a>, stack: &[(usize, usize)]) -> &'b mut Block<'a> {
            let (_, first) = stack[0];
            let mut b = &mut log.roots[first];
            for &(_, i) in &stack[1..] {
                b = &mut b.children[i];
            }
            b
        }

        for line in text.lines() {
            let trimmed = line.trim_start_matches(' ');
            if trimmed.is_empty() {
                continue;
            }
            let indent = line.len() - trimmed.len();
            let trimmed = trimmed.trim_end();
            if let Some(name) = trimmed.strip_prefix("BEGIN ") {
                while stack.last().is_some_and(|&(i, _)| i >= indent) {
                    stack.pop();
                }
                let block = Block {
                    name: name.trim(),
                    indent,
                    fields: Vec::new(),
                    children: Vec::new(),
                };
                let idx = if stack.is_empty() {
                    log.roots.push(block);
                    log.roots.len() - 1
                } else {
                    let parent = open_mut(&mut log, &stack);
                    parent.children.push(block);
                    parent.children.len() - 1
                };
                stack.push((indent, idx));
            } else {
                let (key, value) = match trimmed.find(' ') {
                    Some(p) => (&trimmed[..p], trimmed[p + 1..].trim_start()),
                    None => (trimmed, ""),
                };
                if stack.is_empty() {
                    log.preamble.push((key, value));
                } else {
                    open_mut(&mut log, &stack).fields.push((key, value));
                }
            }
        }
        log
    }

    /// The first root named `name`.
    pub fn root(&self, name: &str) -> Option<&Block<'a>> {
        self.roots.iter().find(|b| b.name == name)
    }

    /// The `BEGIN GAME` block, which holds the initial state and the frames.
    pub fn game(&self) -> Option<&Block<'a>> {
        self.root("GAME")
    }

    /// The `BEGIN FRAME n` blocks in order, with their frame numbers.
    pub fn frames(&self) -> Vec<(i64, &Block<'a>)> {
        let Some(game) = self.game() else {
            return Vec::new();
        };
        game.children
            .iter()
            .filter_map(|b| {
                let n = b.name.strip_prefix("FRAME ")?.trim().parse().ok()?;
                Some((n, b))
            })
            .collect()
    }
}

/// A position in the engine's internal units, as the log writes it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Pos {
    pub x: i64,
    pub y: i64,
    pub z: i64,
}

/// One member of a unit, from a `GUY` block.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Guy {
    /// The unit type id. Written only in the start-of-game dump.
    pub kind: Option<i64>,
    pub pos: Option<Pos>,
    pub angle: Option<i64>,
}

/// One unit: the `Object` base plus its members.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UnitDump {
    pub flags: i64,
    /// The object number within its owner's unit array.
    pub o: i64,
    pub who: i64,
    pub pos: Pos,
    pub guys: Vec<Guy>,
}

/// One building: the `Object` base as written under `BUILDDATA`/`WALLDATA`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BuildDump {
    pub flags: i64,
    pub o: i64,
    pub who: i64,
    pub pos: Pos,
}

/// One leader's level-0 fields.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LeaderDump {
    pub who: i64,
    pub tribe: i64,
    pub defeated_by: i64,
    pub gov: i64,
    pub score: i64,
    pub leader_flags: i64,
    pub leader_flags2: i64,
}

/// One city from the `CITIES` list.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CityDump {
    pub x: i64,
    pub y: i64,
    pub pop: i64,
    pub who: i64,
}

/// A named constant from the `CONSTANTS` block: scalar or array.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConstantDump<'a> {
    /// The key as written, with any `[scan]`/`[scan2]` suffix removed.
    pub name: &'a str,
    /// The values in order; one for a scalar, several for an array.
    pub values: Vec<i64>,
    /// Whether the key carried an array suffix.
    pub array: bool,
}

/// The state written once, before frame 1.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Initial<'a> {
    /// `GAME INFO` → `GAMEINFO` fields, e.g. `MAP_SIZE`, `(int)seed`.
    pub game_info: Vec<(&'a str, &'a str)>,
    pub players: Vec<Vec<(&'a str, &'a str)>>,
    pub world: Vec<(&'a str, &'a str)>,
    pub cities: Vec<CityDump>,
    pub constants: Vec<ConstantDump<'a>>,
    pub units: Vec<UnitDump>,
    pub builds: Vec<BuildDump>,
    pub leaders: Vec<LeaderDump>,
}

/// One frame's worth of state.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Frame {
    pub n: i64,
    pub units: Vec<UnitDump>,
    pub builds: Vec<BuildDump>,
    pub leaders: Vec<LeaderDump>,
}

fn pos_of(b: &Block<'_>) -> Pos {
    Pos {
        x: b.int("x_internal").unwrap_or(0),
        y: b.int("y_internal").unwrap_or(0),
        z: b.int("z_internal").unwrap_or(0),
    }
}

/// The `OBJECT` → `SUBOBJECT` base of a unit or building block.
fn object_base(b: &Block<'_>) -> Option<(i64, i64, i64, Pos)> {
    let sub = b.find("SUBOBJECT")?;
    Some((
        sub.int("flags")?,
        sub.int("o")?,
        sub.int("who")?,
        pos_of(sub),
    ))
}

fn unit_of(b: &Block<'_>) -> Option<UnitDump> {
    let (flags, o, who, pos) = object_base(b)?;
    let guys = b
        .kids("GUY")
        .map(|g| Guy {
            kind: g.int("type"),
            pos: match (g.int("x"), g.int("y"), g.int("z")) {
                (Some(x), Some(y), Some(z)) => Some(Pos { x, y, z }),
                _ => None,
            },
            angle: g.int("angle"),
        })
        .collect();
    Some(UnitDump {
        flags,
        o,
        who,
        pos,
        guys,
    })
}

fn build_of(b: &Block<'_>) -> Option<BuildDump> {
    let (flags, o, who, pos) = object_base(b)?;
    Some(BuildDump { flags, o, who, pos })
}

fn leader_of(b: &Block<'_>) -> LeaderDump {
    let i = |k| b.int(k).unwrap_or(0);
    LeaderDump {
        who: i("who"),
        tribe: i("tribe"),
        defeated_by: i("defeated_by"),
        gov: i("gov"),
        score: i("score"),
        leader_flags: i("leader_flags"),
        leader_flags2: i("leader_flags2"),
    }
}

fn constants_of<'a>(b: &Block<'a>) -> Vec<ConstantDump<'a>> {
    let mut out: Vec<ConstantDump<'a>> = Vec::new();
    for (key, value) in &b.fields {
        let (name, array) = match key.find('[') {
            Some(p) => (&key[..p], true),
            None => (*key, false),
        };
        let Ok(v) = value.trim().parse::<i64>() else {
            continue;
        };
        match out.last_mut() {
            Some(last) if last.name == name && array => last.values.push(v),
            _ => out.push(ConstantDump {
                name,
                values: vec![v],
                array,
            }),
        }
    }
    out
}

/// Gathers the per-subsystem records under one block — a `GAME` or a `FRAME`.
fn records(b: &Block<'_>) -> (Vec<UnitDump>, Vec<BuildDump>, Vec<LeaderDump>) {
    let units = b.kids("UNITDATA").filter_map(unit_of).collect();
    let builds = b.kids("BUILDDATA").filter_map(build_of).collect();
    let leaders = b.kids("LEADERDATA").map(leader_of).collect();
    (units, builds, leaders)
}

impl<'a> Log<'a> {
    /// The start-of-game state, from `GAME INFO` and the body of `GAME`
    /// before the first frame.
    pub fn initial(&self) -> Option<Initial<'a>> {
        let game = self.game()?;
        let mut init = Initial::default();
        if let Some(gi) = self.root("GAME INFO").and_then(|g| g.kid("GAMEINFO")) {
            init.game_info = gi.fields.clone();
            init.players = gi.kids("PLAYER").map(|p| p.fields.clone()).collect();
        }
        if let Some(w) = game.kid("WORLD") {
            init.world = w.fields.clone();
        }
        if let Some(c) = game.kid("CITIES") {
            init.cities = c
                .kids("CITY")
                .map(|c| CityDump {
                    x: c.int("x").unwrap_or(0),
                    y: c.int("y").unwrap_or(0),
                    pop: c.int("pop").unwrap_or(0),
                    who: c.int("who").unwrap_or(0),
                })
                .collect();
        }
        if let Some(k) = game.kid("CONSTANTS") {
            init.constants = constants_of(k);
        }
        let (units, builds, leaders) = records(game);
        init.units = units;
        init.builds = builds;
        init.leaders = leaders;
        Some(init)
    }

    /// Every frame's typed state, in order.
    pub fn frame_states(&self) -> Vec<Frame> {
        self.frames()
            .into_iter()
            .map(|(n, b)| {
                let (units, builds, leaders) = records(b);
                Frame {
                    n,
                    units,
                    builds,
                    leaders,
                }
            })
            .collect()
    }
}

impl fmt::Display for Pos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Every snippet below is the shape of real lines from this install's
    // logs, trimmed; see the module note for where they come from.
    const SAMPLE: &str = "\
play2, team 0 8
init_teams: on_team 0
BEGIN GAME INFO
 RUN COUNT 0
 Player
 WHO =  0
 BEGIN GAMEINFO
  (int) version 111935788
  MAP_SIZE 2
  (int)seed 12345
  BEGIN PLAYER
   flags 7
   tribe 11
   who 0
   Player
  BEGIN PLAYER
   flags 1
   tribe 2
   who 1
   Emperor Huayna Capac
BEGIN GAME
 BEGIN COMMANDMANAGER
 BEGIN WORLD
  seed 7236
  xs 60
  ys 60
  player_territory_limit 44
 BEGIN CITIES
  length 20
  BEGIN CITY
   x 3168
   y 30816
   pop 1
   who 0
 BEGIN BUILDDATA
  BEGIN WALLDATA
   BEGIN OBJECT
    BEGIN SUBOBJECT
     flags 39
     o 2000
     who 0
     x_internal 3168
     y_internal 30816
     z_internal 536
 BEGIN CONSTANTS
  unit_move_speed 1
  rocky_modifier 170
  fort_upgrade_terr[scan] 2
  fort_upgrade_terr[scan] 4
  fort_upgrade_terr[scan] 6
  fort_upgrade_terr[scan] 9
  units_killed[scan2] 0
  units_killed[scan2] 3
  one_age_down 15
 BEGIN UNITDATA
  BEGIN OBJECT
   BEGIN SUBOBJECT
    flags 65
    o 0
    who 0
    x_internal 4248
    y_internal 32664
    z_internal 528
  BEGIN GUY
   type 69
   x 4248
   y 32664
   z 539
   angle 1431655765
  BEGIN GUY
   type 69
   x 4254
   y 32555
   z 548
   angle 1431655765
 BEGIN LEADERDATA
  who 0
  tribe 11
  defeated_by -1
  gov -1
  score 0
 leader_flags 176160787
 leader_flags2 0
 BEGIN LEADERDATA
  who 1
  tribe 2
  defeated_by -1
  gov -1
  score 0
 leader_flags 33554439
 leader_flags2 0
 BEGIN GUY
  type 69
  x 4248
  y 32664
  z 539
  angle 1431655765
 BEGIN FRAME 1
  BEGIN UNITDATA
   BEGIN OBJECT
    BEGIN SUBOBJECT
     flags 73
     o 0
     who 0
     x_internal 4248
     y_internal 32664
     z_internal 528
   BEGIN GUY
   BEGIN GUY
  BEGIN LEADERDATA
   who 0
   tribe 11
   defeated_by -1
   gov -1
   score 181
  leader_flags 33554451
  leader_flags2 0
 BEGIN FRAME 2
  BEGIN UNITDATA
   BEGIN OBJECT
    BEGIN SUBOBJECT
     flags 73
     o 0
     who 0
     x_internal 4250
     y_internal 32664
     z_internal 528
   BEGIN GUY
   BEGIN GUY
";

    #[test]
    fn preamble_and_roots() {
        let log = Log::parse(SAMPLE);
        assert_eq!(log.preamble[0], ("play2,", "team 0 8"));
        assert_eq!(log.preamble[1], ("init_teams:", "on_team 0"));
        let names: Vec<_> = log.roots.iter().map(|b| b.name).collect();
        assert_eq!(names, vec!["GAME INFO", "GAME"]);
    }

    #[test]
    fn blocks_nest_by_indent_and_close_on_dedent() {
        let log = Log::parse(SAMPLE);
        let game = log.game().unwrap();
        let names: Vec<_> = game.children.iter().map(|b| b.name).collect();
        assert_eq!(
            names,
            vec![
                "COMMANDMANAGER",
                "WORLD",
                "CITIES",
                "BUILDDATA",
                "CONSTANTS",
                "UNITDATA",
                "LEADERDATA",
                "LEADERDATA",
                "GUY",
                "FRAME 1",
                "FRAME 2"
            ]
        );
        // The building's SUBOBJECT is three levels down.
        let sub = game.kid("BUILDDATA").unwrap().find("SUBOBJECT").unwrap();
        assert_eq!(sub.indent, 4);
        assert_eq!(sub.int("o"), Some(2000));
    }

    #[test]
    fn fields_attach_to_the_innermost_open_block_whatever_their_indent() {
        // LeaderData writes leader_flags one level shallower than who/tribe.
        let log = Log::parse(SAMPLE);
        let game = log.game().unwrap();
        let leaders: Vec<_> = game.kids("LEADERDATA").collect();
        assert_eq!(leaders.len(), 2);
        assert_eq!(leaders[0].int("leader_flags"), Some(176160787));
        assert_eq!(leaders[1].int("leader_flags"), Some(33554439));
        assert_eq!(leaders[1].int("leader_flags2"), Some(0));
    }

    #[test]
    fn labels_without_values_are_kept_as_empty_fields() {
        let log = Log::parse(SAMPLE);
        let gi = log.root("GAME INFO").unwrap();
        assert_eq!(gi.get("Player"), Some(""));
        assert_eq!(gi.get("WHO"), Some("=  0"));
        let players: Vec<_> = gi.kid("GAMEINFO").unwrap().kids("PLAYER").collect();
        assert_eq!(players[1].get("Emperor"), Some("Huayna Capac"));
    }

    #[test]
    fn constants_group_array_entries_under_one_name() {
        let log = Log::parse(SAMPLE);
        let init = log.initial().unwrap();
        let k = &init.constants;
        assert_eq!(k[0].name, "unit_move_speed");
        assert_eq!(k[0].values, vec![1]);
        assert!(!k[0].array);
        assert_eq!(k[1].name, "rocky_modifier");
        assert_eq!(k[1].values, vec![170]);
        assert_eq!(k[2].name, "fort_upgrade_terr");
        assert_eq!(k[2].values, vec![2, 4, 6, 9]);
        assert!(k[2].array);
        assert_eq!(k[3].name, "units_killed");
        assert_eq!(k[3].values, vec![0, 3]);
        assert_eq!(k[4].name, "one_age_down");
        assert_eq!(k.len(), 5);
    }

    #[test]
    fn initial_state_is_typed() {
        let log = Log::parse(SAMPLE);
        let init = log.initial().unwrap();
        assert_eq!(
            init.game_info.iter().find(|(k, _)| *k == "(int)seed"),
            Some(&("(int)seed", "12345"))
        );
        assert_eq!(init.players.len(), 2);
        assert_eq!(
            init.world.iter().find(|(k, _)| *k == "xs"),
            Some(&("xs", "60"))
        );
        assert_eq!(
            init.cities,
            vec![CityDump {
                x: 3168,
                y: 30816,
                pop: 1,
                who: 0
            }]
        );
        assert_eq!(init.builds.len(), 1);
        assert_eq!(init.builds[0].o, 2000);
        assert_eq!(init.builds[0].flags, 39);
        assert_eq!(
            init.builds[0].pos,
            Pos {
                x: 3168,
                y: 30816,
                z: 536
            }
        );
        assert_eq!(init.units.len(), 1);
        let u = &init.units[0];
        assert_eq!((u.flags, u.o, u.who), (65, 0, 0));
        assert_eq!(u.guys.len(), 2);
        assert_eq!(u.guys[0].kind, Some(69));
        assert_eq!(
            u.guys[1].pos,
            Some(Pos {
                x: 4254,
                y: 32555,
                z: 548
            })
        );
        assert_eq!(init.leaders.len(), 2);
        assert_eq!(init.leaders[0].tribe, 11);
        assert_eq!(init.leaders[0].leader_flags, 176160787);
    }

    #[test]
    fn frames_are_typed_and_per_frame_guys_are_empty() {
        let log = Log::parse(SAMPLE);
        let frames = log.frame_states();
        assert_eq!(frames.len(), 2);
        assert_eq!(frames[0].n, 1);
        assert_eq!(frames[1].n, 2);
        let u = &frames[0].units[0];
        assert_eq!(u.flags, 73);
        assert_eq!(u.guys.len(), 2);
        assert_eq!(u.guys[0], Guy::default());
        assert_eq!(frames[0].leaders[0].score, 181);
        assert_eq!(frames[0].leaders[0].leader_flags, 33554451);
        assert_eq!(frames[1].units[0].pos.x, 4250);
    }

    #[test]
    fn an_empty_text_is_an_empty_log() {
        let log = Log::parse("");
        assert!(log.roots.is_empty());
        assert!(log.initial().is_none());
        assert!(log.frame_states().is_empty());
    }
}
