"""image.py — the executable, its imports, and its function table.

Everything here is read from the user's install on demand; nothing it
produces enters the repository. The function table is the linker map's `f`
symbols (every function, with its decorated name), and — when a `cache_dir`
is given — the PDB's procedure records too (the ones with private symbols,
with their code size). `llvm-pdbutil` is run once per install and its dump
cached in that directory, which the caller keeps out of git: `lift.py`'s
default is `target/recomp/`.
"""
import os
import re
import struct
import subprocess

PDBUTIL = "/opt/homebrew/opt/llvm/bin/llvm-pdbutil"


class Image:
    def __init__(self, path):
        self.path = path
        b = open(path, "rb").read()
        self.raw = b
        pe = struct.unpack_from("<I", b, 0x3C)[0]
        assert b[pe:pe + 4] == b"PE\0\0", "not a PE image"
        nsec = struct.unpack_from("<H", b, pe + 6)[0]
        opt_size = struct.unpack_from("<H", b, pe + 20)[0]
        opt = pe + 24
        assert struct.unpack_from("<H", b, opt)[0] == 0x10B, "not PE32"
        self.base = struct.unpack_from("<I", b, opt + 28)[0]
        self.size = struct.unpack_from("<I", b, opt + 56)[0]
        self.entry = self.base + struct.unpack_from("<I", b, opt + 16)[0]
        ndirs = struct.unpack_from("<I", b, opt + 92)[0]
        self.dirs = [struct.unpack_from("<II", b, opt + 96 + 8 * i) for i in range(ndirs)]
        self.sections = []
        for i in range(nsec):
            h = opt + opt_size + 40 * i
            name = b[h:h + 8].rstrip(b"\0").decode()
            vsize, va, rawsize, rawoff, = struct.unpack_from("<IIII", b, h + 8)
            chars = struct.unpack_from("<I", b, h + 36)[0]
            self.sections.append((name, self.base + va, vsize, rawoff, rawsize, chars))
        self.text = next(s for s in self.sections if s[0] == ".text")

    def is_code(self, va):
        name, sva, vsize, *_ = self.text
        return sva <= va < sva + vsize

    def is_mapped(self, va):
        return any(s[1] <= va < s[1] + max(s[2], s[4]) for s in self.sections)

    def read(self, va, n):
        for name, sva, vsize, rawoff, rawsize, _ in self.sections:
            if sva <= va < sva + max(vsize, rawsize):
                off = va - sva
                data = self.raw[rawoff + off:rawoff + min(rawsize, off + n)]
                return data + b"\0" * (n - len(data))
        raise KeyError(f"{va:08x} is not in the image")

    def u32(self, va):
        return struct.unpack("<I", self.read(va, 4))[0]

    def u8(self, va):
        return self.read(va, 1)[0]

    def imports(self):
        """[(iat_va, dll, name)] from the import directory."""
        rva, _ = self.dirs[1]
        out = []
        d = self.base + rva
        while True:
            oft, _, _, name_rva, ft = struct.unpack("<IIIII", self.read(d, 20))
            if not ft:
                break
            dll = self._cstr(self.base + name_rva)
            src = oft or ft
            i = 0
            while True:
                ent = self.u32(self.base + src + 4 * i)
                if not ent:
                    break
                name = f"#{ent & 0xffff}" if ent & 0x80000000 else self._cstr(self.base + ent + 2)
                out.append((self.base + ft + 4 * i, dll, name))
                i += 1
            d += 20
        return out

    def _cstr(self, va):
        s = b""
        while True:
            c = self.read(va, 64)
            z = c.find(b"\0")
            if z >= 0:
                return (s + c[:z]).decode("latin-1")
            s += c
            va += 64


MAP_LINE = re.compile(r"^\s*[0-9a-f]{4}:[0-9a-f]{8}\s+(\S+)\s+([0-9a-f]{8})\s+f\s", re.I)
PROC = re.compile(r"S_[GL]PROC32 \[size = \d+\] `(.*)`\s*\n\s*parent = \d+, end = \d+, addr = (\d{4}):(\d+), code size = (\d+)")
THUNK = re.compile(r"S_THUNK32 \[size = \d+\] `(.*)`\s*\n.*\n\s*kind = \w+, size = (\d+), addr = (\d{4}):(\d+)")


class Functions:
    """Entry VA -> (name, size or None)."""

    def __init__(self, image, install, cache_dir=None):
        self.by_va = {}
        mp = os.path.join(install, "sbl", "rise_z.map")
        if os.path.exists(mp):
            for line in open(mp, encoding="latin-1"):
                m = MAP_LINE.match(line)
                if m:
                    va = int(m.group(2), 16)
                    if image.is_code(va):
                        self.by_va.setdefault(va, [m.group(1), None])
        dump = os.path.join(cache_dir, "pdb-symbols.txt") if cache_dir else None
        pdb = os.path.join(install, "sbl", "rise.pdb")
        if dump and not os.path.exists(dump) and os.path.exists(pdb) and os.path.exists(PDBUTIL):
            os.makedirs(cache_dir, exist_ok=True)
            with open(dump, "w") as f:
                subprocess.run([PDBUTIL, "dump", "--symbols", pdb], stdout=f, check=True)
        if dump and os.path.exists(dump):
            text = open(dump, encoding="latin-1").read()
            secs = {i + 1: s[1] for i, s in enumerate(image.sections)}
            for m in PROC.finditer(text):
                va = secs[int(m.group(2))] + int(m.group(3))
                ent = self.by_va.setdefault(va, [m.group(1), None])
                ent[0] = m.group(1)
                ent[1] = int(m.group(4))
            for m in THUNK.finditer(text):
                va = secs[int(m.group(3))] + int(m.group(4))
                ent = self.by_va.setdefault(va, [m.group(1), None])
                ent[1] = ent[1] or int(m.group(2))
        self.sorted = sorted(self.by_va)

    def name(self, va):
        e = self.by_va.get(va)
        return e[0] if e else f"sub_{va:08x}"

    def size(self, va):
        e = self.by_va.get(va)
        return e[1] if e else None

    def next_after(self, va):
        import bisect
        i = bisect.bisect_right(self.sorted, va)
        return self.sorted[i] if i < len(self.sorted) else None
