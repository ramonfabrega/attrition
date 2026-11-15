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
/// **The HUD's clock on the photographs** (item 4203, `docs/HUD.md` §5;
/// the client's `hud::tests::the_clock_reads_the_photographs`): of the
/// three photographs of the original (run1559b and run1564 on frame 3279,
/// run1570 on 3327, each frame settled by its movers), those whose clock,
/// read by eye, the bar's `TIME` prints on the frame it shows. 0 of 3
/// before (the bar printed the frame alone); under the victory timers'
/// `(frames + 14) / 15`, 0 of 3.
pub const HUD_CLOCK_PHOTOS: (usize, usize) = (3, 3);
/// **The HUD's clock against the packets** (item 4203; the client's
/// `hud::tests::the_clock_is_the_original_s_tick`): of the packets on
/// disk, those whose `Game::tick` (`+0x560`) is the bar's clock in
/// seconds on the packet's `Game::frame` (`+0x550`), read off each packet
/// by `tools/render/packet_clock.py`.
pub const HUD_CLOCK_PACKETS: (usize, usize) = (109, 109);
/// **The HUD's panel on a wounded unit** (item 4203; the client's
/// `hud::tests::the_panel_s_damage_half_is_the_dump_s`): `(equal,
/// compared)`, each number the panel prints for the attacked script's
/// Scout `1/0` and Slinger `0/7` against run1571's `UNITDATA` on every
/// frame it prints (6690..6899); the Scout's `damage` climbs from 0 to 19
/// there, so the `damage` half of hits left parts if it is wrong.
pub const HUD_WOUNDED: (usize, usize) = (840, 840);

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

/// **The attacked script** (item 4195, `docs/CLIENT_INPUT.md` §8): the
/// boxed script, then a soldier trained and right-pressed on the first
/// enemy our walk shows slot 0. The Barracks' `PREQ0` is The Art of War,
/// so the Library researches it first ([`ATTACK_RESEARCH_FRAME`]); the
/// idle Citizen [`BARRACKS_BUILDER`] drops the Barracks at
/// [`BARRACKS_AT`] once it is available, ten frames after; the finished
/// Barracks [`BARRACKS_OBJECT`] trains the Slingers ten frames after it is done;
/// and on [`ATTACK_FRAME`], the first frame slot 1's Scout `1/0` stands in
/// slot 0's live sight on our walk, the Slinger [`SLINGER`] is sent at it:
/// `Console::execute_at_cursor`'s enemy arm, `attack(ox, whom, ignore 0,
/// QUEUE_NEW)`. The client's `minted_attacked_at_the_opening_camera`
/// mints these from presses, keys and a right press and holds them equal
/// field for field.
pub fn attacked() -> Vec<Order> {
    let mut orders = boxed();
    orders.extend([
        (
            ATTACK_RESEARCH_FRAME,
            0,
            vec![
                Command::Group {
                    who: 0,
                    objects: vec![LIBRARY_OBJECT],
                },
                Command::QueueUp {
                    unit: ART_OF_WAR,
                    num: 1,
                },
            ],
        ),
        (
            BARRACKS_FRAME,
            0,
            vec![
                Command::Group {
                    who: 0,
                    objects: vec![BARRACKS_BUILDER],
                },
                Command::Build {
                    x: BARRACKS_AT.x,
                    y: BARRACKS_AT.y,
                    x2: BARRACKS_AT.x,
                    y2: BARRACKS_AT.y,
                    build: BARRACKS_TYPE,
                    queued: 2,
                },
            ],
        ),
        (
            TRAIN_FRAME,
            0,
            vec![
                Command::Group {
                    who: 0,
                    objects: vec![BARRACKS_OBJECT],
                },
                Command::QueueUp {
                    unit: SLINGERS_TYPE,
                    num: 1,
                },
            ],
        ),
        (
            ATTACK_FRAME,
            0,
            vec![
                Command::Group {
                    who: 0,
                    objects: vec![SLINGER],
                },
                Command::Attack {
                    ox: i32::from(ATTACKED.1),
                    whom: i32::from(ATTACKED.0),
                    ignore: 0,
                    queued: 2,
                },
            ],
        ),
    ]);
    orders
}

