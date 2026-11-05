"""plant.py — one console `weather` plant's re-mark, and whether it enters
the diagonal-transition arm (`docs/MAPGEN.md` §19, §20; item 3876).

    plant.py scan  [--style N]                  every distinct grid on disk, every single plant
    plant.py cell  FILE X,Y [X,Y ...] [--set X,Y,LAND,SUB ...] [--printed FILE]

A model of the chain as read from the decompile, never a source:
`ConsoleWin::run_cmd@007d6a70` case 99 → `Terrain::spot_update_land@0084fa50`
(land `BASELAND`, sub 3) → `TerrainOut::add_update_wcoord@008738a0` →
`Terrain::verify_land@0084d8a0` → the four-edge reset → per listed cell
`TerrainOut::mark_trans_flags@008625a0` (`check_for_tendrils@00859eb0`,
`set_edge_x@00859500`, `set_edge_y@00859410`) →
`TerrainOut::create_transition_renderables@00862c60`'s
`check_blocks@0085b3e0` and its `word == 0` test, which enters
`TerrainOut::check_for_diagonal@008595f0`. Only the shift-6 field of each
edge is held. Every edge write is logged as the original's `Set edge` print
is (`TERRAIN` at detail 40), so a `--printed` file's first `Set edge` line is
compared write for write: run1495's 81 agree (`--set 41,42,2,0` reverts its
plant, then `41,42`).

`scan` reads each gamelog's first `WORLD` block that carries cells, dedupes
the grids, and plants every cell under `COAST_BASE` 3 and 0 (the shipped
tilesets: four `BASELAND` bases, one `SANDY`, one `OCEAN`). `cell` plants in
the order given, each on the grid the one before left.
"""
import os, sys, hashlib, collections
from multiprocessing import Pool

ARCHIVE = os.environ.get(
    "RON_GAMELOG_DIR",
    os.path.expanduser("~/ron-data/AppData/Roaming/Microsoft Games/Rise of Nations/Logs"))
BASELAND, SANDY, OCEAN, NONE = 0, 1, 2, 3
# move_x/move_y[0..8]: centre, then NW N NE E SE S SW W (world::MOVE_49).
MOVE = [(0, 0), (-1, -1), (0, -1), (1, -1), (1, 0), (1, 1), (0, 1), (-1, 1), (-1, 0)]


class Combo:
    def __init__(self, coast_base, num_tiles=(4, 1, 1, None)):
        self.cb = coast_base
        self.nt = num_tiles

    def __call__(self, l1, s1, l2, s2):
        """TileSet::is_combo_acceptable@0087abe0: 1 = (l2,s2) one up."""
        if l1 == l2:
            if abs(s2 - s1) < 2:
                return 1 + (s2 <= s1)
            return 0
        if l1 == BASELAND:
            if l2 == SANDY:
                return int(s1 == self.cb)
        elif l1 == SANDY and l2 == BASELAND:
            return 2 if s2 == self.cb else 0
        if l2 == OCEAN:
            return 1
        if l2 < l1:
            if s2 == self.nt[l2] - 1:
                return 2 if s1 == 0 else 0
        elif s2 == 0:
            return int(s1 == self.nt[l1] - 1)
        return 0


