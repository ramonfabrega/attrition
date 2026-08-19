# File formats

The reverse-engineering log. **Every claim here must cite evidence** — a byte
offset from a real file, a cross-check against a second file, or a link to
prior work. No inferred struct layouts, no "probably a length prefix". If we
cannot show why we believe something, it does not get written down as fact; it
goes under *Open questions*.

---

## Verified: the shipped install (2026-08-19)

Depot `287450` pulled to `game/` (2.87 GB). Everything in this section is a
direct observation of that install.

**The XML layer ships with formal schemas.** `game/Data/` contains `.dtd` files
alongside the data they describe:

| Schema | Lines | Describes |
| --- | --- | --- |
| `rules.dtd` | 685 | `rules.xml` (2,438 lines) |
| `unitrules.dtd` | 61 | `unitrules.xml` (20,857 lines) |
| `buildingrules.dtd` | 48 | `buildingrules.xml` |
| `techrules.dtd` | 28 | `techrules.xml` (1,621 lines) |

Also present with schemas: `resourcerules`, `craftrules`, `citytemplates`,
`goods`, `paramtypes`, `soundtypes`, `soundfiles`, `sound`, `playerprofile`,
`triggerbuilder`. Plus `unitrules.xsd`, `sound.xsd`, and a `.sps` file beside
most `.dtd` (schema-project files from whatever editor Big Huge Games used).

**This is bigger than it looks.** A DTD is a machine-readable specification of
the data model — element structure, attribute names, cardinality, enumerated
values, defaults. Phase 1 was scoped as reverse-engineering; for the XML layer
it is not. It is schema-driven code generation, and the schemas are *theirs*,
not our inference. `rules.dtd` at 685 lines is effectively Big Huge Games'
own description of how a nation, a government, an age, and an attrition rule
are shaped.

**The AI is partly scripted, in the open.** `game/ai/scripts/` contains three
`.bhs` files: `aibestbuildlibrary.bhs`, `defensive.bhs`, `economic.bhs`. Three
files is far too little to be a whole RTS AI, so the tactical core is
presumably in the executable — but build libraries and economic/defensive
posture are exactly the layer that is hardest to reconstruct by observation.
Phase 4 was called the hardest and least-oracled part of the project; this
softens that, and the phase plan in `CLAUDE.md` should be revisited once these
three files have actually been read.

**Two executables:** `riseofnations.exe` and `patriots.exe` — base game and
Thrones & Patriots, shipped side by side rather than merged.

**Loose plain-text data at the install root**, outside `Data/` and outside any
archive: `balancerules.txt`, `counterchart.txt`, `game.txt`, `graphics.txt`,
`interface.txt`, `labels.txt`, `masks.txt`, `soundlist.txt`, `soundtypes.txt`,
`taunts.txt`, `saveobjects.txt`, `obsoletescriptfuncs.txt`. `counterchart.txt`
is the likely home of the rock-paper-scissors combat matrix.

**Not yet identified:** `rules.dat`, `Ron.s14`, `rise xml.spp`, and the `bond`,
`sbl`, `tribes`, `mapstyles`, `conquest`, `scenario` directories.

**Localisation noise:** many `Data/` files have `.xml.4`, `.xml.7`, `.xml.9`
… siblings. These are per-language variants and can be ignored wholesale.

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

This applies to the *binary* formats only. As the verified section above
records, the XML layer ships with its own DTDs and needs no reverse
engineering at all.

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

### 3. XML rules — *not a reverse-engineering problem at all*

Confirmed present as plain text, with formal DTDs (see the verified section
above). Generate Rust types from the schemas rather than hand-writing them; the
DTD is the authority, and community documentation is only a cross-check.

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
