# The HUD, plain

Items 4197 and 4198 (DECISIONS 129 §7), designed by the pass of 2026-11-12
before either was cut. **The menu and the panel are built** (item 4197,
`client::hud`); **the bar is built** (item 4198, `client::hud::Bar` over
`sim::observe::LeaderScene`). This is the document the lanes
implement and amend; a claim below is a rule to hold the build to unless
it says what backs it.

What a person needs on screen to play, in the lobby page's manner
(`docs/LOBBY.md`): text and rectangles in the debug overlay's font
(`client::overlay`). It is **not the original's interface**. Porting the
original's windows is on the cut list (`CLAUDE.md`, "Cut from v1") and
Ramon's to reopen; the photographs show them, and no row here scores
against a photograph's pixels.

Why it is booked: set beside three photographs, the client's still lacks
the interface before it lacks anything else, and two of the orders the
mouse mints hide behind keys a person has to be told (`f` chooses the
Farm, `c` trains the Citizen; `docs/CLIENT_INPUT.md` §1).

## 1. Three parts, two items

| part | what it shows | where its values come from | item |
|---|---|---|---|
| the menu | one button for each thing the selection can order from the keyboard | `client::input`'s `BUILD_KEYS` and `TRAIN_KEYS` | 4197 |
| the panel | the selection: each object's type name, its health over its full health; a building's construction | `sim::observe`'s `UnitScene` and `BuildingScene`, which carry them today | 4197 |
| the bar | the six resources, population over its cap, the age, the nation, the game's clock | `sim::observe`'s `LeaderScene` (item 4198; before it, nothing the door carried) | 4198 |

That split is the gate's. The menu and the panel read what the scene
already holds, so 4197 touches `crates/client` alone and gates scoped. The
bar needs a leader's record through `sim::observe`, a new door, so 4198
gates whole and takes the whole slot's turn (DECISIONS 129 §1).

**The entry line** (item 4250) is a fourth part, drawn only while open:
the chat being typed, after `SAY:`, above the menu's row. Return opens it
and sends it; what it mints is `docs/CLIENT_INPUT.md` §12.

**The pause notice** (item 4252) is a fifth, drawn only while the slot's
pause stands: `PAUSED`, centred one margin below the top (`hud::pause_rect`).
The text and the place are ours; the original's `do_pause_notice` is not
read. What the key mints is `docs/CLIENT_INPUT.md` §13.

**The market's row** (item 4259) is a sixth, drawn only while the slot's
own finished Market is the selection: a sell and a buy button for each
good the original's market panel offers (`BuildOut::make_options@00629ed0`,
food and timber in the Ancient age), each with its price, in a row along
the bottom edge anchored to the right corner (`hud::market_menu`,
`MARKET_BUTTON` 180 × 36). A button the original adds disabled — a sale
under a hundred of the good, a purchase under its price — is drawn grey
and mints nothing. Unlike the menu's buttons it is no key's event: a press
on it is `Event::Sell` or `Event::Buy`, which no key sends. The place, the
size and the labels are ours. What it mints is `docs/CLIENT_INPUT.md` §16.

**The city's row** (item 4261) is a seventh, drawn only while the slot's
own city centre is the selection: the panel's four gather buttons
(`BuildOut::make_options@00629ed0`'s `OPTION_CITY_FARM_GATHER`,
`_WOODCUTTER_`, `_MINE_` and `_ALL_`), GATHER FARM, WOOD, MINE and ALL, in
the market's place (`hud::city_menu`, `MARKET_BUTTON` 180 × 36); the two
rows never show at once. A press on one is `Event::CityGather`, which no
key sends. The place and the labels are ours. What it mints is
`docs/CLIENT_INPUT.md` §17.

**The craft row** (item 4264) is an eighth, drawn while units are the
selection and a member offers a craft (`UnitScene::crafts`): one button per
craft, in row order, named as the crate names it, in the market's place
(`hud::spell_menu`). A press on one is `Event::Spell`, which no key sends;
a targeted craft's next press on the map is its pick. The place and the
labels are ours. What it mints is `docs/CLIENT_INPUT.md` §18.

**The queue row** (item 4265) is a ninth, drawn while one finished building
of the slot's own is the selection and its production queue holds entries
(`BuildingScene::queued`): one CANCEL button per live entry, in queue order,
one row above the bottom edge so a city's row keeps its place
(`hud::queue_menu`). A press on one is `Event::Unqueue` with its slot, which
no key sends. The place and the labels are ours. What it mints is
`docs/CLIENT_INPUT.md` §19.

