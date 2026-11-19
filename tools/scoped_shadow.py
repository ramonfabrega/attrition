"""Predict one scoped test from a prior observation, then always run and score it."""
import argparse
from collections import Counter
import hashlib
import json
import os
from pathlib import Path
import platform
import stat
import time
from typing import NamedTuple

from gate_profile import test_requests

AGES = 'free::ages::the_draw_list_across_the_ages_resolves'
FORECAST = 'ages-forecast.json'
STABILITY = 'ages-stability.json'


class Stamp(NamedTuple):
    device: int
    inode: int
    size: int
    modified: int
    changed: int
    mode: int


def digest(value):
    return hashlib.sha256(json.dumps(value, sort_keys=True).encode()).hexdigest()


def requests(report):
    rows = test_requests(report / 'fixture-requests', [AGES])
    names = Counter()
    for row, count in rows.items():
        present, encoded, _test = row.split(b'\t')
        name = bytes.fromhex(encoded.decode()).decode()
        if present != b'1' or Path(name).name != name or name in ('', '.', '..'):
            raise ValueError('missing or non-local fixture')
        names[name] += count
    if not names:
        raise ValueError('no audited ages inputs')
    return names


def files_below(root):
    """Include the whole tree. Refuse unreadable or linked directories."""
    def failed(error):
        raise error
    for directory, children, files in os.walk(root, onerror=failed):
        yield Path(directory)
        for child in children:
            if (Path(directory) / child).is_symlink():
                raise ValueError(f'linked directory: {child}')
        for name in files:
            yield Path(directory) / name


def inventory(names, binary, env):
    if 'RON_INDEXED_ARCHIVE_SOURCE' in env or 'RON_INDEXED_ARCHIVE_PATH' in env:
        raise ValueError('explicit archive remapping is outside this pilot')
    install = Path(env['RON_INSTALL']).resolve()
    if not (install / 'Data/rules.xml').is_file():
        raise ValueError('missing install')
    logs = Path(env.get('RON_GAMELOG_DIR', Path(env['HOME']) /
                        'ron-data/AppData/Roaming/Microsoft Games/Rise of Nations/Logs')).resolve()
    cache = Path(env.get('RON_SETUP_OBSERVATIONS_DIR', logs / '.ron-observations-v1')).resolve()
    paths = {'binary': [binary], 'install': files_below(install),
             'setup cache': files_below(cache) if cache.exists() else [],
             'captures': [], 'indexes': []}
    for name in names:
        path = logs / name
        s = path.stat()
        paths['captures'].append(path)
        seconds, nanos = divmod(s.st_mtime_ns, 10**9)
        index = path.with_name(f'{name}.ronidx-v1-{s.st_size}-{seconds}-{nanos}')
        if index.exists():
            paths['indexes'].append(index)
    inventory = {}
    for kind, files in paths.items():
        records = {}
        for path in files:
            s = path.stat()
            if not (stat.S_ISREG(s.st_mode) or stat.S_ISDIR(s.st_mode) and kind in ('install', 'setup cache')):
                raise ValueError(f'nonregular input: {path}')
            records[str(path)] = Stamp(s.st_dev, s.st_ino, s.st_size, s.st_mtime_ns, s.st_ctime_ns, s.st_mode)
        inventory[kind] = records
    return inventory


def fingerprint(inputs, names, env):
    # Keep environment values out of the report. These two paths describe
    # the experiment's output and history, not what the Rust test reads.
    settings = {k: v for k, v in env.items() if
                (k.startswith(('RON_', 'RUST_', 'Malloc', 'DYLD_', 'LC_'))
                 or k in ('HOME', 'PATH', 'LANG', 'TZ'))
                and k not in ('RON_GATE_SHADOW_FROM', 'RON_FIXTURE_AUDIT_DIR')}
    keys = {'runtime': digest([platform.platform(), platform.machine(), settings, AGES, '--exact', '--test-threads=1']),
            'requests': digest(names), 'model': hashlib.sha256(Path(__file__).read_bytes()).hexdigest()}
    for kind, records in inputs.items():
        hashes = {}
        for path, stamp in records.items():
            if stat.S_ISDIR(stamp.mode):
                hashes[path] = 'directory'
                continue
            with open(path, 'rb') as stream:
                hashes[path] = hashlib.file_digest(stream, 'sha256').hexdigest()
        keys[kind] = digest(hashes)
    return keys


