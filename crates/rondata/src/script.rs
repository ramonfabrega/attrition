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
    script_to(SCOUT_TO, FARM_AT)
}

/// **The clicked script** (item 4149, `docs/CLIENT_INPUT.md` §5): the same
/// three orders as a person clicks them at the camera the client opens
/// with. The move's and the Farm's pixels are the ones [`script`]'s targets
/// project to there; the input layer's inverse names the ground under each,
/// one pixel's rounding off the script's point. The client's
/// `minted_clicked_at_the_opening_camera` mints these orders from the
/// clicks and asserts them equal to this, field for field.
pub fn clicked() -> Vec<Order> {
    script_to(CLICKED_SCOUT_TO, CLICKED_FARM_AT)
}

/// **The clicked script, as the main view mints it** (item 4194,
/// `docs/CLIENT_INPUT.md` §9): [`clicked`] with the Scout's move carrying
/// the main view's formation fields in place of the minimap's −1/−1. A
/// right button going down writes `drag_form = max(GroupData::get_form, 0)`
/// and `drag_form_mod = get_form_mod_option` (`Console::down@007cd8f0`),
/// for one unit as for many: the Scout's `UnitData +0xaa` is 0 and its
/// `+0xab` −1 on frame 29 (run1563a), so the move carries [`SCOUT_FORM`]
/// and [`SCOUT_WIDTH`], the leader's byte or 50. [`clicked`]'s −1/−1 is
/// what `WorldMap::on_right_up@008c7050`, the minimap's arm, issues. The
/// client's `minted_clicked_at_the_opening_camera` mints this from the
/// main view's presses and holds it equal field for field.
pub fn clicked_main_view() -> Vec<Order> {
    let mut orders = clicked();
    if let Command::MoveTo { form, width, .. } = &mut orders[0].2[1] {
        (*form, *width) = (SCOUT_FORM, SCOUT_WIDTH);
    }
    orders
}

/// The Scout's move's `form` and `width` as the main view mints them: its
/// own `form` byte, 0, clamped up to 0, and `get_form_mod_option`'s 50 for
/// one member whose `form_mod` is −1.
pub const SCOUT_FORM: i8 = 0;
pub const SCOUT_WIDTH: i8 = 50;

/// The scout's destination as clicked: [`SCOUT_TO`]'s pixel at the opening
/// camera, (1384, 862), under the layer's inverse.
pub const CLICKED_SCOUT_TO: Pos = Pos { x: 5883, y: 31896 };
/// The Farm's drop point as clicked: [`FARM_AT`]'s pixel at the opening
/// camera, (1338, 478), under the layer's inverse; `snap_center` puts the
/// site where it puts [`FARM_AT`]'s, (3456, 29760).
pub const CLICKED_FARM_AT: Pos = Pos { x: 3554, y: 29860 };

/// **The right-pressed script** (item 4179, `docs/CLIENT_INPUT.md` §6): the
/// clicked script and two right presses on the opening frames' own
/// objects, each the command `Console::execute_at_cursor` issues for one
/// selected Citizen with no modifier held. On [`RIGHT_BUILD_FRAME`] the
/// woodcutter [`HELPER`] is sent to the Farm's unfinished site
/// [`FARM_SITE`]: `swarm_around` with `BUILD_AT`. On [`RIGHT_GATHER_FRAME`]
/// the Citizen the capital trained, [`TRAINED`], idle beside it, is sent to
/// the Woodcutter's camp [`WOODCUTTER`], which has room: `gather`. The
/// client's `minted_right_pressed_at_the_opening_camera` mints these from
/// presses and right presses and holds them equal field for field.
/// [`boxed`] extends it (item 4180).
pub fn right_pressed() -> Vec<Order> {
    let mut orders = clicked();
    orders.extend([
        (
            RIGHT_BUILD_FRAME,
            0,
            vec![
                Command::Group {
                    who: 0,
                    objects: vec![HELPER],
                },
                Command::SwarmAround {
                    ox: i32::from(FARM_SITE),
                    whom: 0,
                    queued: 2,
                    orders: i32::from(sim::ids::order::BUILD_AT),
                },
            ],
        ),
        (
            RIGHT_GATHER_FRAME,
            0,
            vec![
                Command::Group {
                    who: 0,
                    objects: vec![TRAINED],
                },
                Command::Gather {
                    ox: i32::from(WOODCUTTER),
                    queued: 2,
                },
            ],
        ),
    ]);
    orders
}

/// The Farm's site the build places, `0/2007`: unfinished from frame 91
/// until our walk finishes it on 297 under one builder.
pub const FARM_SITE: i16 = 2007;
/// The Woodcutter's camp, `0/2001` (`TypeIndex` 418), a gather type.
pub const WOODCUTTER: i16 = 2001;
/// The Citizen at the Woodcutter's camp, `0/2`, sent to help build.
pub const HELPER: i16 = 2;
/// The Citizen the capital trains, `0/6`, idle beside it from about frame
/// 180 until about 300.
pub const TRAINED: i16 = 6;
/// The frames the two right presses' packages are consumed on.
pub const RIGHT_BUILD_FRAME: i32 = 120;
pub const RIGHT_GATHER_FRAME: i32 = 200;