**The rename button** (item 4268) is a tenth, drawn while the selection's
first object is a finished building of the slot's own in a city
(`BuildingScene::in_city`): one RENAME button, three rows above the bottom
edge at the right (`hud::rename_button`). A press on it is `Event::Rename`,
which opens the entry line as the city's name box; Return sends it. The
place and the label are ours. What it mints is `docs/CLIENT_INPUT.md` §21.

## 2. The menu (item 4197)

- **A button is a key's event and nothing else.** A press inside a
  button's rectangle is handed to the input layer as the key it shows.
  The layer's arms are untouched: with units selected a build key chooses
  the building the next press places; with a building selected a train
  key queues the unit. So a button can mint nothing a key cannot, and the
  order's bytes cannot differ.
- **Which buttons show** follows the selection, as the keys' arms do:
  units selected, one button a `BUILD_KEYS` row; a building selected, one
  a `TRAIN_KEYS` row; nothing selected, none. Each is labelled with its
  key and its type's name from the install's tables.
- **A press on a button is not a press on the world.** The window asks
  the HUD first; only a press it does not take reaches the layer's press
  arm. A press on a button never clears the selection and never places.
- **Where the buttons stand** is the HUD's to choose and is pinned as
  pixels at the script's window size, the way the scripts pin a click.
  Built (4197): one row along the window's bottom edge, each button
  240×36 pixels, 8 from the edge and from the next; at 1920×1080 the
  first button's centre is (128, 1054) (`hud::tests::BUTTONS_AT`). No
  pixel the scripts press or right-press falls under a button.
- **A button's label** is its key and its type's `NAME`, joined through
  the `TypeIndex`: units from `BASE_UNITTYPES` (0x32), buildings from
  `BASE_BUILDTYPES`. Read as a raw index into the unit names, `c` was
  labelled the Riflemen for the Citizen it trains (seen on the still,
  then made a test's red).

The menu offers what the layer can mint: two rows today. Widening
`BUILD_KEYS` and `TRAIN_KEYS` widens it; **what the original's own menu
would offer** a selection at its age and with its research is not read
and not claimed (§6).

## 3. The panel (item 4197)

One line an object of the selection, capped at the lines the panel holds
with a closing count of the rest:

- a unit: its type's name, `health` of `max_health`, the count of its
  figures when more than one;
- a building: its type's name, `health` of `hits`; unfinished, its
  construction's hits of the full as well.

The values are the scene's own fields, printed as integers. Nothing is
scaled, rounded or computed in the client beyond the join to a name.

**A building's health is its hits left, and a site's climbs** (4197).
`ObjectData::hits_left@006535c0` is `hits(0) − damage`, floored at zero
and capped at `hits(0)`; a building's `hits(0)` is `construct_hits`
(`WallData::hits@00642bb0`, `BuildData::hits@0062e740`), `hits(1)` its
`myhits`. So the scene's `health` against `hits` is the original's own
pair, and on a site it reads `construct_hits − damage` of `myhits`: run1568
prints the Farm's site `0/2007` on frame 210 as `myhits 400`, `damage 0`,
`(int)construct_hits 223`, and the panel prints `223/400 BUILDING 223/400`.
A unit's `hits(0)` is its `myhits` (the harness's hit-point rows).
The printed line is `NAME health/full`, then ` BUILDING c/full` while
unfinished and ` FIGURES n` for more than one figure (the Scout prints 2).

**A wounded unit's health is `myhits − damage`** (4203): the `damage`
half is compared on run1571, where the attacked script's Slinger wounds
slot 1's Scout from frame 6805 (§5).

**The ring (4203).** The original marks a selected unit in
`UnitOut::draw_underlays@00614100`: when `GroupData::member` finds it in the
console's selection, `draw_unit_circles@00606a10` draws one circle at each
figure's place, of radius `unit_block_radius × +0x234` (the unit type's
`CIRCLE_RADIUS`, clamped to 10), in the owner's `TeamColor::get_iface`.
`client::hud::rings` draws that circle, 32 points on the ground at the
figure's height, two pixels wide in the owner's unit colour
(`render::silhouette::team_unit`); the Scout's is 48. **A building's is
ours**: its footprint's rectangle about its place. The original's
(`BuildOut::draw_underlays@0062bdf0`) is `draw_funky_rectangle_circle` at
vtable `+0x130`, `BuildData::max_range`, in `TeamColor::get_unit` at alpha
0x40: run1570's photograph shows it as a rounded red outline about
Napata's plaza. Not ported (§6).