class Remark:
    def __init__(self, grid, xs, ys, combo, depth=0):
        self.g = dict()  # overlay: (x, y) -> (land, sub)
        self.base = grid  # list row-major of (land, sub)
        self.xs, self.ys = xs, ys
        self.c = combo
        self.depth = depth
        self.upd = []  # +0x671c, the update list
        self.trans = []  # +0x66f0, the transition list
        self.e6 = {}  # ('x'|'y', a, b) -> shift-6 bits
        self.log = []  # (kind, a, b, v)

    def inb(self, x, y):
        return 0 <= x < self.xs and 0 <= y < self.ys

    def at(self, x, y):
        return self.g.get((x, y)) or self.base[y * self.xs + x]

    def put(self, x, y, land, sub):
        self.g[(x, y)] = (land, sub)

    def cmb(self, a, b):
        return self.c(a[0], a[1], b[0], b[1])

    # -- the edge writers ------------------------------------------------
    def set_edge(self, k, a, b, v, shift):
        if shift == 6:
            self.e6[(k, a, b)] = self.e6.get((k, a, b), 0) | v
        self.log.append((k, a, b, v))

    def raw(self, k, a, b, v):
        # a raw OR at shift 0 (|1, |2, |4) or shift 3 (|0x38, printed 7)
        self.log.append((k, a, b, v))

    # -- spot_update_land ------------------------------------------------
    def add_update(self, x, y):
        for dx, dy in MOVE:
            n = (x + dx, y + dy)
            if self.inb(*n) and n not in self.upd:
                self.upd.append(n)

    def verify_land(self, x, y, cell):
        """cell: [land, sub], the caller's two ints, updated in place."""
        self.depth += 1
        if self.depth > 500:
            return  # the leak: no decrement (MAPGEN 19.6)
        flags = [0] * 9
        for k in range(1, 9):
            nx, ny = x + MOVE[k][0], y + MOVE[k][1]
            if not self.inb(nx, ny):
                continue
            nl, ns = self.at(nx, ny)
            cl = cell[0]
            r = self.c(cl, cell[1], nl, ns)
            if r != 0:
                continue
            if nl in (OCEAN, SANDY):
                # the centre is rewritten to BASELAND at COAST_BASE, recursed
                self.put(x, y, BASELAND, self.c.cb)
                if y * self.xs + x in self.trans:
                    self.trans.remove(y * self.xs + x)
                inner = [BASELAND, self.c.cb]
                self.verify_land(x, y, inner)
                cell[0], cell[1] = inner
                continue
            # LAB_0084db5b (nl BASELAND, or NONE)
            t = ns
            if abs(cell[1] - ns) > 1:
                t = ns + 1
                if cell[1] <= ns:
                    t = ns - 1
            if cl == BASELAND:
                if ns != t and ns != cell[1]:
                    self.put(nx, ny, nl, t)
                    self.add_update(nx, ny)
                    flags[k] = 1
            elif cl in (SANDY, OCEAN):
                self.put(nx, ny, BASELAND, self.c.cb)
                self.add_update(nx, ny)
                flags[k] = 1
        for k in range(1, 9):
            nx, ny = x + MOVE[k][0], y + MOVE[k][1]
            if self.inb(nx, ny) and flags[k]:
                self.verify_land(nx, ny, list(self.at(nx, ny)))
        self.depth -= 1

    def spot_update_land(self, x, y, land, sub):
        if not self.inb(x, y) or self.at(x, y) == (land, sub):
            return False
        self.put(x, y, land, sub)
        self.add_update(x, y)
        self.verify_land(x, y, [land, sub])
        for ux, uy in self.upd:  # the reset: four edges to 0x38
            for key in (("x", ux, uy), ("y", ux, uy), ("x", ux + 1, uy), ("y", ux, uy + 1)):
                self.e6[key] = 0
        for ux, uy in self.upd:
            self.mark_trans_flags(ux, uy)
        return True

    # -- mark_trans_flags ------------------------------------------------
    def mark_trans_flags(self, x, y):
        C = self.at(x, y)
        idx = y * self.xs + x
        not1 = orth = diag1 = 0
        for k in range(1, 9):
            dx, dy = MOVE[k]
            n = (x + dx, y + dy)
            if not self.inb(*n):
                continue
            r = self.cmb(self.at(*n), C)
            if abs(dx) + abs(dy) == 2:
                diag1 += r == 1
                continue
            orth += 1
            not1 += r != 1
            if r == 1 and idx not in self.trans:
                self.trans.append(idx)
                self.set_edge("x", x, y, 7, 3)
                if x + 1 < self.xs:
                    self.set_edge("x", x + 1, y, 7, 3)
                self.set_edge("y", x, y, 7, 3)
                if y + 1 < self.ys:
                    self.set_edge("y", x, y + 1, 7, 3)
                if self.tendrils(x, y, C) == 0:
                    if dx != 0 and 0 <= x + (dx == 1) < self.xs:
                        self.set_edge("x", x + (dx == 1), y, 7, 6)
                    if dy != 0 and 0 <= y + (dy == 1) < self.ys:
                        self.set_edge("y", x, y + (dy == 1), 7, 6)
        if not1 == orth and diag1:
            if idx not in self.trans:
                self.trans.append(idx)
                self.raw("x", x, y, 7)
                if x + 1 < self.xs:
                    self.raw("x", x + 1, y, 7)
                self.raw("x", x, y, 7)
                if y + 1 < self.ys:
                    self.raw("x", x, y + 1, 7)
            self.tendrils(x, y, C)

    def tendrils(self, x, y, C):
        f = []
        for k in range(1, 9):
            n = (x + MOVE[k][0], y + MOVE[k][1])
            f.append(int(self.inb(*n) and self.cmb(self.at(*n), C) == 1))
        NW, N, NE, E, SE, S, SW, W = f
        xs, ys = self.xs, self.ys
        if N == 0:
            if W == 0:
                if NW == 0:
                    self.raw("x", x, y, 4)
                    self.raw("y", x, y, 4)
                else:
                    self.set_edge("x", x, y, 4, 6)
                    self.set_edge("y", x, y, 4, 6)
            if E == 0:
                if NE == 0:
                    if x + 1 < xs:
                        self.raw("x", x + 1, y, 4)
                    self.raw("y", x, y, 1)
                else:
                    if x + 1 < xs:
                        self.set_edge("x", x + 1, y, 4, 6)
                    self.set_edge("y", x, y, 1, 6)
        if S == 0:
            if W == 0:
                if SW == 0:
                    self.raw("x", x, y, 1)
                    if y + 1 < ys:
                        self.raw("y", x, y + 1, 4)
                else:
                    self.set_edge("x", x, y, 1, 6)
                    if y + 1 < ys:
                        self.set_edge("y", x, y + 1, 4, 6)
            if E == 0:
                if SE == 0:
                    if x + 1 < xs:
                        self.raw("x", x + 1, y, 1)
                    if y + 1 < ys:
                        self.raw("y", x, y + 1, 1)
                else:
                    if x + 1 < xs:
                        self.set_edge("x", x + 1, y, 1, 6)
                    if y + 1 < ys:
                        self.set_edge("y", x, y + 1, 1, 6)
        ret = 0
        if N and S:
            if W == 0 and E == 0:
                self.set_edge("x", x, y, 5, 6)
                if x + 1 < xs:
                    self.set_edge("x", x + 1, y, 5, 6)
                ret = 1
        elif W and E and not N and not S:
            self.set_edge("y", x, y, 5, 6)
            if y + 1 < ys:
                self.set_edge("y", x, y + 1, 5, 6)
            ret = 1
        if N + S + E + W == 3:
            if N == 0:
                self.set_edge("y", x, y, 5, 6)
                self.raw("y", x, y, 2)
                if y - 1 >= 0:
                    self.raw("x", x, y - 1, 7)
                    if x + 1 < xs:
                        self.raw("x", x + 1, y - 1, 7)
                ret = 1
            if E == 0:
                if x + 1 < xs:
                    self.set_edge("x", x + 1, y, 5, 6)
                    self.raw("x", x + 1, y, 2)
                    self.raw("y", x + 1, y, 7)
                    if y + 1 < ys:
                        self.raw("y", x + 1, y + 1, 7)
                ret = 1
            if S == 0:
                if y + 1 < ys:
                    self.set_edge("y", x, y + 1, 5, 6)
                    self.raw("y", x, y + 1, 2)
                    self.raw("x", x, y + 1, 7)
                    if x + 1 < xs:
                        self.raw("x", x + 1, y + 1, 7)
                ret = 1
            if W == 0:
                self.set_edge("x", x, y, 5, 6)
                self.raw("x", x, y, 2)
                if x - 1 >= 0:
                    self.raw("y", x - 1, y, 7)
                    if y + 1 < ys:
                        self.raw("y", x - 1, y + 1, 7)
                return 1
        return ret

    # -- create_transition_renderables' gate ----------------------------
    def word_after_blocks(self, x, y):
        C = self.at(x, y)
        sides = [  # (field key, neighbour)
            (("y", x, y), (x, y - 1)),
            (("x", x + 1, y), (x + 1, y)),
            (("y", x, y + 1), (x, y + 1)),
            (("x", x, y), (x - 1, y)),
        ]
        out = []
        for key, m in sides:
            v = self.e6.get(key, 0) & 7
            if self.inb(*m):
                M = self.at(*m)
                if v == 7:
                    if self.cmb(C, M) == 1:
                        v = 0
                elif self.cmb(M, C) == 1:
                    v |= 7
            out.append(v)
        return out  # N, E, S, W fields; the arm is entered when all are 0

    def entered(self):
        """The listed cells whose shift-6 word is empty after check_blocks."""
        hits = []
        for idx in self.trans:
            x, y = idx % self.xs, idx // self.xs
            if not any(self.word_after_blocks(x, y)):
                hits.append((x, y))
        return hits