/// **The followed script** (item 4202, `docs/CLIENT_INPUT.md` §10): the
/// attacked script, with the Scout [`SCOUT_OBJECT`] pressed twice on slot
/// 1's Scout `1/0`. On [`FOG_FRAME`], the frame before our walk first
/// shows it, `1/0` is under the right press but hidden by the fog:
/// `Console::find_target_daemon` admits no object `is_seen` answers no
/// for, and the press falls to the ground, a `move_to` at [`FOG_TO`] in
/// the Scout's own formation. On [`FOLLOW_FRAME`], the frame after the
/// Slinger's attack, the Scout's press on the now-seen `1/0` (a file holds
/// one package a frame): a selection none of
/// whose members has an attack (`COUNT_ATTACK` 0) draws the dispatch's
/// follow arm, `follow(ox, whom, QUEUE_NEW)`. The client's
/// `minted_followed_at_the_opening_camera` mints these field for field.
pub fn followed() -> Vec<Order> {
    let mut orders = attacked();
    let at = orders
        .iter()
        .position(|o| o.0 == ATTACK_FRAME)
        .expect("the attacked script's attack");
    orders.insert(
        at,
        (
            FOG_FRAME,
            0,
            vec![
                Command::Group {
                    who: 0,
                    objects: vec![SCOUT_OBJECT],
                },
                Command::MoveTo {
                    to_x: FOG_TO.x,
                    to_y: FOG_TO.y,
                    set_angle: 0,
                    angle: 0,
                    orders: 1,
                    queued: 2,
                    form: SCOUT_FORM,
                    width: SCOUT_WIDTH,
                    disembark: 0,
                },
            ],
        ),
    );
    orders.push((
        FOLLOW_FRAME,
        0,
        vec![
            Command::Group {
                who: 0,
                objects: vec![SCOUT_OBJECT],
            },
            Command::Follow {
                ox: i32::from(ATTACKED.1),
                whom: i32::from(ATTACKED.0),
                queued: 2,
            },
        ],
    ));
    orders
}

/// The frame the fog press is consumed on: the last before our walk under
/// [`followed`] shows `1/0` to slot 0, which stands at (6792, 31099).
pub const FOG_FRAME: i32 = 6702;
/// The ground the fog press names: `1/0`'s pixel (1652, 882) at the
/// opening camera on [`FOG_FRAME`], under the layer's inverse.
pub const FOG_TO: Pos = Pos { x: 6792, y: 31100 };
/// The frame the Scout's follow is consumed on, the one after
/// [`ATTACK_FRAME`]: `1/0` is still seen.
pub const FOLLOW_FRAME: i32 = 6704;

/// **A group equal to the last one sent** (item 4267,
/// `docs/CLIENT_INPUT.md` §20): slot 0's `group` of no objects, as
/// `CommandPackage::add_group@0094bb60` writes one whose `who`, objects and
/// uids equal the last group it sent. From [`gathered`] on, each script
/// selects the capital [`CAPITAL`] again with nothing sent between but
/// packages with no group, so its later groups are this.
fn repeated() -> Command {
    Command::Group {
        who: 0,
        objects: Vec::new(),
    }
}

/// **The scripts as the files before item 4267 carried them**
/// (`docs/CLIENT_INPUT.md` §20): each group of no objects replaced by the
/// last group of the same `who` in full, which is the list
/// `CommandPackage::process_group@0094a0c0`'s `num = 0` arm repeats. The
/// original played these files (run1598 to run1606); the receiver reads
/// both forms alike.
pub fn full_groups(orders: &[Order]) -> Vec<Order> {
    let mut last: std::collections::BTreeMap<i8, Vec<i16>> = Default::default();
    orders
        .iter()
        .map(|(f, play, cmds)| {
            let cmds = cmds
                .iter()
                .map(|c| match c {
                    Command::Group { who, objects } if objects.is_empty() => Command::Group {
                        who: *who,
                        objects: last.get(who).cloned().unwrap_or_default(),
                    },
                    Command::Group { who, objects } => {
                        last.insert(*who, objects.clone());
                        c.clone()
                    }
                    _ => c.clone(),
                })
                .collect();
            (*f, *play, cmds)
        })
        .collect()
}