**Amended by item 4398: the red line round a selected city's plaza is not
the funky circle; it is `WallOut::draw_underlays@006410a0`'s
`draw_thick_rectangle@008c6180`**, which `BuildOut::draw_underlays` calls
first, for any selected building (`GroupData::member` of the console's
`play` group; `Console::target_o`'s own branch draws it in
`target_color` instead). run1559b's packet holds the selection: group 0's
`num` 1, `buildings` 1, `who` 0, `list[0]` 2000 (the capital), and
`target_o` −1. The rectangle, by the listing at `006411a4`–`0064136d`:
`k` is the drawn piece's `build_half_tcoord`, or the type's `x_size` less
a non-zero `build_construction` (the foundation's `back`); the centre is
the point less `0x60·k` on both axes, and its sides are `x_size − k` and
`y_size − k` tiles (`ECX`, `EDX`). `draw_thick_rectangle` lays one vertex a
tile on each side, from `(−x, −y)` counter-clockwise, each paired with one
`48.0` out along the previous side's direction, with a corner square
closing each side: **a band 48 world units wide outside the rectangle**,
every vertex at `find_data_z`. Its colour is the owner's
`TeamColor::get_unit` with the alpha's top bit cleared unless
`Camera::is_zoom_out`; its texture is `int_str_array +0x1784`'s; a vertex
off the map takes alpha 0. `client::selection::band` is that geometry, and
**`RUN1559B_3279_SELECTED`** reads it on run1559b's photograph, the city's
type 0 (7 × 7, `back` 1, point (3168, 30816)): of the band's 4,551 pixels,
**1,477 are red in theirs** (`r − g` and `r − b` over 40); moved one tile
east 721 of 4,157, with no back 312 of 5,398; theirs read 3 px south, once,
1,016. The rest of the band is behind the buildings, which draw over an
underlay. Ours, with nothing drawn: 52 red, 9 of them on theirs. The
footprint is the geometry's, never a colour (DECISIONS 142 §5).

**Drawn by item 4401.** `client::selection::underlay` lays the band of each
selected building in the owner's `team_unit` colour, and the GPU's
`underlay` pass blends it at alpha `0x7f` (`gpu::UNDERLAY_ALPHA`, the top
bit cleared) after the ground and before the buildings, with no depth test
and no depth written; `hud::rings` no longer outlines a building. On the
same footprint the row's third half reads the still with the city
selected: **ours red 52 → 2,132, of them on theirs' red 9 → 1,404** (of
theirs' 1,477). Drawn one tile east, it read 855 and 542, and the pin
failed. Ours is a solid 48-unit band; theirs is thinner and soft-edged,
its texture's, so ours paints about 728 red pixels theirs does not.

**The city's plate, placed (item 4403).** `BuildOut::draw_overlays@0062cb30`
draws it for any active city centre (`flags & 4` and `& 0x20`), selected or
not, under `player_profile.prefs & 0x40000` and the visibility branch; it
is not the selection's. The anchor, by the listing at `0062cc4a`–`0062ccae`:
`world_to_console(x, y, z − 0x60·(2·y_size/3))` (the object's `+0x10`,
`+0x14`, `+0xc`), the console's y added. About it, the name (`CityData
+0x90`) `text_width` wide and the font's height tall (`x0 = sx − w/2`,
`y0 = sy − h/2`); `adjust_for_scroll@0051cd20` then widens it for a capital
(`CityData +4 & 0x10`: 3 rows each way, the star's width split) and by the
pads; `draw_scroll@0051cdb0` lays the nine-part scroll on it, the star
inside the left pad tinted the owner's `get_unit`, and the name is written
white. **Every quantity is the packet's** (run1559b, frame 3294): pads 8,
4, 8, 4 (`IFaceRenderManager +0xe84`; `setup_text` makes the top
`scroll_tex[0]`'s width, 4, and the sides twice it); the star 22 × 21;
the font `onscreen_mouseover_size` (`FontFamily` lookup 36, size 14), its
`D3D11GLFont` `mFontHeight` 16; the name `Napata`, its `CharacterInfo`
advances 12 + 8 + 9 + 8 + 6 + 8 = 51; the city's flags carry the capital's
bit (`& 0x10`).
`client::selection::plate` is that rectangle: **x [1083, 1172), y [580,
610)**, 89 × 30, which is the plate on the photograph to the pixel (its
gold edge at 1083 and 1171, 580 and 609). **`RUN1559B_3279_PLATE`** reads
it: of the 2,670 pixels, **1,285 are dark in theirs** (every channel under
40, the scroll's black); moved one plate down, 0; without the star, 1,018
of 1,608. Ours, nothing drawn: 0. The anchor moved 89 px east read 75, and
the pin failed. The photograph's `INTERFACE` mask holds this rectangle as
"the city's label", taken out by eye for the scene's rows; the plate's own
row does not apply it.

**Drawn by item 4410.** `client::selection::plate_layers` lays
`draw_scroll` from the install's own texels: `interface.xml`'s nine
`<IFACETEX name="scroll">` on `TEX_CIV` (`iface_AS0_assets.tga`; corners
4 × 4, edges 138 × 4 and 4 × 21, the body 138 × 21, one texel `(1, 1, 1)`
at alpha 224), each tiled from its first texel as `draw_2d_expander@0051c7b0`
lays them; then `SCROLL_CAPITAL_STAR` (`interface.txt`, on `TEX_COMMON`),
white with 64 alphas, times the owner's `team_unit`, at the pads' inner
corner; then the name from the text's rectangle (past the star, 3 rows
down), each character at its advance. **The photograph's opaque edge is
the texture's to the byte**: 578 of 678 texels, the other 100 under the
blue `Reduced!` that floats over the plate's sides. The GPU's screen pass
(`gpu::ScreenLayer`) draws them after the tone map and before the
overlay, a layer an alpha, at the underlay's constant blend. On the
row's footprint: **ours dark 0 → 1,742, on theirs' dark 0 → 1,251** (of
1,285); drawn 89 px east, ours read 0 and the pin failed. The name is a
stand-in, and is drawn only where it is known (§6).

**The bar (item 4410).** `WallOut::draw_health_bar@00640dd0` draws it for a
member of the console's `play` group (or the mouse-over, or `+0x24`), at
`world_to_console` of the point; `@00640f20` lifts the point `0x180`,
takes 30 × 3 at `Camera::zoom_level` 6, 20 × 2 at 5 and 10 × 1 below,
about the anchor (`x0 = sx − w/2`, `y0 = sy − h/2`); its colour is
`ObjectData::health_level@006534b0`'s (0 `GREEN`, 1 to 3 `YELLOW`, past
`RED`), the level of `hits(0) − damage` against 90, 75, 50, 25 and 10 % of
`hits(0)`; and the listing (`00640fb9`–`0064108b`) hands
`GraphicPieces::draw_health_bar@00908650` the fraction `left / full` in
single precision, `full` `hits(1)` at least 1 and `left` `hits(0) −
damage` zeroed below nothing and capped at `full`. That one lays a black
quad one pixel outside the rectangle and the fill from `x0` over
`trunc(w · fraction)`. Run1559b's packet holds `zoom_level` 5 (the
camera's `+0x294`), so the bar is 20 × 2 in a 22 × 4 frame.
`client::selection::bar` is that rectangle and `bars` draws it in the
screen pass. **`RUN1559B_3279_BAR`** reads it on the frame's 88 pixels:
**theirs black 86** (every channel under 12; the other two are the fill,
`(255, 0, 0)`), moved one bar down 0, without the lift 0. Napata stands at
**`health` 102 of `hits` 1,200** in the scene, 8.5 %: level 5, red, and a
fill of 1 pixel, which is the photograph's. **Ours black 0 → 86, all on
theirs, and both red 0 → 2**; the fill drawn green read 0 red, the bar
drawn without the lift 0 black, and each failed the pin.

## 4. The bar (item 4198)

A `LeaderScene` through `sim::observe`, read-only, for the issuing slot:
the six resources, the population and its cap, the age, the tribe. The
bar prints them as integers in the original's own units, with the nation's
name from the tables and the clock as the frame count and as the time the
original's own bar shows (the photographs read `00:03:38` on frame 3279
and `00:03:41` on the frame run1570's shows, 3327; read below).

The door is a door: it adds no state and no rule to the simulation. Which
fields of the leader are the six the original's bar shows, and at what
scale each is stored, is settled against the dump's `LEADERDATA` record
before the first count, the consumer read before the digits are believed
(`CLAUDE.md`, "Tooling").

**Settled (4198), by the consumers.** `IFaceResources::draw_resource@00818840`
reads, for good `g`, the leader's encrypted block (`LeaderData +0x6eb8`,
`LeaderDataEncrypt` by its type record): the number at `+0x0 + 4g`, which is
`bucket`, the stockpile in whole units; the flash at `+0x4c + 4g`, `over_cap`;
and the rate at `+0x94 + 4g`, `income` in sixteenths, written behind a `+` as
`(income + 8) / 16`. `IFaceMainBase::draw_pop_cap@0080ff90` writes
`LeaderData +0x940` over `+0x7e4`: `control` over `pop_cap`, not the census's
`pop`. The goods are in the record's order: food, timber, wealth, knowledge,
metal, oil.

**Built (4198).** `sim::observe::LeaderScene`, one per player slot in
`Scene::leaders`: `who`, `tribe` and its `nation` name (the tech tree's
`<TRIBE name>`, copied), `ages`, `bucket`, `income`, `over_cap`, `control`,
`pop_cap`. `client::hud::Bar` prints the issuing slot's along the window's
top edge, at the right, as `FOOD 1032 +40 … POP 6/25 AGE 1 NATION <name> FRAME n`: each
stockpile and its shown rate, `CAP` after a good at its cap, the population,
the age counted from 1 (`ages + 1`), the nation, and the frame for the clock.

**The clock, read and built (4203).** `IFaceMainBase::draw_button_row_area@0080f7b0`
writes the speed's name, `: `, and `say_timer@007f1b00` of `Game::tick`
(`gamec +0x560`, by the type record; the listing passes it in `edx` at
`0080f937`). `say_timer` writes `s / 3600`, `(s % 3600) / 60` and
`s % 60`, each through `say_timer_num@007f1a70` (a `0` before a number
under 10), joined by `:`. `Game::do_frame@00591ef0` raises `tick` by one
when the frame count, after its own step, is a multiple of 15
(`TurnControl::check_cannon_time` beside it); `Setup::build_game` and
`Game::clear` set it to 0, and nothing else writes it. So the clock is
`frame / 15` seconds, and the bar prints it after the frame as
`TIME hh:mm:ss` (`client::hud::clock`). The speed's name is not ours to
print (§6). A ranked game's first 60 seconds take the other arm, a
countdown `say_short_timer(60 − tick)`; not built.

**The rates' writing, read and built (4203).** `draw_resource` writes the
stockpile, a `/`, then the rate's string: `internal_strings.xml` entry 404
(`int_str_array +0x1f90`, the string `0`) for a zero `income`, else the
digits of `(income + 8) >> 4` with the sign bias. It then puts a `+` in
front **unless that string equals entry 404**. So a rate that rounds to
zero reads `0`, a rising one `+40`, a falling one `+-3`. run1570's
photograph shows `20240/+10` and `20000/0`. The bar writes each rate so
(`client::hud::rate_text`), where it wrote `+0` and `-3` before.

## 5. The scores, each with how it fails

**Item 4197.**

- **`Hud: the clicked file from buttons`, k of k orders.** The clicked
  script's events (`docs/CLIENT_INPUT.md` §5) with each key replaced by a
  press on its button's pixel, through the window's own routing, mint the
  same orders; the file written is the clicked file byte for byte, the
  same SHA-256 the playback of run1563 already scored. Pinned first as
  drawn today, 0 of k with no button. Made to fail: a button's rectangle
  moved off its pinned pixel loses its order and the count falls.
  **Built (4197): 2 of 2 keyed orders** (`script::HUD_BUTTONS`; the `c`
  `queue_up` and the `f` `build`), all three orders the clicked script's
  and the file the clicked file's 59,280 bytes. Through the layer alone
  (the window before this item) 0 of 2; on a window 80 pixels shorter,
  the buttons off their pixel, 0 of 2.
- **The press that is the HUD's is not the world's.** A press on a button
  with units selected leaves the selection whole and places nothing; the
  same pixel with no button there is the layer's press. Each seen red
  with the routing reversed.
- **`Hud: the panel's values`, k of k.** For the selection the scripts
  make, on the frames the scripts' playbacks dump, each printed number is
  the dump's own (`UNITDATA`'s and `BUILDDATA`'s health and full health)
  for that object on that frame. This checks the panel against the
  original and not against the scene it was read from. **A quiet field is
  checked against the reader first**: a selection whose units are all at
  full health cannot part on health, so at least one compared object is
  one the dump shows damaged or unfinished (the Farm's site under
  construction in the right-pressed script is one).
  **Built (4197): 628 of 628** (`script::HUD_PANEL`), on frames 1 to 299
  of run1568, for the right-pressed script's selections (the Scout, the
  capital, the builder, the helper, the trained Citizen) and one press
  selecting the site `0/2007` on frame 210; 44 of the 628 are the site's
  `construct_hits` while unfinished, 223 on 210 to 396 on 253. Made to
  fail: that number printed one higher, 584 of 628; the dump read one
  frame late, 536 of 626. Read first as `myhits − damage`, the site's
  health parted on the same 44 (ours 223, theirs 400 on 210), and §3's
  reading settled which side was wrong.

