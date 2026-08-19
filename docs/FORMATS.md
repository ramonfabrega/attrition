# File formats

The reverse-engineering log. **Every claim here must cite evidence** — a byte
offset from a real file, a cross-check against a second file, or a link to
prior work. No inferred struct layouts, no "probably a length prefix". If we
cannot show why we believe something, it does not get written down as fact; it
goes under *Open questions*.

Nothing in this file has been verified yet. Everything below is either prior
art from other people or a hypothesis to be tested once we have the game.

---

## Prior art

The only serious public RoN format work is
[ptasev/Rise-of-Nations](https://github.com/ptasev/Rise-of-Nations):

- **BIG archive extractor** — unpacks the game's asset containers.
- **BH3 / BHA ↔ glTF converter** — 3D models, both directions. This matters
  disproportionately: the ability to convert *back* to BH3 means new art can be
  tested inside the original game long before our renderer exists.
- A Blender addon (unmaintained) and unreleased 3ds Max plugins.

Neither the repo nor RoN Heaven's modding library publishes an actual byte-level
format spec, so anything we need precisely we will have to establish ourselves —
reading their implementations counts as evidence, guessing does not.

---

## Formats by priority

### 1. Recorded Games — **the oracle, and completely undocumented**

Written to `Documents/My Games/Rise of Nations/Recorded Games/` when the save
option is enabled. Playback is via *Tools and Extras* in the main menu.

This is the highest-value unknown in the entire project. An RTS recording is
almost certainly a lockstep order log — an initial game configuration followed
by timestamped player commands — because that is the only compact way to store
a full match, and RoN is a lockstep game.

If that holds, it gives us three things at once:

1. The **complete order vocabulary** — every command the sim must accept.
2. A **test corpus** — real matches to replay against.
3. The **divergence metric** — feed orders in, compare our state to the
   original's at each tick. Ticks-before-divergence becomes the project's score.

No public parser exists. `RepInfo` (an old third-party replay manager that
supported RoN/RoL and surfaced player names, nations, and colours) is proof that
at least the header is tractable.

**Open questions:** Container compressed? Is a periodic state checksum embedded
(most lockstep games store one for desync detection — if RoN does, that is a
free, precise oracle rather than an inferred one)? Header versioned across EE
patches?

### 2. BIG archives

The asset containers. Extraction already solved by ptasev's tooling; we need our
own reader eventually, but borrowing to start is correct — Phase 1 is about
getting at the contents, not about owning the container.

### 3. XML rules — *not actually a reverse-engineering problem*

`rules.xml`, `unitrules.xml`, `buildingrules.xml` and friends ship as plain text
and are documented by the modding community:

- `rules.xml` — nation powers, government effects, attrition rates, population
  caps, game-setup options, and the global scaling factors (`UNIT_COST_FACTOR`,
  `UNIT_MOVE_SPEED`) that the per-unit numbers are expressed relative to.
- `unitrules.xml` — per-unit `ATTACK`, `HITS`, `MOVES`, `COST` (base cost ×10,
  with a letter for resource type), plus which nation gets which unit and which
  graphics it uses.
- `buildingrules.xml` — the same shape for buildings.

Notable: **multiplayer aborts on data-file mismatch between clients.** That is
strong evidence these files feed the deterministic sim directly rather than a
presentation layer — which is exactly what we want, and it means our sim can
consume the same files unmodified.

### 4. BH3 / BHA models

3D geometry and animation. Converters exist both directions. Not needed until
Phase 3, but *reading* them early is what proves out an eventual art swap.

### 5. BHS scripts

Scenario/campaign scripting that runs alongside the XML and can add behaviour
the data files cannot express. Scope unknown. Relevant to Conquer the World and
custom scenarios; almost certainly deferrable past Phase 2.

### 6. Map / scenario formats

Undocumented publicly. Needed for Phase 2 — a recorded game is worthless
without the map it was played on. Likely embedded in or referenced by the
recording.

---

## Open questions

- Do recorded games embed a periodic state checksum? *(Determines whether our
  oracle is exact or inferred — the single most consequential unknown.)*
- Do EE's data files differ from the 2003 originals in ways that matter to the
  sim? Worth diffing if original CDs are available.
- Is the map stored in the recording, or referenced by name and hash?
- How much simulation behaviour lives in `.bhs` rather than the XML?