/// **The gathered script** (item 4249, `docs/CLIENT_INPUT.md` §11): the
/// followed script, then the capital [`CAPITAL`] given a rally point and a
/// Citizen. On [`GATHER_FRAME`] a press on the capital and a right press
/// on bare ground: a building that trains (`COUNT_GATHER_POINT`) and no
/// object under the cursor draw `Console::execute_at_cursor`'s ground arm,
/// `gather_point(x, y, action 0, add_to_end 0)` at [`GATHER_AT`]. On
/// [`GATHER_TRAIN_FRAME`] the capital trains a Citizen, who walks to it.
/// The client's `minted_gathered_at_the_opening_camera` mints these field
/// for field. The Citizen's `group` repeats the rally point's, so it is
/// `repeated()` (item 4267).
pub fn gathered() -> Vec<Order> {
    let mut orders = followed();
    let capital = || Command::Group {
        who: 0,
        objects: vec![CAPITAL],
    };
    orders.extend([
        (
            GATHER_FRAME,
            0,
            vec![
                capital(),
                Command::GatherPoint {
                    x: GATHER_AT.x,
                    y: GATHER_AT.y,
                    action: 0,
                    add_to_end: 0,
                },
            ],
        ),
        (
            GATHER_TRAIN_FRAME,
            0,
            vec![
                repeated(),
                Command::QueueUp {
                    unit: CITIZEN,
                    num: 1,
                },
            ],
        ),
    ]);
    orders
}

/// The frame the rally point is consumed on, two after [`FOLLOW_FRAME`].
pub const GATHER_FRAME: i32 = 6706;
/// The ground the rally point names, under the layer's inverse at the
/// opening camera.
pub const GATHER_AT: Pos = Pos { x: 4306, y: 31001 };
/// The frame the capital's Citizen is queued on.
pub const GATHER_TRAIN_FRAME: i32 = 6708;

/// **The chatted script** (item 4250, `docs/CLIENT_INPUT.md` §12): the
/// gathered script and one line more, typed into the entry line and sent
/// on [`CHAT_FRAME`]: `chat(bits, taunt −1, taunt_num 0, CHAT_TEXT)`, the
/// plain box's default arm (`LeaderOut::issue_chat@006d2c50`) in a game of
/// two nations, each seated and live, so `bits` names both, [`CHAT_BITS`].
/// The package carries the chat alone, as every recorded chat's does. The
/// client's `minted_chatted_at_the_opening_camera` mints these field for
/// field.
pub fn chatted() -> Vec<Order> {
    let mut orders = gathered();
    orders.push((
        CHAT_FRAME,
        0,
        vec![Command::Chat {
            bits: CHAT_BITS,
            taunt: -1,
            taunt_num: 0,
            text: CHAT_TEXT.to_string(),
        }],
    ));
    orders
}

/// The frame the chat is consumed on, two after [`GATHER_TRAIN_FRAME`].
pub const CHAT_FRAME: i32 = 6710;
/// The line typed: no digit first (no taunt), no targeting character, not
/// the cheat prefix.
pub const CHAT_TEXT: &str = "good luck";
/// The recipients: slot 0 itself and slot 1.
pub const CHAT_BITS: i32 = 0b11;

/// **The paused script** (item 4252, `docs/CLIENT_INPUT.md` §13): the
/// chatted script and three packages more, all on [`PAUSE_FRAME`], the
/// shape of every recorded pause (`rondata::input::PAUSES_HELD`):
/// `pause(1)`, the pause key's (`TurnControl::issue_toggle_pause@00956b40`
/// with the bit clear); the capital's rally point moved to
/// [`PAUSE_GATHER_AT`] while the game stands paused; and `pause(0)`, the key
/// again. The frame does not move between them. The client's
/// `minted_paused_at_the_opening_camera` mints these field for field. The
/// rally point's `group` is `repeated()` (item 4267).
pub fn paused() -> Vec<Order> {
    let mut orders = chatted();
    orders.extend([
        (PAUSE_FRAME, 0, vec![Command::Pause { state: 1 }]),
        (
            PAUSE_FRAME,
            0,
            vec![
                repeated(),
                Command::GatherPoint {
                    x: PAUSE_GATHER_AT.x,
                    y: PAUSE_GATHER_AT.y,
                    action: 0,
                    add_to_end: 0,
                },
            ],
        ),
        (PAUSE_FRAME, 0, vec![Command::Pause { state: 0 }]),
    ]);
    orders
}