# -- the reader --------------------------------------------------


_LAND_KEY = {"BASELAND": 0, "SANDY": 1, "OCEAN": 2, "NONE": 3}


def read_world(f, first_line):
    """f positioned on the 'BEGIN WORLD' line (already read): the grid."""
    indent = len(first_line) - len(first_line.lstrip())
    xs = ys = seed = None
    cells = []
    cur = None
    for raw in f:
        line = raw.decode("latin-1") if isinstance(raw, bytes) else raw
        body = line.strip()
        ind = len(line) - len(line.lstrip())
        if body and ind <= indent:
            break
        if ind == indent + 1:
            if body in _LAND_KEY:
                cur = [_LAND_KEY[body], None]
                cells.append(cur)
            elif body.startswith("land_sub ") and cur is not None:
                cur[1] = int(body.split()[1])
            elif body.startswith("xs ") and xs is None:
                xs = int(body.split()[1])
            elif body.startswith("ys ") and ys is None:
                ys = int(body.split()[1])
            elif body.startswith("seed ") and seed is None:
                seed = int(body.split()[1])
    return xs, ys, seed, [tuple(c) for c in cells]




def first_grid(path, cap=6_000_000):
    """The first WORLD block with cells, and the lobby before it."""
    info = {"players": 0, "frame": None}
    gi = None
    try:
        with open(path, "rb") as f:
            for n, raw in enumerate(f):
                if n > cap:
                    break
                s = raw.decode("latin-1")
                b = s.strip()
                if b.startswith("BEGIN FRAME"):
                    info["frame"] = b.split()[-1]
                if b == "BEGIN GAMEINFO" and "MAP_STYLE" not in info:
                    gi = len(s) - len(s.lstrip())
                    continue
                if gi is not None:
                    if b and len(s) - len(s.lstrip()) <= gi:
                        gi = None
                    else:
                        if b == "BEGIN PLAYER":
                            info["players"] += 1
                        for k in ("MAP_STYLE", "MAP_SIZE"):
                            if b.startswith(k + " ") and k not in info:
                                info[k] = b.split(None, 1)[1]
                if b == "BEGIN WORLD":
                    xs, ys, _, cells = read_world(f, s)
                    if xs and len(cells) == xs * ys:
                        return info, (xs, ys, cells)
    except OSError:
        pass
    return info, None


