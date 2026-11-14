# The tracks

The finish line, decomposed into the avenues the loop can run, each with
the number a landing moves and the gate that says the move was real. The
steering pass maintains this page (`docs/LOOP.md`, "the passes"); the
queue books from it; no backlog file stands under it (`docs/DECISIONS.md` entry 125). A fresh
session reads the queue first and this page second, to know which track
its item serves. Written at the seventy-fourth pass, 2026-11-08, with Ramon
on the steer row (`docs/DECISIONS.md` entry 114); the milestone and the
order of play are the seventy-sixth's, 2026-11-09, also with Ramon (entry
116).

## The finish line (decided with Ramon, the seventy-fourth pass)

Our own app, no Wine. It opens in a moment. A lobby asks for a map, a
nation and computer opponents; one click, and the game is on screen fast,
drawn with the original's art, running smooth and light. You play a whole
game against the computer. Under the picture the rules are the original's
exactly: fed the same orders, Rise of Nations does the same thing frame for
frame. A game can be saved and replayed, by us and by the original.

**What "exactly" is counted in** (entry 132 §10, Ramon, 2026-11-13): zero
mismatched frames on the twelve state checksums over a declared set of
games. The original computes them every turn and prints them only in a dump
window; step H (item 4214) hooks them every frame. Until it prints, the
draw stream's floors stand in, and they are blind to state: two files three
units apart agree on every draw of 24,001 frames.

Not in it, by design: multiplayer, Conquer the World, the editor, new art.
Each is a finish line of its own, after this one is finished and tested.

## The milestone: one lobby, all the way through

**A game played here, on a start built here, is replayed by the original and
agrees with it for its full length.** One lobby — Great Lakes, the harness's
own — taken through every track it needs before any track goes wide:

1. **The start, built** from the lobby's choices with no dump of the
   original (`Start: 1 of 161`; 161 of 161 since item 4211).
2. **The game run from that start** in lockstep with the lobby's long
   capture, where it ran from an installed dump before.
3. **A recorded game written** in the original's container, and the
   original playing a file we wrote (`docs/RECGAME.md`, the playback
   preflight).
4. **Orders minted through `Command`** — a script first, the client's mouse
   after — carried in that file.
5. **The checksum**, ported, agreeing on the frames the playback prints.

Once a person plays, no capture of the original exists to diff against. The
original replaying our game is the only oracle left, and it scales to every
game anyone plays afterwards. So this is reached before the start block is
taken across the other 160 lobbies, before M is counted on a second lobby,
and before the census filler: breadth on a track is the filler, and the
chain is the headline. The steps are booked in `docs/QUEUE.md`; the driver
designs each as the one before it lands.

## The tracks