/// **The rallied script** (item 4254, `docs/CLIENT_INPUT.md` §14.1): the
/// paused script without its pause. The chatted script and one package
/// more, on [`PAUSE_FRAME`]: the capital's rally point moved to
/// [`PAUSE_GATHER_AT`], with no `pause` before or after it. Its playback
/// (run1602) separates the moved rally point from the pause's three turns
/// (parked 4252a). Its `group` is `repeated()` (item 4267).
pub fn rallied() -> Vec<Order> {
    let mut orders = chatted();
    orders.push((
        PAUSE_FRAME,
        0,
        vec![
            repeated(),
            Command::GatherPoint {
                x: PAUSE_GATHER_AT.x,
                y: PAUSE_GATHER_AT.y,
                action: 0,
                add_to_end: 0,
            },
        ],
    ));
    orders
}

/// The frame the game is paused on, two after [`CHAT_FRAME`], before the
/// Citizen [`GATHER_TRAIN_FRAME`] queues comes out.
pub const PAUSE_FRAME: i32 = 6712;
/// The rally point given while paused: bare ground the layer's inverse
/// names at the opening camera (stage 0).
pub const PAUSE_GATHER_AT: Pos = Pos { x: 4655, y: 31075 };

/// **The marketed script** (item 4259, `docs/CLIENT_INPUT.md` §16): the
/// paused script and two packages more, each from slot 0's Market
/// [`MARKET_OBJECT`]'s panel: on [`SELL_FRAME`] a hundred of
/// [`SELL_GOOD`] sold, `sell(0, food, 0)`; on [`BUY_FRAME`] a hundred of
/// [`BUY_GOOD`] bought, `buy(0, timber, 0)`. Each package carries its order
/// alone and no `group`, as `CommandManager::issue_sell@00942a80` and
/// `issue_buy` write it and as every recorded sale and purchase carries
/// it. Slot 0 is the Nubians, whose power opens the market without
/// Coinage. The client's `minted_marketed_at_the_opening_camera` mints
/// these field for field.
pub fn marketed() -> Vec<Order> {
    let mut orders = paused();
    orders.extend([
        (
            SELL_FRAME,
            0,
            vec![Command::Sell {
                who: 0,
                good: SELL_GOOD,
                flags: 0,
            }],
        ),
        (
            BUY_FRAME,
            0,
            vec![Command::Buy {
                who: 0,
                good: BUY_GOOD,
                flags: 0,
            }],
        ),
    ]);
    orders
}

/// **The city-gathered script** (item 4261, `docs/CLIENT_INPUT.md` §17):
/// the marketed script and one package more, on [`CITY_GATHER_FRAME`],
/// from the capital's panel: its FARM gather button,
/// `OPTION_CITY_FARM_GATHER`, which `Options::do_city_gather@0071cdb0`
/// issues as `GroupOut::issue_city_gather@0070ac70` writes it, the
/// selection's `group` and `city_gather(t, 2)`, `t` [`CITY_GATHER_TYPE`].
/// Nineteen of the twenty recorded carry the Farm's type and every one
/// `queued` 2. The client's `minted_city_gathered_at_the_opening_camera`
/// mints it field for field. Its `group` is `repeated()` (item 4267).
pub fn city_gathered() -> Vec<Order> {
    let mut orders = marketed();
    orders.push((
        CITY_GATHER_FRAME,
        0,
        vec![
            repeated(),
            Command::CityGather {
                t: CITY_GATHER_TYPE,
                queued: 2,
            },
        ],
    ));
    orders
}

