#!/usr/bin/env python3
"""The sweep cache's receipts: list them, or `--verify` them against disk.

    python3 tools/emu_receipts.py [--cache DIR] [--emu-dir DIR] [--exe PATH] [--verify]

`crates/sim/src/sweep/support.rs`'s `emu_rows` caches each run of a
`tools/emu/sweep_*.py` script as `<key>.rows` under
`~/Library/Caches/attrition/emu-cache/` (or `$RON_EMU_CACHE_DIR`; `target/emu-cache/` with no `HOME`): a `#` header carrying the receipt, then the rows
verbatim. The key is the SHA-256 of a text naming the executable's hash,
the script's hash, each shared module's name and hash (every `tools/emu/*.py`
that is not a `sweep_*.py`, plus the `sweep_*.py` the script imports,
transitively) and the arguments — no path. This tool rebuilds that text
exactly as the Rust does (`docs/EMULATOR.md` §1.1).

Listing prints one row per receipt. `--verify` re-hashes every input the
receipt names as it is on disk now — the executable at its recorded path
(or `--exe`), the script and the modules under `--emu-dir` — re-derives the
module set, and checks the header's key against its own hashes; a stale
entry is named with its reason and the exit is 1. Nothing is deleted: a
stale entry is never read again (its key no longer matches), and the cache
is `target/`'s to clear.
"""
import argparse
import hashlib
import os
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
MAGIC = 'emu-cache v1'


def sha256_file(path, _memo={}):
    """A file's hex SHA-256, remembered by path, size and mtime."""
    st = os.stat(path)
    at = (str(path), st.st_size, st.st_mtime_ns)
    if at not in _memo:
        h = hashlib.sha256()
        with open(path, 'rb') as f:
            for chunk in iter(lambda: f.read(1 << 20), b''):
                h.update(chunk)
        _memo[at] = h.hexdigest()
    return _memo[at]


def imported(text):
    """The top-level module names a Python text's import lines name — the
    same reading as `support.rs`'s `imported`."""
    out = []
    for line in text.splitlines():
        line = line.lstrip().split('#', 1)[0]
        if line.startswith('from '):
            words = line[5:].split()
            if words:
                out.append(words[0].split('.')[0])
        elif line.startswith('import '):
            for part in line[7:].split(','):
                words = part.split()
                if words:
                    out.append(words[0].split('.')[0])
    return [m for m in out if m]


def shared_modules(emu_dir, script):
    """The module file names a script's rows can depend on, sorted."""
    emu_dir = Path(emu_dir)
    found = {p.name for p in emu_dir.glob('*.py') if not p.name.startswith('sweep_')}
    todo = sorted(found) + [script]
    seen = set(todo)
    while todo:
        text = (emu_dir / todo.pop()).read_text(errors='replace')
        for module in imported(text):
            name = f'{module}.py'
            if module.startswith('sweep_') and (emu_dir / name).is_file() and name not in seen:
                seen.add(name)
                found.add(name)
                todo.append(name)
    found.discard(script)
    return sorted(found)


def key_text(exe_sha, script_sha, modules, args=()):
    """The text whose SHA-256 is the key: `support.rs`'s `cache_key`."""
    text = f'{MAGIC}\nexecutable {exe_sha}\nscript {script_sha}\n'
    text += ''.join(f'module {name} {sha}\n' for name, sha in modules)
    text += ''.join(f'arg {a}\n' for a in args)
    return text


def key_of(exe_sha, script_sha, modules, args=()):
    return hashlib.sha256(key_text(exe_sha, script_sha, modules, args).encode()).hexdigest()


def key_for(emu_dir, exe, script, args=()):
    """`(key, exe_sha, script_sha, [(module, sha)])` of a run as it would
    be keyed now."""
    emu_dir = Path(emu_dir)
    exe_sha = sha256_file(exe)
    script_sha = sha256_file(emu_dir / script)
    modules = [(m, sha256_file(emu_dir / m)) for m in shared_modules(emu_dir, script)]
    return key_of(exe_sha, script_sha, modules, args), exe_sha, script_sha, modules