def _scan_one(name):
    return name, *first_grid(os.path.join(ARCHIVE, name))


def _plant_all(args):
    key, xs, ys, cells = args
    out = {}
    for cb in (3, 0):
        hits = []
        for y in range(ys):
            for x in range(xs):
                r = Remark(cells, xs, ys, Combo(cb))
                if r.spot_update_land(x, y, BASELAND, 3) and r.entered():
                    hits.append(((x, y), r.entered()))
        out[cb] = hits
    return key, out


def scan(style=None):
    names = sorted(n for n in os.listdir(ARCHIVE) if n.startswith("gamelog") and n.endswith(".txt"))
    with Pool(4) as pool:
        got = pool.map(_scan_one, names, chunksize=4)
    by = collections.OrderedDict()
    for name, info, g in got:
        if g is None or (style is not None and info.get("MAP_STYLE") != str(style)):
            continue
        h = hashlib.sha1(repr(g).encode()).hexdigest()[:12]
        by.setdefault(h, [g, []])[1].append((name, info))
    with Pool(4) as pool:
        res = pool.map(_plant_all, [(h, *v[0]) for h, v in by.items()])
    total = 0
    for h, out in sorted(res, key=lambda r: by[r[0]][1][0][1].get("MAP_STYLE", "")):
        (xs, ys, _), files = by[h]
        info = files[0][1]
        hits = sorted(p for p, _ in out[3])
        total += len(hits)
        same = "" if hits == sorted(p for p, _ in out[0]) else f" (COAST_BASE 0: {sorted(p for p, _ in out[0])})"
        print(f"{h} style {info.get('MAP_STYLE')} players {info['players']} {xs}x{ys} "
              f"files {len(files)} ({files[0][0]}): {len(hits)} passing {hits}{same}")
    print(f"{len(by)} distinct grids over {sum(len(v[1]) for v in by.values())} files; {total} passing plants")