/// **The unqueued script** (item 4265, `docs/CLIENT_INPUT.md` §19): the
/// city-gathered script and one package more. On [`UNQUEUE_FRAME`] the
/// capital, still selected from the city's gather and still training the
/// gathered script's Citizen, has its first queue button pressed:
/// `IFaceSelected::on_button_clicked`'s `OPTION_QUEUE` with slot 0, whose
/// arm calls `BuildOut::issue_unqueue@00629e70(0)` for the capital,
/// `unqueue(0, CAPITAL, 0, uid 0)`, in a package with no group. Twelve of
/// the thirteen recorded cancels carry slot 0 or 2 on a capital whose
/// `uid` is 0. The client's `minted_unqueued_at_the_opening_camera` mints
/// it field for field.
pub fn unqueued() -> Vec<Order> {
    let mut orders = city_gathered();
    orders.push((
        UNQUEUE_FRAME,
        0,
        vec![Command::Unqueue {
            who: 0,
            o: i32::from(CAPITAL),
            unit: UNQUEUE_SLOT,
            uid: 0,
        }],
    ));
    orders
}

/// **The renamed script** (item 4268, `docs/CLIENT_INPUT.md` §21): the
/// unqueued script and one package more. On [`RENAME_FRAME`] the capital,
/// still selected, has its name opened for editing (`Options::exec`'s
/// option `0x117`, popup 6) and [`RENAME_NAME`] typed and sent:
/// `Popup::on_modal_end@007c13f0`'s case 6 calls
/// `CommandManager::issue_rename_city@009411d0(console who, CAPITAL,
/// name)`, `rename_city(0, CAPITAL, name)`, in a package with no group.
/// The client's `minted_renamed_at_the_opening_camera` mints it field for
/// field.
pub fn renamed() -> Vec<Order> {
    let mut orders = unqueued();
    orders.push((
        RENAME_FRAME,
        0,
        vec![Command::RenameCity {
            who: 0,
            o: i32::from(CAPITAL),
            name: RENAME_NAME.to_string(),
            after_nul: Vec::new(),
        }],
    ));
    orders
}

/// **The hot-keyed script** (item 4270, `docs/CLIENT_INPUT.md` §22): the
/// renamed script and, on [`HOTKEY_FRAME`], ctrl and [`HOTKEY_KEY`] with
/// the capital still selected from the gather: the capital's `group`,
/// which repeats the last one sent and so goes as no objects
/// (`CommandPackage::add_group@0094bb60`), and `hotkey(0 × 18 + key, 0,
/// 0, −1.0, −1.0, 0)`, as `CommandManager::issue_hotkey@00941fd0` writes
/// a selection's.
pub fn hotkeyed() -> Vec<Order> {
    let mut orders = renamed();
    orders.push((
        HOTKEY_FRAME,
        0,
        vec![
            Command::Group {
                who: 0,
                objects: Vec::new(),
            },
            Command::Hotkey {
                group: HOTKEY_KEY,
                clear: 0,
                valid: 0,
                x: HOTKEY_NO_CAMERA,
                y: HOTKEY_NO_CAMERA,
                zoom: 0,
            },
        ],
    ));
    orders
}

/// **The garrisoned script** (item 4273, `docs/CLIENT_INPUT.md` §23):
/// the hot-keyed script and, on [`GARRISON_FRAME`], the Citizen
/// [`GARRISON_CITIZEN`] selected by a press, the garrison button
/// (`Options::do_garrison@0071d6b0` sets `selecting_spot`) and a pick on
/// the capital: `Options::picked_spot@00721c40`'s `OPTION_GARRISON` arm,
/// `GroupOut::issue_garrison@0070a9b0(CAPITAL, 0, QUEUE_NEW)` — the
/// Citizen's `group` in full and `garrison(CAPITAL, 0, 2)`, the form all
/// six recorded garrisons carry. The client's
/// `minted_garrisoned_at_the_opening_camera` mints it field for field.
pub fn garrisoned() -> Vec<Order> {
    let mut orders = hotkeyed();
    orders.push((
        GARRISON_FRAME,
        0,
        vec![
            Command::Group {
                who: 0,
                objects: vec![GARRISON_CITIZEN],
            },
            Command::Garrison {
                ox: i32::from(CAPITAL),
                whom: 0,
                queued: 2,
            },
        ],
    ));
    orders
}

