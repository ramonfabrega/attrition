#!/usr/bin/env python3
"""Compare run69 figure-correction policies from one owned checkpoint.
Usage: python3 tools/viewer/branch_experiment.py INSTALL LOGS NEW_OUTPUT_DIRECTORY [figures|clocks]
"""
import json
from pathlib import Path
import subprocess
import sys
import time
from lab_demo import SOURCES, digest, matching_frames
from verify_export import read, verify


def compare(control, treatment):
    verify(control)
    verify(treatment)
    if len(control['frames']) != len(treatment['frames']):
        raise ValueError('branch window lengths differ')
    first = None
    for a, b in zip(control['frames'], treatment['frames']):
        if (a['index'], a['n']) != (b['index'], b['n']):
            raise ValueError('branches compare different source records')
        def originals(frame):
            return [(u['id'], u['original'], u['originalRecord'], u['originalPath'])
                    for u in frame['units'] if u['original'] is not None]
        if originals(a) != originals(b):
            raise ValueError('original records changed across branches')
        if a != b and first is None:
            first = dict(frame=a['n'], source_index=a['index'],
                         changed_keys=[k for k in sorted(set(a) | set(b)) if a.get(k) != b.get(k)],
                         control_differences=a['differences'], treatment_differences=b['differences'])
            left = {u['id']: u for u in a['units']}
            right = {u['id']: u for u in b['units']}
            first['unit_changes'] = [dict(id=identity, fields={
                key: dict(control=left.get(identity, {}).get(key), treatment=right.get(identity, {}).get(key))
                for key in sorted(set(left.get(identity, {})) | set(right.get(identity, {})))
                if left.get(identity, {}).get(key) != right.get(identity, {}).get(key)
            }) for identity in sorted(set(left) | set(right)) if left.get(identity) != right.get(identity)]
    return dict(records=len(control['frames']), first_changed_record=first,
                all_exported_records_equal=first is None)


def main(install, logs, destination, intervention='figures'):
    if intervention not in ('figures', 'clocks'):
        raise ValueError('intervention must be figures or clocks')
    repo = Path(__file__).resolve().parents[2]
    output = Path(destination).resolve()
    if output.is_relative_to(repo):
        raise ValueError('capture-derived output must stay outside the repository')
    paths = [Path(logs).resolve() / name for name in SOURCES]
    before = {str(p): digest(p) for p in paths}
    output.mkdir(parents=True, exist_ok=False)
    with (output / 'build.log').open('w') as log:
        subprocess.run(['cargo', 'build', '-p', 'rondata', '--release', '--example', 'debug_view',
                        '--target-dir', str(repo / 'target')], cwd=repo, stdout=log,
                       stderr=subprocess.STDOUT, check=True)
    binary = repo / 'target/release/examples/debug_view'
    binary_hash = digest(binary)
    commands = []
    for name, reader, policy in [('reference', 'indexed', 'standard'),
                                 ('control', 'checkpoint', 'compare-' + intervention)]:
        cmd = [str(binary), str(Path(install).resolve()), str(paths[0]), str(output / (name + '.html')),
               '--reader', reader, '--corrections', policy, '--from', '95', '--count', '130',
               '--trace', str(paths[1])]
        for path in paths[2:]:
            cmd += ['--sibling', str(path)]
        start = time.perf_counter()
        with (output / (name + '.log')).open('w') as log:
            subprocess.run(cmd, cwd=repo, stdout=log, stderr=subprocess.STDOUT, check=True)
        commands.append(dict(command=cmd, seconds=time.perf_counter()-start))
    reference = read(output / 'reference.html')
    control = read(output / 'control.html')
    treatment = read(output / ('control.without-future-' + intervention + '.html'))
    matching_frames(reference, control)
    if len(reference['frames']) != 130 or len(control['frames']) != 130:
        raise ValueError('reference or control is truncated')
    if reference['frames'][0]['n'] != 95 or reference['frames'][-1]['n'] != 224:
        raise ValueError('unexpected retained capture frame range')
    interventions = [n for n in treatment['notes'] if n.startswith('LAB INTERVENTION:')]
    if len(interventions) != 1 or control['seedInputs'] != treatment['seedInputs']:
        raise ValueError('intervention metadata or seed-policy preservation failed')
    result = compare(control, treatment)
    if before != {str(p): digest(p) for p in paths} or binary_hash != digest(binary):
        raise ValueError('input or exporter changed during experiment')
    result.update(schema=1, intervention=interventions[0], inputs_sha256=before,
                  binary_sha256=binary_hash, commands=commands, install_content_bound=False,
                  conclusion_scope='Only exported state in frames 95–224; not universal or hidden-state equivalence.')
    (output / 'result.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps({k: result[k] for k in ['intervention', 'records', 'first_changed_record', 'all_exported_records_equal']}, indent=2))


if __name__ == '__main__':
    if len(sys.argv) not in (4, 5):
        sys.exit(__doc__)
    main(*sys.argv[1:])