def read(directory, name):
    return json.loads((directory / name).read_text())


def write(directory, name, value):
    with (directory / name).open('x') as stream:
        json.dump(value, stream, indent=2)
        stream.write('\n')


def score(report):
    directory = report / 'isolated/rondata'
    forecast = read(directory, FORECAST)
    rows = [json.loads(line) for line in (directory / 'measurements.jsonl').read_text().splitlines()]
    actual = [row for row in rows if row['name'] == AGES]
    if len(actual) != 1:
        raise ValueError('need exactly one completed ages measurement')
    row = actual[0]
    stable = (directory / STABILITY).is_file() and read(directory, STABILITY)['stable']
    try:
        observed = digest(requests(report))
    except (OSError, ValueError):
        observed = None
    inputs_match = stable and observed is not None and forecast.get('key', {}).get('requests') == observed
    passed = bool(row['success'] and row['exit'] == 0)
    return dict(predicted_pass=forecast['predicted_pass'], actual_pass=passed,
                missed_failure=forecast['predicted_pass'] and not passed,
                comparable=bool(inputs_match), key=forecast.get('key'), reason=forecast['reason'],
                forecast_seconds=forecast['seconds'],
                test_seconds=(row.get('resource_timing') or {}).get('run_seconds'),
                test_cpu_seconds=(row['user'] or 0) + (row['system'] or 0))


def forecast(previous, binary, env):
    names = requests(previous)
    inputs = inventory(names, binary, env)
    key = fingerprint(inputs, names, env)
    if inputs != inventory(names, binary, env):
        raise ValueError('inputs changed while hashing')
    predicted, reason = False, 'seed: no prior prospective observation'
    if (previous / 'isolated/rondata' / FORECAST).is_file():
        old = score(previous)
        if old['actual_pass'] and old['comparable']:
            changed = sorted(k for k in key if key[k] != old['key'].get(k))
            predicted = not changed
            reason = 'matching inputs' if predicted else 'changed: ' + ', '.join(changed)
        else:
            reason = 'prior observation incomplete, unstable or red'
    return dict(predicted_pass=predicted, key=key, reason=reason,
                bytes_hashed=sum(s.size for group in inputs.values() for s in group.values()
                                 if stat.S_ISREG(s.mode))), inputs, names


def record(previous, directory, binary):
    started = time.monotonic()
    try:
        prediction, inputs, _ = forecast(previous, binary, os.environ)
        prediction['stamps'] = digest(inputs)
    except (OSError, ValueError, KeyError) as error:
        prediction = dict(predicted_pass=False, reason=str(error))
    prediction['seconds'] = time.monotonic() - started
    write(directory, FORECAST, prediction)


def check_stability(previous, directory, binary, env):
    prediction = read(directory, FORECAST)
    stamps = prediction.get('stamps')
    stable = stamps is not None and stamps == digest(inventory(requests(previous), binary, env))
    write(directory, STABILITY, {'stable': stable})


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest='action', required=True)
    plan = sub.add_parser('forecast', help='run by the scoped gate before the actual test')
    for arg in ('previous', 'directory', 'binary'):
        plan.add_argument(arg, type=Path)
    sub.add_parser('score').add_argument('report', type=Path)
    args = parser.parse_args()
    try:
        if args.action == 'forecast':
            record(args.previous, args.directory, args.binary)
        else:
            print(json.dumps(score(args.report), indent=2))
    except (OSError, ValueError, KeyError) as error:
        parser.exit(1, f'incomplete observation: {error}\n')