def printed_writes(path):
    import re
    with open(path, "rb") as f:
        for raw in f:
            s = raw.decode("latin-1")
            if "Set edge" in s:
                return [(m[0], int(m[1]), int(m[2]), int(m[3])) for m in
                        re.findall(r"Set edge (\w) \((\d+),(\d+)\) with flags (\d+)", s)]
    return []


def cell(path, plants, sets, printed=None):
    info, g = first_grid(path if os.path.isabs(path) else os.path.join(ARCHIVE, path))
    if g is None:
        sys.exit(f"{path}: no WORLD block with cells")
    xs, ys, cells = g
    cells = list(cells)
    for x, y, land, sub in sets:
        cells[y * xs + x] = (land, sub)
    names = "BSON"
    r = None
    for x, y in plants:
        r = Remark(cells, xs, ys, Combo(3))
        r.spot_update_land(x, y, BASELAND, 3)
        for (a, b), v in r.g.items():
            cells[b * xs + a] = v
        print(f"plant ({x}, {y}): changed {sorted(r.g.items())}")
        for i in r.trans:
            w = r.word_after_blocks(i % xs, i // xs)
            print(f"  listed ({i % xs}, {i // xs}) N E S W after check_blocks {w}")
        print(f"  enters check_for_diagonal at {r.entered()}; {len(r.log)} Set edge writes")
    if printed:
        want = printed_writes(printed)
        same = want == r.log
        print(f"printed {len(want)} writes, predicted {len(r.log)}: {'equal' if same else 'DIFFER'}")
        for i, (a, b) in enumerate(zip(r.log, want)):
            if a != b:
                print(f"  first difference at write {i + 1}: predicted {a}, printed {b}")
                break
        sys.exit(0 if same else 1)


if __name__ == "__main__":
    a = sys.argv[1:]
    if not a or a[0] not in ("scan", "cell"):
        sys.exit(__doc__)
    if a[0] == "scan":
        scan(int(a[a.index("--style") + 1]) if "--style" in a else None)
    else:
        plants, sets, printed, i = [], [], None, 2
        while i < len(a):
            if a[i] == "--set":
                sets.append(tuple(int(v) for v in a[i + 1].split(","))); i += 2
            elif a[i] == "--printed":
                printed = a[i + 1]; i += 2
            else:
                plants.append(tuple(int(v) for v in a[i].split(","))); i += 1
        cell(a[1], plants, sets, printed)