**Item 4198.**

- **`Hud: the bar's values`, k of k.** On every frame of the harness
  lobby's captures where the level-9 `LEADERDATA` record prints, each of
  the bar's numbers is the record's, for both the human's slot and one
  computer's. Pinned first with no bar, 0 of k. Made to fail: one resource
  read from the neighbouring field.
  **Built (4198): 57,420 of 57,420** (`script::HUD_BAR`;
  `client::hud::tests::the_bar_s_values_are_the_leader_record_s`). Our
  side is the bar's printed text, parsed back, for `Bar::of` on the
  scene `sim::observe` gives of the harness lobby's walk
  (`FreeRunner::from_lobby` on run53's `GAMEINFO`, stepped to frame
  11,472). Their side is the record, read through `rondata::gamelog`
  alone: the nine Great Lakes leader captures of that game
  (`diff::leader`'s merchant files, frames 6950 to 11471), 1,305 frames
  carrying the level-9 block, slots 0 and 1, 2,610 records. 22 values a
  record: per good `bucket`, `(income + 8) / 16` and whether `over_cap`
  is set; `control`, `pop_cap`, `ages_get() + 1`; and the nation, the
  record's `tribe` joined through the tech tree's names. 3,138 thinner
  blocks of those slots carry no goods and are not compared. Made to fail:
  each good showing its neighbour's stockpile, 44,389 of 57,420; the
  record read one frame late, 56,326 of 57,420. Slots 8 and 9 (printed,
  no player) are not compared.
- **The clock's line** was pinned only once its rule was read (4203, §4).
  The record prints no clock, and the printed frame is the walk's own
  step count, aligned to the dump's `FRAME` header by construction, so
  the frame is not counted: a comparison that cannot fail.
  - The bar's `TIME` gets most of its backing from the packets. **109 of
    109** (`script::HUD_CLOCK_PACKETS`;
    `client::hud::tests::the_clock_is_the_original_s_tick`): on every
    packet on disk, frames 91 to 18,368 on maps 7, 14 and 18, the packet's
    `Game::tick` is `clock_seconds` of its `Game::frame`, and the printed
    `TIME` is that tick written `hh:mm:ss` by arithmetic apart from
    `clock`. Their side is `tools/render/packet_clock.py`'s file, outside
    the repository. Made to fail: the victory timers' `(frames + 14) / 15`
    agrees on only the 9 packets whose frame is a multiple of 15.
  - **3 of 3 photographs** (`script::HUD_CLOCK_PHOTOS`;
    `the_clock_reads_the_photographs`): run1559b and run1564 on frame
    3279, run1570 on 3327. Each frame is the movers' (items 4073, 4191),
    never the clock's, and each reading (`00:03:38`, `00:03:38`,
    `00:03:41`) is by eye and pinned. This also settles 3331's reading:
    the photograph was taken paused on 3331, but the fast-forward last drew
    3327, and 3327 / 15 is 221. Made to fail: the victory timers' rule
    reads `00:03:39` and `00:03:42`, 0 of 3.