def parse(path):
    """A cache file's receipt as a dict, or None when it has no header."""
    entry = {'file': Path(path), 'modules': [], 'args': []}
    with open(path, errors='replace') as f:
        if f.readline().rstrip('\n') != f'# {MAGIC}':
            return None
        for line in f:
            line = line.rstrip('\n')
            if not line.startswith('# '):
                return None
            if line == '# end':
                return entry
            field, _, rest = line[2:].partition(' ')
            if field in ('executable', 'script', 'module'):
                sha, _, name = rest.partition(' ')
                if field == 'module':
                    entry['modules'].append((name, sha))
                else:
                    entry[field] = (sha, name)
            elif field == 'arg':
                entry['args'].append(rest)
            else:
                entry[field] = rest
    return None


def check(entry, emu_dir, exe=None):
    """The reasons an entry is stale against the inputs on disk now; empty
    when it is fresh."""
    reasons = []
    emu_dir = Path(emu_dir)
    exe_sha, exe_path = entry.get('executable', ('', ''))
    script_sha, script = entry.get('script', ('', ''))
    own = key_of(exe_sha, script_sha, entry['modules'], entry['args'])
    if entry.get('key') != own or entry['file'].stem != own:
        reasons.append(f"the header's key is not its own hashes' ({own[:12]})")
    exe = Path(exe or exe_path)
    if not exe.is_file():
        reasons.append(f'executable missing: {exe}')
    elif sha256_file(exe) != exe_sha:
        reasons.append(f'executable changed: {exe}')
    if not (emu_dir / script).is_file():
        reasons.append(f'script missing: {script}')
        return reasons
    if sha256_file(emu_dir / script) != script_sha:
        reasons.append(f'script changed: {script}')
    recorded = dict(entry['modules'])
    now = shared_modules(emu_dir, script)
    for name in sorted(set(recorded) - set(now)):
        reasons.append(f'module gone: {name}')
    for name in now:
        if name not in recorded:
            reasons.append(f'module new: {name}')
        elif sha256_file(emu_dir / name) != recorded[name]:
            reasons.append(f'module changed: {name}')
    return reasons


def default_cache():
    if os.environ.get('RON_EMU_CACHE_DIR'):
        return Path(os.environ['RON_EMU_CACHE_DIR'])
    home = os.environ.get('HOME')
    return Path(home, 'Library/Caches/attrition/emu-cache') if home else ROOT / 'target/emu-cache'


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split('\n\n')[0])
    ap.add_argument('--cache', type=Path, default=None, help='the cache directory')
    ap.add_argument('--emu-dir', type=Path, default=ROOT / 'tools/emu')
    ap.add_argument('--exe', type=Path, default=None,
                    help="the executable to verify against (default: each receipt's own path)")
    ap.add_argument('--verify', action='store_true',
                    help="re-hash every receipt's inputs; exit 1 naming each stale entry")
    a = ap.parse_args(argv)
    cache = a.cache or default_cache()
    files = sorted(cache.glob('*.rows')) if cache.is_dir() else []
    entries = []
    for f in files:
        e = parse(f)
        if e is None:
            print(f'{f.name}: not a receipt (no header)')
        entries.append((f, e))
    if not entries:
        print(f'no receipts in {cache}')
        return 0
    print(f"{'key':<12}  {'rows':>6}  {'date':<20}  {'exe':<12}  {'mods':>4}  script")
    for f, e in entries:
        if e is None:
            continue
        print(f"{e.get('key', '?')[:12]:<12}  {e.get('rows', '?'):>6}  {e.get('date', '?'):<20}  "
              f"{e.get('executable', ('?',))[0][:12]:<12}  {len(e['modules']):>4}  "
              f"{e.get('script', ('', '?'))[1]}")
    if not a.verify:
        return 0
    stale = 0
    for f, e in entries:
        reasons = ['no header'] if e is None else check(e, a.emu_dir, a.exe)
        if reasons:
            stale += 1
            print(f'STALE {f.name}: ' + '; '.join(reasons))
    print(f'{len(entries) - stale} of {len(entries)} receipts match the inputs on disk')
    return 1 if stale else 0


if __name__ == '__main__':
    sys.exit(main())