/// **The buildmasked script** (item 4276, `docs/CLIENT_INPUT.md` §24):
/// the garrisoned script and two packages more. On
/// [`BUILDMASK_TRAIN_FRAME`] a press on the capital and its Citizen
/// button: the capital's `group` in full and `queue_up(CITIZEN, 1)`, so
/// the queue holds a train job. On [`BUILDMASK_FRAME`] its infinite-queue
/// button: `IFaceSelected::on_button_clicked@00826e00`'s
/// `OPTION_INFINITE`, whose arm in `Options::exec@007188c0` (case `0x40`)
/// calls `GroupOut::issue_buildmask@00708820(0x40)` for a selection of
/// buildings — the `group`, which repeats the last one sent and so goes
/// as no objects (`CommandPackage::add_group@0094bb60`), and
/// `buildmask(0x40, 1)`, the form all five recorded buildmasks carry. The
/// client's `minted_buildmasked_at_the_opening_camera` mints it field for
/// field.
pub fn buildmasked() -> Vec<Order> {
    let mut orders = garrisoned();
    orders.extend([
        (
            BUILDMASK_TRAIN_FRAME,
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
            BUILDMASK_FRAME,
            0,
            vec![
                Command::Group {
                    who: 0,
                    objects: Vec::new(),
                },
                Command::Buildmask {
                    mask: BUILDMASK_INFINITE,
                    set: 1,
                },
            ],
        ),
    ]);
    orders
}

/// **The garrisoned Citizen called out** (item 4280,
/// `docs/CLIENT_INPUT.md` §26): [`buildmasked`], and on
/// [`COME_OUT_FRAME`] a press on the capital and then on the icon of
/// [`GARRISON_CITIZEN`] inside it, which `Options::exec@007188c0` case
/// `0xaa` sends as `UnitOut::issue_come_out@00607be0`: `come_out(0,
/// GARRISON_CITIZEN, COME_OUT_UID)`, with no group.
pub fn came_out() -> Vec<Order> {
    let mut orders = buildmasked();
    orders.push((
        COME_OUT_FRAME,
        0,
        vec![Command::ComeOut {
            who: 0,
            o: i32::from(GARRISON_CITIZEN),
            uid: COME_OUT_UID,
        }],
    ));
    orders
}

/// **The stanced script** (item 4282, `docs/CLIENT_INPUT.md` §27): the
/// came-out script and one package more. On [`STANCE_FRAME`] the capital
/// is still selected (the come-out's press), and a press on the STANCE
/// button mints `Options::do_stance(-1)`'s package: the selection's
/// `group`, a repeat sent as no objects, and `stance(STANCE_CYCLE)`.
pub fn stanced() -> Vec<Order> {
    let mut orders = came_out();
    orders.push((
        STANCE_FRAME,
        0,
        vec![
            Command::Group {
                who: 0,
                objects: vec![],
            },
            Command::Stance {
                stance: STANCE_CYCLE,
            },
        ],
    ));
    orders
}

/// The frame of the stance press, ten after [`COME_OUT_FRAME`].
pub const STANCE_FRAME: i32 = 6790;
/// The cycling press: `Group::action_stance(-1)` steps one option forward
/// from the group's modal one.
pub const STANCE_CYCLE: i32 = -1;

/// The frame of the come-out, after the Citizen is inside the capital
/// (6767, `docs/CLIENT_INPUT.md` §23.4).
pub const COME_OUT_FRAME: i32 = 6780;
/// [`GARRISON_CITIZEN`]'s `uid` (`+0x30`) on [`COME_OUT_FRAME`], read on
/// the walk (the client's `minted_came_out_at_the_opening_camera`).
pub const COME_OUT_UID: i16 = 9;

/// The frame the capital's Citizen is queued on, ten after
/// [`GARRISON_FRAME`].
pub const BUILDMASK_TRAIN_FRAME: i32 = 6760;
/// The frame of the infinite-queue button, two after
/// [`BUILDMASK_TRAIN_FRAME`].
pub const BUILDMASK_FRAME: i32 = 6762;
/// `OPTION_INFINITE`'s mask, `WallData::build_masks`' `0x40`.
pub const BUILDMASK_INFINITE: i32 = 0x40;