**Item 4203.**

- **`Hud: a wounded unit's panel`, 840 of 840** (`script::HUD_WOUNDED`;
  `client::hud::tests::the_panel_s_damage_half_is_the_dump_s`). The
  walk is the attacked script's orders, issued on the lobby on their
  frames. The selection is the Scout `1/0` and the Slinger `0/7`. On
  each of run1571's 210 printed frames (6690..6899), each printed number
  is compared with the dump's `myhits − damage` and `myhits`. The Scout's
  hits left fall 50, 47, 44, 41, 37, 34, 31, and 95 of the lines are
  wounded. Made to fail: full hits printed (the damage dropped), 745 of
  840, every wounded line parting; the dump read one frame late, 830 of
  836. No capture was taken. run1571, a Great Lakes playback in lockstep
  with our walk, already holds the wounded unit.
- **The rates' writing.** In the bar's 57,420, a rate token is read back
  only in the original's writing (`0`, or `+` and a non-zero number).
  The bar's old writing, `+0` for a zero, parts on exactly the 5,534
  zero rates the records hold (49,276 of 54,810). A negative rate is in
  no record, so `a_rate_is_written_as_the_original_writes_it` checks
  that arm against the listing's arithmetic, written again from −400 to
  400.

The queue's `Hud:` line is the driver's to write once the first count
exists; its parser goes beside `lobby_line` in `rondata::diff::floors`.

