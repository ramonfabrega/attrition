#!/usr/bin/env python3
"""Reproduce a retained run69 discrepancy without launching the original game.

Usage: python3 tools/viewer/lab_demo.py INSTALL LOG_DIRECTORY NEW_OUTPUT_DIRECTORY
Generated capture-derived artifacts must stay outside the repository.
"""
import hashlib
import json
import subprocess
import sys
import time
from pathlib import Path
from verify_export import read, verify

SOURCES = ['gamelog-run69-greatlakes-3k.txt', 'rontrace-run69.log',
           'gamelog-run11-checksum.txt', 'gamelog-run3-fulldump-types.txt',
           'gamelog-run12-dumpall-seeds.txt', 'gamelog-run13-window-95-105.txt']


def digest(path):
    with Path(path).open('rb') as source:
        return hashlib.file_digest(source, 'sha256').hexdigest()


def matching_frames(broad, focused):
    verify(broad)
    verify(focused)
    by_index = {f['index']: f for f in broad['frames']}
    if any(by_index.get(f['index']) != f for f in focused['frames']):
        raise ValueError('focused replay changed a complete frame record')


def witness(data):
    verify(data)
    for frame in data['frames']:
        for row in frame['differences']:
            if row['entity'] == 'city:0/2000' and row['field'] == 'peasant_dist':
                if row['original'] == row['rust']:
                    raise ValueError('witness values agree')
                return {'frame': frame['n'], 'source_index': frame['index'], **row}
    raise ValueError('expected city-field discrepancy no longer exists; investigate, do not invent it')



def verify_checkpoint_windows(broad, focused, restored_windows):
    if len(restored_windows) != 3:
        raise ValueError('expected three checkpoint windows')
    if restored_windows[0]['frames'] != focused['frames']:
        raise ValueError('checkpoint window differs from uninterrupted focused replay')
    first = focused['frames'][0]['index']
    for i, restored in enumerate(restored_windows):
        matching_frames(broad, restored)
        if len(restored['frames']) != 3 or restored['frames'][0]['index'] != first + i * 3:
            raise ValueError('checkpoint window has wrong global indices or length')


def main(install, logs, destination):
    repo = Path(__file__).resolve().parents[2]
    output = Path(destination).resolve()
    if output.is_relative_to(repo):
        raise ValueError('capture-derived output must be outside the repository')
    paths = [Path(logs).resolve() / name for name in SOURCES]
    before = {str(p): digest(p) for p in paths}
    output.mkdir(parents=True, exist_ok=False)
    with (output / 'build.log').open('w') as log:
        subprocess.run(['cargo', 'build', '-p', 'rondata', '--release', '--example', 'debug_view',
                        '--target-dir', str(repo / 'target')],
                       cwd=repo, stdout=log, stderr=subprocess.STDOUT, check=True)
    binary = repo / 'target/release/examples/debug_view'
    binary_hash = digest(binary)
    measurements = []

    def export(name, reader, start, count, windows=1):
        target = output / (name + '.html')
        cmd = [str(binary), str(Path(install).resolve()), str(paths[0]), str(target),
               '--reader', reader, '--from', str(start), '--count', str(count), '--trace', str(paths[1])]
        if windows != 1:
            cmd += ['--windows', str(windows)]
        for sibling in paths[2:]:
            cmd += ['--sibling', str(sibling)]
        begin = time.perf_counter()
        with (output / (name + '.log')).open('w') as log:
            subprocess.run(cmd, cwd=repo, stdout=log, stderr=subprocess.STDOUT, check=True)
        data = read(target)
        verify(data)
        if len(data['frames']) != count:
            raise ValueError('exported window is shorter than requested')
        measurements.append({'name': name, 'seconds': time.perf_counter() - begin,
                             'bytes': target.stat().st_size, 'command': cmd})
        return data

    broad = export('broad-indexed', 'indexed', 1900, 130)
    memory = export('broad-memory', 'memory', 1900, 130)
    matching_frames(broad, memory)
    if len(broad['frames']) != len(memory['frames']):
        raise ValueError('reader window lengths differ')
    for key in ['notes', 'world', 'seedInputs', 'guyInputs', 'applied']:
        if broad[key] != memory[key]:
            raise ValueError('reader provenance differs: ' + key)
    found = witness(broad)
    focused = export('focused', 'indexed', found['frame'], 3)
    matching_frames(broad, focused)
    if witness(focused) != found:
        raise ValueError('focused replay changed the witness')
    export('checkpoint', 'checkpoint', found['frame'], 3, windows=3)
    restored_windows = [read(output / ('checkpoint.html' if i == 0 else f'checkpoint.window-{i}.html'))
                        for i in range(3)]
    verify_checkpoint_windows(broad, focused, restored_windows)
    after = {str(p): digest(p) for p in paths}
    if digest(binary) != binary_hash:
        raise ValueError('exporter binary changed during demonstration')
    if before != after:
        raise ValueError('source input changed during demonstration')
    report = {'schema': 1, 'witness': found, 'inputs_sha256': before,
              'binary_sha256': binary_hash, 'install': str(Path(install).resolve()),
              'install_content_bound': False, 'measurements': measurements,
              'checkpoint_windows_verified': 3,
              'reduction': {'broad_records': len(broad['frames']), 'focused_records': len(focused['frames']),
                            'standalone_replay': False,
                            'blockers': ['The in-memory checkpoint is not serialized; a new process still starts at setup.',
                                         'Full captures supply correction observations and lookup data.',
                                         'The same external install is required; its contents are not bundled or hashed.']}}
    (output / 'result.json').write_text(json.dumps(report, indent=2) + '\n')
    (output / 'README.txt').write_text(
        'Open focused.html. In Field differences, filter for peasant_dist.\n'
        'result.json contains the exact witness, input hashes and replay commands.\n'
        'This is a reduced diagnostic window, NOT an independently resumable replay.\n'
        'Existing comparator residue is not a new acceptance failure or fidelity advance.\n')
    print(json.dumps({'output': str(output), 'witness': found, 'standalone_replay': False}, indent=2))


if __name__ == '__main__':
    if len(sys.argv) != 4:
        sys.exit(__doc__)
    main(*sys.argv[1:])