| track | what "done" is | the number | its gate | state at the pass of 2026-11-14 (earlier states in the passes' records under `docs/audit/`) |
|---|---|---|---|---|
| **rules** (phase 3) | every captured game replays in lockstep for its full length | the scoreboard's words, `rondata::diff`'s floors; entry 29's three counters | the release diff suite | **whole** (`Scoreboard:`, `Golden:`): every word at its length, 130 chapters closed; no item stands; a chapter is booked only when another track's frame needs one |
| **structure** | the sim crate has a public surface and subsystem boundaries, the tests and sweeps live outside the source, nothing is defined twice | `docs/STRUCTURE.md`'s shape counts (files in the flat root, lines of test code in `src/`, duplicated names, the `Sim` struct's fields), each falling to its target; **every floor unchanged on every landing** | the whole release gate, no behaviour diff | every target of `docs/STRUCTURE.md` met (`Structure:`), and a gate on every later move; item 4224, one walk a game, is booked under its kind, whole |
| **renderer** (phase 4) | a captured game drawn with the original's art, placed where the original would draw it | pieces in register K of M (`docs/RENDERER.md` §5.4); frames in register as its floor; the clock: seconds to lobby, seconds to frame zero, slowest-percentile frame milliseconds, peak resident memory | a `render` crate's pins, parsed as the queue's | `Renderer:` K 59 of M 60 (4208's birds), frames 35 of 35; the rows read the client's still (4171), and the ground, the coast, the land's light and tint, the trees, the ring, the paved base and the trees' shadows have landed on it (4173 to 4223); next the lake's last 25 tiles or 4227c's HDR floor |
| **start** | a game starts from a lobby's choices with no dump of the original: map generation's remaining chain and the frame-zero start builder | the start block equal to the original's on every lobby on disk (the dumps print it) | `rondata::diff` on the start blocks | **closed**: `Start: 161 of 161 lobbies` (4034, 4186, 4196, 4211), the harness's lobby walked from it for 24,000 frames |
| **checksum** | the original's full-state hash, ported, compared per frame against the original's own | ~~frames whose hash agrees, of the frames the original prints under `CHECKSUM=2`~~ the state lines that agree, of twelve, on the frames a `DUMP_ALL` window of the playback prints (entry 119: that setting prints no frame) | the diff suite | **open**, the milestone's step 5 (`Checksum:`): state 8 of 12 lines, lines 421 of 1,053; the leaders line the last that counts, `LEADERS_HELD_OUT` 27 (item 4159); parity 0 of 1 games, step H printing every frame (4214, run1578), its first partings moved (units 3166, item 4230) |
| **orders** | a mouse and keyboard mint the same order stream the replays carry; the stream is the replay and, later, the network format | orders issued through the public `Command` API replaying in lockstep | the diff suite on a minted stream | **step 4 closed** (`Recording:`): clicks, right presses, a box, an attack and a follow, each minted here and played by the original (4149 to 4202); no item stands |
| **lobby** | our own screen producing the start's choices | a page of client code; no counter | the client's tests | `Lobby:` 161 of 161; a plain HUD, the menu and the panel (`Hud:`; 4197, 4198, 4203) |
| **AI** (phase 5) | a computer opponent plays a whole game against a person in our client | the held-out battery, games nobody tuned for scored on their first try; then the original replaying games played here against the computer | the diff suite; the original on the capture lane | **ported as far as the captures reach**: the computer's decisions are computed here, not replayed, in every lockstep game on the scoreboard, at four difficulties (`Held-out:` 8 of 9 whole). What remains is the branches no capture entered, which a person's play will enter and the milestone's oracle will score |
| **save and replay** | a game saved as seed plus orders, replayable here and written back into the original's recorded-game container | a saved game replays to the same hash; the original replays one of ours | the diff suite; the original on the capture lane | **step 3 stands** (`Recording:`): every recording on disk round-trips, and the original plays our order-free file for 24,000 of 24,000 frames (4058, run1560) |
| **census** | the territory count tells the truth about the original's 3,611 simulation functions | ported · replaced by design · open, the readers' marks landed row by row (`docs/audit/2026-11-06-territory-*.md`; lore's scoring of the `replaced` groups) | `tools/census.py`'s pins | `Rubric:` 574 open of 3,611, 569 of them carrying a reader's ruling (entry 132); filler for an idle lane |

**The order of play** (the seventy-sixth pass, entry 116). The milestone
first, one lobby deep, on the driver's own builders; the renderer beside it
on one lane, taking textures and team colours next; then the lobby screen
and the client's input, which the milestone's steps 3 and 4 make
checkable; then breadth — the other lobbies' start blocks, M on more
lobbies, the census filler — as filler for an idle worker. The rules track
is whole and stays whole: its floors run on every gate forever.

**The workers** (entries 116, 126, 128 §4, 129 §1): the driver's builders on
the chain and three lanes, four in all, confirmed at the pass by load. At
most one lane holds an item the queue marks whole. No lane gates whole but
a `structure` lane since entry 132 §6: a lane's gate runs its brief's
filters and the booking gate, scoped, runs the corpus. The count is
Ramon's, and a pass moves it only by recommending; he may add a fifth or
sixth mid-tranche on the booking gate's minutes.