## 6. Confidence, and what is not established

**Diff-backed (4197)**: the panel's numbers, against run1568's
`UNITDATA` and `BUILDDATA` on every frame they print (§5); the buttons'
orders, by the clicked file's bytes, which the original played in
lockstep (run1563). **On a reading alone**: §3's hits-left rule for a
building (three functions, one reader; the 628 agree under it and 44 part
without it, so the diff stands behind it for a site with no damage and
no queue). **Diff-backed (4198)**: the bar's six stockpiles, six shown
rates, six cap marks, population, cap, age and nation, against the level-9
`LEADERDATA` record on every frame the nine Great Lakes leader captures
print it (§5). **On a reading alone**: which fields the original's bar
shows (§4, two consumers, one reader); the diff cannot say the original
draws `bucket` rather than another field, only that ours prints the
record's `bucket`. **Diff-backed (4203)**: the clock, against
`Game::tick` on 109 packets and three photographs' readings; a wounded
unit's hits left, against run1571 (§5); a zero rate's `0`, against the
5,534 zero rates and the photograph's `/0`. **On a reading alone**: a
negative rate's `+-n`; the ring's radius and its one circle for each
figure (one reader, two functions, and the stills by eye). What the bar
rested on before it was built:

- **Read from the tree by the pass**: the scene's fields (§1, §3), the two
  key tables, the absence of any leader state behind the door.
