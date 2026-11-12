# The HUD, plain

Items 4197 and 4198 (DECISIONS 129 §7), designed by the pass of 2026-11-12
before either was cut. **Nothing here is built.** This is the document the
lanes implement and amend; a claim below is a rule to hold the build to,
not a finding.

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
| the bar | the six resources, population over its cap, the age, the nation, the game's clock | **nothing the door carries**: `Scene` holds no leader state | 4198 |

That split is the gate's. The menu and the panel read what the scene
already holds, so 4197 touches `crates/client` alone and gates scoped. The
bar needs a leader's record through `sim::observe`, a new door, so 4198
gates whole and takes the whole slot's turn (DECISIONS 129 §1).

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

## 4. The bar (item 4198)

A `LeaderScene` through `sim::observe`, read-only, for the issuing slot:
the six resources, the population and its cap, the age, the tribe. The
bar prints them as integers in the original's own units, with the nation's
name from the tables and the clock as the frame count and as the time the
original's own bar shows (the photographs read `00:03:38` on frame 3279
and `00:03:41` on 3331 at normal speed: three seconds over 52 frames; the
rule behind it is not read, §6).

The door is a door: it adds no state and no rule to the simulation. Which
fields of the leader are the six the original's bar shows, and at what
scale each is stored, is settled against the dump's `LEADERDATA` record
before the first count, the consumer read before the digits are believed
(`CLAUDE.md`, "Tooling").

## 5. The scores, each with how it fails

**Item 4197.**

- **`Hud: the clicked file from buttons`, k of k orders.** The clicked
  script's events (`docs/CLIENT_INPUT.md` §5) with each key replaced by a
  press on its button's pixel, through the window's own routing, mint the
  same orders; the file written is the clicked file byte for byte, the
  same SHA-256 the playback of run1563 already scored. Pinned first as
  drawn today, 0 of k with no button. Made to fail: a button's rectangle
  moved off its pinned pixel loses its order and the count falls.
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

**Item 4198.**

- **`Hud: the bar's values`, k of k.** On every frame of the harness
  lobby's captures where the level-9 `LEADERDATA` record prints, each of
  the bar's numbers is the record's, for both the human's slot and one
  computer's. Pinned first with no bar, 0 of k. Made to fail: one resource
  read from the neighbouring field.
- **The clock's line** is pinned only once its rule is read (§6); until
  then the bar prints the frame and the photograph's two readings stand
  as the first check a rule must pass.

The queue's `Hud:` line is the driver's to write once the first count
exists; its parser goes beside `lobby_line` in `rondata::diff::floors`.

## 6. Confidence, and what is not established

Nothing here is diff-backed yet: it is a design. What it rests on:

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
- **The resources' rates** (the `+10`, `+70` beside each on the
  photographs) and where the original keeps them.
- **The clock's rule**: frames to the time shown, at each game speed.
- **The messages, the minimap, the commerce caps and the alerts** on the
  photographs. None is in these two items.
- **Whether a HUD drawn into the frame moves a still's row.** The stills'
  rows are counted on a frame with no HUD; the HUD is drawn only in a
  window, never under `--still`, so every pinned row holds by
  construction, and the first lane says so with the rows' own tests.