/// **The HUD's buttons** (item 4197, `docs/HUD.md` §5; the client's
/// `hud::tests::the_buttons_mint_the_clicked_file`): of the clicked
/// script's orders a key stands behind (the `c` `queue_up`, the `f`
/// `build`), those minted whole with each key replaced by a press on its
/// button's pixel, and the file written from them the clicked file byte
/// for byte. 0 → 2 of 2 (item 4197): with no button the press landed on
/// the world, and the test still mints that way to see it fail.
pub const HUD_BUTTONS: (usize, usize) = (2, 2);
/// **The HUD's panel** (item 4197, `docs/HUD.md` §5; the client's
/// `hud::tests::the_panel_s_values_are_the_dump_s`): `(equal, compared)`,
/// each number the panel prints for the right-pressed script's selections
/// and the Farm's site against run1568's `UNITDATA` and `BUILDDATA` on the
/// same frame. 0 → 628 of 628 (item 4197), 44 of them the site's
/// `construct_hits` while it is unfinished; read as `myhits − damage`, a
/// site's health parted on those 44 (584 of 628), the misreading
/// `ObjectData::hits_left` settles (`docs/HUD.md` §3).
pub const HUD_PANEL: (usize, usize) = (628, 628);
/// **The HUD's bar** (item 4198, `docs/HUD.md` §5; the client's
/// `hud::tests::the_bar_s_values_are_the_leader_record_s`): `(equal,
/// compared)`, each value the bar prints for slots 0 and 1 against the
/// level-9 `LEADERDATA` record on every frame the nine Great Lakes leader
/// captures print it: six stockpiles, six rates, six cap marks, `control`,
/// `pop_cap`, the age and the nation, 22 a record. 0 of 0 with no bar (no
/// `LeaderScene` behind the door).
pub const HUD_BAR: (usize, usize) = (57_420, 57_420);

/// **The boxed script** (item 4180, `docs/CLIENT_INPUT.md` §7): the
/// right-pressed script and one more package, a box dragged round the three
/// Citizens at the capital's west farms and a right press on the ground
/// south of them. The box takes [`BOXED`] in object order
/// (`Console::on_left_up@007cde90`; one type, so `SelectGroup::add`'s
/// `TypeIndex` order is the same), and the default profile's civilian rule
/// keeps every one, all workers. The right press mints the move with the
/// formation `Console::down@007cd8f0` reads off them: `form`
/// [`BOX_FORM`], the members' shared `UnitData +0xaa`, and `width`
/// [`BOX_WIDTH`], `get_form_mod_option`'s 50 for members whose `+0xab` is
/// all −1 — the bytes run1568's `UNITDATA` prints for `0/3`..`0/5` on 239.
/// The client's `minted_boxed_at_the_opening_camera` mints it from a drag
/// and a right press and holds it equal field for field.
pub fn boxed() -> Vec<Order> {
    let mut orders = right_pressed();
    orders.push((
        BOX_FRAME,
        0,
        vec![
            Command::Group {
                who: 0,
                objects: BOXED.to_vec(),
            },
            Command::MoveTo {
                to_x: BOX_TO.x,
                to_y: BOX_TO.y,
                set_angle: 0,
                angle: 0,
                orders: 1,
                queued: 2,
                form: BOX_FORM,
                width: BOX_WIDTH,
                disembark: 0,
            },
        ],
    ));
    orders
}

/// The three Citizens the box takes, `0/3`, `0/4` and `0/5` (TI 50, land),
/// farming at `0/2002`..`0/2004` west of the capital.
pub const BOXED: [i16; 3] = [3, 4, 5];
/// The frame the box's move is consumed on.
pub const BOX_FRAME: i32 = 240;
/// The ground the right press names: the pixel (700, 600) at the opening
/// camera, under the layer's inverse.
pub const BOX_TO: Pos = Pos { x: 2344, y: 32710 };
/// The move's formation, as the right press reads it off the members.
pub const BOX_FORM: i8 = 9;
pub const BOX_WIDTH: i8 = 50;

/// The script's three orders, the move sent to `scout_to` and the Farm
/// dropped at `farm_at`.
fn script_to(scout_to: Pos, farm_at: Pos) -> Vec<Order> {
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
                    to_x: scout_to.x,
                    to_y: scout_to.y,
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
                    x: farm_at.x,
                    y: farm_at.y,
                    x2: farm_at.x,
                    y2: farm_at.y,
                    build: FARM_TYPE,
                    queued: 2,
                },
            ],
        ),
    ]
}