- **By eye, on three photographs**: that the original's bar shows six
  resources with a rate each, population over a cap, the nation and the
  age, a clock, and that the selection's panel shows a name and hits.

Not established:

- **What the original's menu offers.** Which buildings and units a
  selection may order at its age and research, and their order on the
  panel. The simulation refuses an order it cannot take; the menu here
  does not ask it. A later item makes the menu the legal list and scores
  it by the orders the simulation accepts.
- ~~**The resources' rates** (the `+10`, `+70` beside each on the
  photographs) and where the original keeps them.~~ Read (4198, §4):
  `income` in sixteenths, shown as `(income + 8) / 16` by
  `IFaceResources::draw_resource@00818840`; the bar's 15,660 rate values
  agree with the record's `income` under that rule (§5). ~~Not read: what
  the original writes for a zero rate and for a negative one~~: read
  (4203, §4). A zero reads `0`, a negative `+-n`. The negative arm rests
  on the reading and the listing's arithmetic alone, since no record
  holds a negative `income`.
- ~~**The clock's rule**: frames to the time shown, at each game
  speed.~~ Read (4203, §4): `Game::tick`, `frame / 15`, written by
  `say_timer`. Diff-backed on 109 packets and 3 photographs (§5). The
  game speed changes how fast frames pass, not the rule. Still not
  established: **the speed's name** before the time (`game_speeds` at
  `turn_control->speed`; `Normal` on every photograph), and **a ranked
  game's countdown**, the first 60 seconds' other arm.
- **The ring's place in the draw.** The original draws it among the
  underlays, before the objects. Ours is a quad in the HUD's overlay,
  drawn over the world: a ring shows through a tree or a building
  standing in front of its figure. No row counts the ring against a
  photograph.
- **A building's marker.** `draw_funky_rectangle_circle` at
  `BuildData::max_range` is not read, and its shape and size are not
  ported; ours outlines the footprint.