/// The frame of the garrison, ten after [`HOTKEY_FRAME`].
pub const GARRISON_FRAME: i32 = 6750;
/// The Citizen sent in: slot 0's `0/2`, standing at the farm `0/2007` on
/// [`GARRISON_FRAME`].
pub const GARRISON_CITIZEN: i16 = 2;

/// The frame of the hot key, ten after [`RENAME_FRAME`].
pub const HOTKEY_FRAME: i32 = 6740;
/// The key: `1`, slot 0's hot-key group 1, empty before it.
pub const HOTKEY_KEY: i32 = 1;
/// −1.0f's bits, the camera fields of a hot key with a selection.
pub const HOTKEY_NO_CAMERA: u32 = 0xbf80_0000;

/// The frame of the rename, ten after [`UNQUEUE_FRAME`].
pub const RENAME_FRAME: i32 = 6730;
/// The capital's new name: nine units, under the box's twenty-one and the
/// command's twenty.
pub const RENAME_NAME: &str = "Attrition";

/// The frame of the cancel, two after [`CITY_GATHER_FRAME`].
pub const UNQUEUE_FRAME: i32 = 6720;
/// The queue slot the cancel names: the entry in training.
pub const UNQUEUE_SLOT: i32 = 0;

/// The frame of the city's gather, two after [`BUY_FRAME`].
pub const CITY_GATHER_FRAME: i32 = 6718;
/// The FARM button's type, `Options::exec@007188c0`'s `do_city_gather(0x1a1)`.
pub const CITY_GATHER_TYPE: i32 = 417;

/// Slot 0's Market, `0/2006` (`TypeIndex` 436), finished before the window.
pub const MARKET_OBJECT: i16 = 2006;
/// The frame of the sale, two after [`PAUSE_FRAME`].
pub const SELL_FRAME: i32 = 6714;
/// The good sold: food.
pub const SELL_GOOD: i32 = 0;
/// The frame of the purchase, two after [`SELL_FRAME`].
pub const BUY_FRAME: i32 = 6716;
/// The good bought: timber.
pub const BUY_GOOD: i32 = 1;

/// slot 0's Library, `0/2005` (`TypeIndex` 435).
pub const LIBRARY_OBJECT: i16 = 2005;
/// The Art of War's `TypeIndex`: `BASE_TECHTYPES` (0x220) plus its record,
/// 28 — the Barracks' `PREQ0`, researched at the Library (`JOB_TIME` 200).
pub const ART_OF_WAR: i32 = 572;
/// The Barracks' `TypeIndex`: `BASE_BUILDTYPES` (0x19e) plus its record,
/// 13 (`sim::ids::ty::BARRACKS`).
pub const BARRACKS_TYPE: i32 = sim::ids::ty::BARRACKS as i32;
/// The Slingers' `TypeIndex` (unit record 32): 40 food and 40 timber on
/// our walk, `moves` 28, a 6-tile reach.
pub const SLINGERS_TYPE: i32 = sim::ids::ty::SLINGERS as i32;
/// The Citizen who builds the Barracks, `0/3`, idle where the box sent it.
pub const BARRACKS_BUILDER: i16 = 3;
/// The Barracks' drop point, as clicked: the pixel (614, 475) at the
/// opening camera, under the layer's inverse — the pixel of (1248, 32160),
/// the clear site on a 192-unit grid nearest the builder on our placement
/// check. `snap_center` puts the site at (1152, 32064).
pub const BARRACKS_AT: Pos = Pos { x: 1249, y: 32163 };
/// The Barracks' site the build places, `0/2008`.
pub const BARRACKS_OBJECT: i16 = 2008;
/// The Slinger the Barracks trains, `0/7`.
pub const SLINGER: i16 = 7;
/// The enemy the Slinger is sent at, `(who, o)`: slot 1's Scout (TI 69,
/// land).
pub const ATTACKED: (u8, i16) = (1, 0);
/// The frames the four packages are consumed on.
pub const ATTACK_RESEARCH_FRAME: i32 = 310;
pub const BARRACKS_FRAME: i32 = 521;
pub const TRAIN_FRAME: i32 = 965;
pub const ATTACK_FRAME: i32 = 6703;

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
