#!/usr/bin/env python3
"""patch_exe.py <in.exe> <out.exe> <dll name> — add one import to a PE32.

Writes a copy of the executable with a new section (`.rtrace`, read/write)
holding a copy of the import directory table plus one more descriptor, whose
DLL is loaded — and its DllMain run — by the loader before the program's
entry point. The original file is not touched; the copy is what the trace
runs launch. Everything else in the image (code, data, relocations, the
original ILT/IATs the copied descriptors point at) is byte-identical, so the
PDB's addresses still hold, and `GetModuleHandle(NULL)` is the same base.

The descriptor imports one symbol by name (`Hook`), which the DLL exports.
"""
import struct
import sys


def rd32(b, o):
    return struct.unpack_from("<I", b, o)[0]


def rd16(b, o):
    return struct.unpack_from("<H", b, o)[0]


def align(v, a):
    return (v + a - 1) // a * a


src, dst, dll = sys.argv[1], sys.argv[2], sys.argv[3]
img = bytearray(open(src, "rb").read())

pe = rd32(img, 0x3C)
assert img[pe:pe + 4] == b"PE\0\0"
nsec = rd16(img, pe + 6)
optsz = rd16(img, pe + 20)
opt = pe + 24
assert rd16(img, opt) == 0x10B, "PE32 only"
sec_align = rd32(img, opt + 32)
file_align = rd32(img, opt + 36)
size_of_image = rd32(img, opt + 56)
size_of_headers = rd32(img, opt + 60)
ndirs = rd32(img, opt + 92)
dirs = opt + 96
imp_rva, imp_size = rd32(img, dirs + 8), rd32(img, dirs + 12)
sect = opt + optsz

secs = []
for i in range(nsec):
    o = sect + i * 40
    secs.append(dict(off=o, name=img[o:o + 8].rstrip(b"\0").decode(), vsize=rd32(img, o + 8),
                     va=rd32(img, o + 12), rawsize=rd32(img, o + 16), raw=rd32(img, o + 20),
                     chars=rd32(img, o + 36)))


def rva_to_off(rva):
    for s in secs:
        if s["va"] <= rva < s["va"] + max(s["vsize"], s["rawsize"]):
            return s["raw"] + (rva - s["va"])
    raise ValueError(hex(rva))


# the existing descriptors, up to the null one
old = []
o = rva_to_off(imp_rva)
while True:
    d = img[o:o + 20]
    if d == b"\0" * 20:
        break
    old.append(bytes(d))
    o += 20

# room for one more section header
hdr_end = sect + (nsec + 1) * 40
assert hdr_end <= size_of_headers, "no room for a section header"
assert img[sect + nsec * 40:hdr_end] == b"\0" * 40

new_va = align(max(s["va"] + s["vsize"] for s in secs), sec_align)
new_raw = align(len(img), file_align)

# layout of the new section: IDT | ILT | IAT | hint/name | dll name
n_desc = len(old) + 2
idt_size = n_desc * 20
ilt_rva = new_va + idt_size
iat_rva = ilt_rva + 8
hint_rva = iat_rva + 8
name_rva = hint_rva + 2 + len(b"Hook\0")
name_rva = align(name_rva, 2)
blob = bytearray()
for d in old:
    blob += d
blob += struct.pack("<IIIII", ilt_rva, 0, 0, name_rva, iat_rva)
blob += b"\0" * 20
blob += struct.pack("<II", hint_rva, 0)  # ILT
blob += struct.pack("<II", hint_rva, 0)  # IAT (the loader overwrites it)
blob += b"\0\0" + b"Hook\0"
while len(blob) < name_rva - new_va:
    blob += b"\0"
blob += dll.encode() + b"\0"
vsize = len(blob)
rawsize = align(vsize, file_align)
blob += b"\0" * (rawsize - vsize)

# section header
hdr = struct.pack("<8sIIIIIIHHI", b".rtrace", vsize, new_va, rawsize, new_raw, 0, 0, 0, 0,
                  0xC0000040)  # INITIALIZED_DATA | READ | WRITE
img[sect + nsec * 40:sect + nsec * 40 + 40] = hdr
struct.pack_into("<H", img, pe + 6, nsec + 1)
struct.pack_into("<I", img, opt + 56, align(new_va + vsize, sec_align))
struct.pack_into("<II", img, dirs + 8, new_va, idt_size)
# the bound-import directory would pre-resolve the old descriptors; it is empty here
assert rd32(img, dirs + 11 * 8) == 0
# the PE checksum is not verified by Wine; zero it rather than leave a stale one
struct.pack_into("<I", img, opt + 64, 0)

img += b"\0" * (new_raw - len(img))
img += blob
open(dst, "wb").write(img)
print(f"{dst}: +.rtrace at rva {new_va:#x} raw {new_raw:#x}, {len(old)}+1 imports, "
      f"SizeOfImage {size_of_image:#x} -> {rd32(img, opt + 56):#x}")