- **The selected building's band, drawn** (items 4398 and 4401, §3): its
  geometry is counted on run1559b (`RUN1559B_3279_SELECTED`) and drawn
  untextured at one constant alpha. Not read: string `+0x1784`'s texture
  and its coordinates (`TexArray::add`'s arguments are registers the
  decompiler lost), which is why ours is wider and harder than theirs;
  card mode 3 (`+0x2c(3)`): whether it depth-tests as `render_states[3]`
  does for land units (0x20287, `docs/RENDERER.md` §3.7) is not read, and
  ours tests nothing; `get_unit`'s alpha being `0xff`; the listing's alpha
  0 for a vertex off the map (ours carries one alpha a pass); and the
  draw's place among the other underlays. The name plate is
  `BuildOut::draw_overlays@0062cb30`'s (`CityData +0x90`'s name on
  `IFaceRenderManager::draw_scroll` in the owner's colour, under
  `player_profile.prefs & 0x40000`, every seen city, selected or not), a
  drawing helper rather than an `iface*` window; the bar is
  `WallOut::draw_health_bar@00640dd0`/`@00640f20`'s (§3, item 4410).
- **The plate, drawn** (items 4403 and 4410, §3): its rectangle and its
  drawing are counted on run1559b (`RUN1559B_3279_PLATE`). Not
  established: **the name.** The sim carries no founding name
  (`sim::cities::rename`'s SEAM), so the client draws a plate only for a
  city a rename has named, and the row names `Napata` itself (item 4416
  minted the name's own row on the packets, `CITY_NAMES_SCORE` 0 of 521;
  a captured city's name is not read, parked 4416a); and the
  font's advances are held for `N`, `a`, `p` and `t` alone
  (`selection::PLATE_ADVANCES`, the packet's): a name with any other
  character draws no plate rather than a guessed one. **The glyphs** are
  the overlay's 5 × 7 in white, not the font's, which is
  `D3D11GLFont`'s from the system's face (`sbl/d3dgl.pdb`) and not read.
  Also not established: which architecture set's `TEX_CIV` the original
  loads (`iface_AS0_assets.tga` is `interface.xml`'s; `AS3`'s scroll
  differs only in the body's `(0, 0, 0)` for `(1, 1, 1)`), `tile_random`'s
  offsets (the body is one texel, so they draw nothing different),
  `add_smooth`'s blend of the star (ours is the texel times the colour,
  at the texel's alpha), the prefs bit's default and the visibility
  branch's arms (`semaphore & 0x42`, the ally and the over-player
  cases), and whether a rectangle end is inclusive (the photograph's gold
  edge fits `[x0, x1)`).
- **The bar, drawn for a selection** (item 4410, §3): counted on
  run1559b (`RUN1559B_3279_BAR`). Not established: the arms for the
  mouse-over, `+0x24` and another player's unfinished building; the
  `GREEN` and `YELLOW` globals' values (ours the plain primaries; the
  photograph shows only `RED`, `(255, 0, 0)`); the zoom level off any
  camera but run1559b's packet (`selection::BAR_ZOOM_LEVEL` 5: the client
  carries no `Camera::zoom_level`); `screen_scale_x` and `_y` other than
  1; and the place of the bar and the plate among the other overlays.
- **Slots 8 and 9.** The record prints two more leaders (no tribe, no
  goods); the bar shows the issuing slot only and they are not compared.
- **A narrow window.** The bar is one line, about 1,430 pixels at the
  font's scale; a window narrower than that clips it at the left.
- **The messages, the minimap, the commerce caps and the alerts** on the
  photographs. None is in these two items.
- **Whether a HUD drawn into the frame moves a still's row.** The stills'
  rows are counted on a frame with no HUD; the HUD is drawn only in a
  window, never under `--still`, so every pinned row holds by
  construction, and the first lane says so with the rows' own tests.
  Built (4197): a row's still (`still_of`) draws no overlay and no HUD;
  the client's `--still` draws the HUD only under `--select O[,O…]`, a
  look for a person and no row's.
- **`BuildData::hits`' queue arm.** With the building finished and the
  head of its queue `TypeIndex` 0x29a or 0x286, `hits(0)` is scaled by the
  queue's progress (a float in the original); no compared object holds it
  and the scene does not carry it, so a building in that state may print
  other hits left than the original's.
- ~~**A damaged object.** Every compared object is at full hits left but
  the site, whose `damage` is 0~~: compared (4203, §5), the Scout on
  run1571, 840 of 840. A damaged **building** is still in no compared
  dump.
- **What a press on the panel does**: it reaches the world (only a
  button takes a press). A right press on a button reaches the world too.
