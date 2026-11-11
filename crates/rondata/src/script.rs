//! **The minted script** (item 4059, `docs/RECGAME.md` §9.1): three of a
//! person's orders in the wire's typed vocabulary — a Scout moved, a
//! Citizen trained, a Farm placed — on the harness lobby's built start.
//! The diff suite mints them into the file the original played
//! (`diff::minted`); the client's input layer is counted against them
//! (item 4145, `docs/CLIENT_INPUT.md`). Out of the test tree so both can name it.

use crate::commands::Command;
use crate::recgame::Order;
use sim::Pos;

/// The harness lobby's long capture, whose `GAMEINFO` block is the lobby's
/// choices (`docs/START.md` §7.4): the script's objects are its built
/// start's.
pub const LOBBY_DUMP: &str = "gamelog-run53-greatlakes-24k-trace.txt";

/// The scout's destination: 2,400 units due east of where it starts
/// (`0/0` at (3480, 31896)), open grass the whole way on our map.
pub const SCOUT_TO: Pos = Pos { x: 5880, y: 31896 };
/// The Farm's drop point: the clear Farm site nearest the capital on our
/// placement check, on a 192-unit grid round it; `sim::Sim::snap_center`
/// puts the site at (3456, 29760).
pub const FARM_AT: Pos = Pos { x: 3552, y: 29856 };
/// The Farm's `TypeIndex`, record 3 past [`crate::load::BASE_BUILDTYPES`].
/// (`crate::artdata::FARM_TYPE`, `0x1a1`.)
pub use crate::artdata::FARM_TYPE;
/// The Citizen's `TypeIndex`.
pub const CITIZEN: i32 = 50;
/// The objects the script names, all slot 0's: the scout, the capital and
/// the citizen who builds (a woodcutter, the one nearest the site).
pub const SCOUT_OBJECT: i16 = 0;
pub const CAPITAL: i16 = 2000;
pub const BUILDER: i16 = 1;
/// The frames the three packages are consumed on, in recording frames:
/// each is processed by the tick that produces `FRAME f + 1`
/// (`crate::input`'s frame convention).
pub const MOVE_FRAME: i32 = 30;
pub const QUEUE_FRAME: i32 = 60;
pub const BUILD_FRAME: i32 = 90;

/// The script: three packages, each a `group` naming the objects and the
/// one command that acts on that selection, as a person's clicks make them
/// (`Options::picked_spot`'s plain drop: `queued` 2, `QUEUE_NEW`; the
/// right-click's `orders` 1, `MOVE_TO`, and `form`/`width` −1, as every
/// `move_to` of the 2017 sample has them).
pub fn script() -> Vec<Order> {
    vec![
        (
            MOVE_FRAME,
            0,
            vec![
                Command::Group {
                    who: 0,
                    objects: vec![SCOUT_OBJECT],
                },
                Command::MoveTo {
                    to_x: SCOUT_TO.x,
                    to_y: SCOUT_TO.y,
                    set_angle: 0,
                    angle: 0,
                    orders: 1,
                    queued: 2,
                    form: -1,
                    width: -1,
                    disembark: 0,
                },
            ],
        ),
        (
            QUEUE_FRAME,
            0,
            vec![
                Command::Group {
                    who: 0,
                    objects: vec![CAPITAL],
                },
                Command::QueueUp {
                    unit: CITIZEN,
                    num: 1,
                },
            ],
        ),
        (
            BUILD_FRAME,
            0,
            vec![
                Command::Group {
                    who: 0,
                    objects: vec![BUILDER],
                },
                Command::Build {
                    x: FARM_AT.x,
                    y: FARM_AT.y,
                    x2: FARM_AT.x,
                    y2: FARM_AT.y,
                    build: FARM_TYPE,
                    queued: 2,
                },
            ],
        ),
    ]
}
