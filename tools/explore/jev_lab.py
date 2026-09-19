#!/usr/bin/env python3
"""Small authored Jev capability probes. No game files or production integration.

Uses only the standard library. Credentials stay in process memory. Outputs
contain exact authored requests and responses, never authentication headers.
"""
import argparse
from concurrent.futures import ThreadPoolExecutor
import json
import os
from pathlib import Path
import random
import statistics
import subprocess
import time
import urllib.error
import urllib.request


def cases():
    rows = []

    def add(family, state, question, options, expected):
        rows.append(dict(id=f'{family}-{sum(r["family"] == family for r in rows):02}',
                         family=family, state=state, question=question,
                         options=options, expected=expected))

    evidence = [
        ('All 18 required fixtures were absent. The runner skipped them and exited zero.', 'unsupported'),
        ('All 18 required fixtures loaded. Every compared field agreed. The claim concerns only those fields on those fixtures.', 'supported'),
        ('All 18 required fixtures loaded. One compared coordinate differed. The claim says every compared field agreed.', 'contradicted'),
        ('A deterministic replay matched itself twice. The claim says it matches the original executable.', 'unsupported'),
        ('A function was entered in a trace. The claim says both arms of its supply-exemption branch were validated.', 'unsupported'),
        ('The profiler could not read process memory. The claim says the process used zero bytes.', 'unsupported'),
        ('A summary says PASS. Its detailed report says required=18, loaded=17, missing=1. The claim says all required fixtures were checked.', 'contradicted'),
        ('The oracle timed out before producing any answer. The claim says the candidate has a gameplay defect.', 'unsupported'),
        ('For inputs 0 through 255 inclusive, both programs returned equal outputs. The claim says they agree on those 256 inputs.', 'supported'),
        ('For inputs 0 through 255 inclusive, both programs returned equal outputs. The claim says they agree on every 32-bit input.', 'unsupported'),
        ('A faulty equality mutant failed the regression; the restored implementation passed. The claim says the regression detects that specific mutant.', 'supported'),
        ('A faulty equality mutant failed the regression; the restored implementation passed. The claim says a new production bug was discovered.', 'unsupported'),
    ]
    opts = dict(supported='The evidence establishes the stated claim within its stated scope.',
                contradicted='The evidence directly refutes the claim.',
                unsupported='The evidence neither establishes nor directly refutes the claim.')
    for state, expected in evidence:
        add('evidence', state, 'Classify the claim using only the supplied evidence. If no explicit claim is given, assess whether the required fixture comparison succeeded.', opts, expected)

    for flags in [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 15, 16, 17, 31, 128, 255]:
        add('bitmask', {'code_a': 'return (flags & 1) != 0;', 'code_b': 'return flags == 1;', 'flags': flags},
            'Do these two functions return the same boolean for the supplied flags?',
            dict(same='Equal outputs', different='Different outputs'),
            'same' if bool(flags & 1) == (flags == 1) else 'different')

    for ms in [0, 1, 66, 67, 99, 133, 199, 200, 333, 999, 1023, 65535]:
        exact = ms * 3 // 200
        add('arithmetic', {'milliseconds': ms, 'rule': 'Multiply milliseconds by 3, divide by 200, truncate toward zero. All intermediate arithmetic is exact and nonnegative.'},
            'Which integer is the result?', {str(x): None for x in sorted(set([max(0, exact - 1), exact, exact + 1, exact + 2]))}, str(exact))

    # Relabeling deliberately avoids familiar gameplay semantics.
    rng = random.Random(1926)
    for i in range(12):
        labels = rng.sample(['amber', 'birch', 'cobalt', 'delta', 'elm', 'flint'], 3)
        a, b, c = labels
        state = {'experiments': [
            {'settings': {a: 0, b: 0, c: 0}, 'outcome': 0},
            {'settings': {a: 1, b: 0, c: 0}, 'outcome': 0},
            {'settings': {a: 0, b: 1, c: 0}, 'outcome': 1}],
            'candidates': {'A': f'outcome = {b}', 'B': f'outcome = {b} OR {c}', 'C': f'outcome = {a} AND {b}'}}
        add('hypotheses', state, 'Which candidate rules agree with every observed experiment? Interpret OR and AND as Boolean operations.',
            dict(A='Only A', AB='A and B only', ABC='A, B, and C', none='None'), 'AB')

    for i in range(12):
        a, b = rng.sample(['supply', 'terrain', 'stance', 'season', 'formation'], 2)
        target = rng.randrange(2)
        candidates = [(0, 0), (0, 1), (1, 0), (1, 1)]
        rng.shuffle(candidates)
        opts = {f'run{j}': f'{a}={x}, {b}={y}' for j, (x, y) in enumerate(candidates)}
        expected = next(k for k, xy in zip(opts, candidates) if xy == (1, 0))
        add('experiment', {'rules': [f'H1: output equals {a}', f'H2: output equals {a} AND {b}'], 'target_output_for_H1': target},
            'Select the experiment on which H1 and H2 predict different outputs. The target_output_for_H1 field is irrelevant; maximize discrimination.', opts, expected)

    for i in range(12):
        hostile, supplied, exempt = bool(i & 1), bool(i & 2), bool(i & 4)
        hp = 5 if i < 8 else 1
        add('policy', {'rules': 'At end of tick, lose 2 health iff hostile AND not supplied AND not exempt. Move-safe sets hostile=false. Resupply sets supplied=true. Attack changes none of these fields. Waiting changes none. Each action takes one tick. Choose a surviving action with minimum cost.',
                       'state': dict(hostile=hostile, supplied=supplied, exempt=exempt, health=hp),
                       'costs': dict(wait=0, attack=1, resupply=2, move_safe=3)},
            'Which action should be taken?', dict(wait='Wait', attack='Attack', resupply='Resupply', move_safe='Move-safe'),
            'resupply' if hostile and not supplied and not exempt and hp <= 2 else 'wait')
    return rows


def request_for(case, permutation=0):
    options = list(case['options'].items())
    random.Random(permutation).shuffle(options)
    return {'model': 'jev-1.13.0', 'state': case['state'], 'questions': {
        'decision': {'type': 'choice', 'instructions': case['question'], 'criteria': dict(options)}}}


def followups():
    """Adaptively designed after the first screen; not a held-out benchmark."""
    import copy
    rows = []
    for original in cases():
        if original['family'] == 'evidence':
            row = copy.deepcopy(original)
            row['family'] = 'evidence_explicit'
            row['id'] = row['id'].replace('evidence', row['family'])
            if original['id'] == 'evidence-00':
                row['state'] += ' The claim is: these results establish agreement with the original on all required fixtures.'
            if original['id'] == 'evidence-01':
                row['state'] += ' The claim is: every compared field agreed on these 18 fixtures.'
            row['question'] = 'Does the supplied evidence establish, refute, or leave unresolved the explicit claim? Assess only its stated scope.'
            rows.append(row)
        if original['family'] == 'experiment':
            for variant in ['clean', 'prose', 'computed']:
                row = copy.deepcopy(original)
                row['family'] = f'experiment_{variant}'
                row['id'] = original['id'].replace('experiment', row['family'])
                row['state'].pop('target_output_for_H1')
                row['question'] = 'Which experiment makes the two hypotheses predict different outputs? Inputs and outputs are Boolean, 0=false and 1=true.'
                if variant == 'prose':
                    first = row['state']['rules'][0].split()[-1]
                    second = row['state']['rules'][1].split()[-1]
                    row['state'] = f'Hypothesis H1 predicts success whenever {first} is present, regardless of {second}. Hypothesis H2 predicts success only when both {first} and {second} are present.'
                    for key, description in row['options'].items():
                        row['options'][key] = description.replace('=0', ' is absent').replace('=1', ' is present')
                if variant == 'computed':
                    predictions = {}
                    for key, description in row['options'].items():
                        a, b = [int(part.split('=')[1]) for part in description.split(', ')]
                        predictions[key] = dict(H1=a, H2=a & b)
                    row['state']['computed_predictions'] = predictions
                rows.append(row)
        if original['family'] == 'arithmetic':
            row = copy.deepcopy(original)
            row['family'] = 'arithmetic_computed'
            row['id'] = original['id'].replace('arithmetic', row['family'])
            product = row['state']['milliseconds'] * 3
            row['state']['computed_division'] = {'integer_quotient': product // 200, 'remainder': product % 200, 'divisor': 200}
            rows.append(row)
    # These have varied answers; the initial hypothesis template always used AB.
    rng = random.Random(572)
    for i in range(32):
        truth = rng.randrange(4)
        inputs = rng.sample([(0, 0), (0, 1), (1, 0), (1, 1)], rng.randrange(1, 5))
        functions = [lambda a, b: a, lambda a, b: b, lambda a, b: a & b, lambda a, b: a | b]
        observed = [(a, b, functions[truth](a, b)) for a, b in inputs]
        survivors = ''.join('ABCD'[k] for k, fn in enumerate(functions) if all(fn(a, b) == y for a, b, y in observed))
        all_subsets = [''.join('ABCD'[k] for k in range(4) if mask & (1 << k)) for mask in range(1, 16)]
        options = rng.sample([s for s in all_subsets if s != survivors], 3) + [survivors]
        rng.shuffle(options)
        rows.append(dict(id=f'hypotheses_varied-{i:02}', family='hypotheses_varied',
            state={'rules': dict(A='output = a', B='output = b', C='output = a AND b', D='output = a OR b'),
                   'observations': [dict(a=a, b=b, output=y) for a, b, y in observed]},
            question='Which set contains exactly all rules consistent with every observation? AND and OR are Boolean operations.',
            options={s: 'Exactly rules ' + ', '.join(s) for s in options}, expected=survivors))
    return rows


def evaluate(case, key, permutation):
    payload = request_for(case, permutation)
    request = urllib.request.Request('https://api.typesafe.ai/v1/systemone',
        data=json.dumps(payload).encode(), headers={'Authorization': 'Bearer ' + key,
                                                   'Content-Type': 'application/json'})
    start = time.monotonic()
    try:
        with urllib.request.urlopen(request, timeout=30) as response:
            result = json.load(response)
    except urllib.error.HTTPError as error:
        # Do not serialize the exception/request object or any headers.
        return dict(id=case['id'], family=case['family'], error=f'HTTP {error.code}', request=payload)
    except (urllib.error.URLError, TimeoutError):
        return dict(id=case['id'], family=case['family'], error='transport failure', request=payload)
    answer = result['answers']['decision']
    if answer['choice'] not in case['options'] or set(answer['probabilities']) != set(case['options']):
        raise ValueError('Response options do not match request')
    return dict(id=case['id'], family=case['family'], permutation=permutation,
                expected=case['expected'], correct=answer['choice'] == case['expected'],
                elapsed=time.monotonic() - start, request=payload, response=result)


def retrieval_cases():
    """Authored lab-document queries; fixed candidate excerpts, including no-match."""
    root = Path(__file__).resolve().parents[2]
    catalog = [
        ('2026-09-09-soak-activity.md', ['Everything is frozen except bookkeeping, but the activity check is green. Where did we investigate that?', 'Can a changing random seed trick our measure of how much a match is doing?']),
        ('2026-09-09-soak-rng-digest.md', ['Two runs consume randomness differently but the determinism check cannot see it. What did we fix?', 'Which report added the missing random-number state to replay equality?']),
        ('2026-09-09-fixture-request-audit.md', ['A green test suite may never have opened the data it was supposed to check. Where is the relevant investigation?', 'Which report explains why counting absent file lookups is not the same as counting skipped tests?']),
        ('2026-09-09-trace-frame-sequence.md', ['A binary capture repeats a simulation step and the reader accepts it. What investigation applies?', 'Where do we explain that internally consecutive records can still be missing their ending?']),
        ('2026-09-09-path-counterexample-reduction.md', ['I want a tiny reproducer from a failing generated input while preserving the original kind of failure.', 'Where did we show that a combined flag catches an equality-versus-membership bug?']),
        ('2026-09-10-single-reseat-interventions.md', ['An intervention changes animal orientation for hundreds of steps while our headline remains identical. Where is that evidence?', 'Which report suppressed just one correction at a time and checked the resulting long-lived differences?']),
        ('2026-09-10-checkpoint-export.md', ['I need several nearby diagnostic views without paying the entire replay prefix for each.', 'Which report verifies that restored viewer windows keep their global record indices?']),
        ('2026-09-09-fixture-list-memory.md', ['A test is holding gigabytes merely to compare a list of filenames. Where did we investigate it?', 'Where did an explicit install reveal that earlier fast gates had skipped the important test bodies?']),
    ]
    documents = {}
    rows = []
    for i, (name, queries) in enumerate(catalog):
        path = root / 'docs' / 'lab' / name
        documents[f'doc{i}'] = {'file': name, 'excerpt': path.read_text()[:2600]}
        for j, query in enumerate(queries):
            rows.append(dict(id=f'retrieval-{i:02}-{j}', family='retrieval', question=query, expected=f'doc{i}'))
    for i, query in enumerate(['Where is the measured comparison of Jev versus Kev on game-playing strength?', 'Which report establishes the renderer runs at 144 frames per second on Apple Silicon?']):
        rows.append(dict(id=f'retrieval-none-{i}', family='retrieval', question=query, expected='none'))
    for row in rows:
        row['state'] = {'documents': documents, 'query': row['question']}
        row['question'] = 'Select the single document whose excerpt best answers the query. Select none if no excerpt addresses it. Judge semantic relevance, not just shared words.'
        row['options'] = {key: value['file'] for key, value in documents.items()} | {'none': 'No provided excerpt addresses the query'}
    return rows


def summarize(rows):
    result = {}
    for family in sorted({r['family'] for r in rows}):
        group = [r for r in rows if r['family'] == family and 'error' not in r]
        result[family] = {'answered': len(group), 'correct': sum(r['correct'] for r in group),
            'errors': sum('error' in r for r in rows if r['family'] == family),
            'wrong': [{'id': r['id'], 'expected': r['expected'],
                       'answer': r['response']['answers']['decision']} for r in group if not r['correct']]}
    good = [r for r in rows if 'error' not in r]
    if good:
        result['overall'] = dict(requests=len(rows), median_seconds=statistics.median(r['elapsed'] for r in good),
            input_tokens=sum(r['response']['usage']['input_tokens'] for r in good),
            estimated_usd=sum(r['response']['usage']['input_tokens'] for r in good) * 0.042 / 1_000_000)
    return result


def batched_retrieval(selected, key, permutation):
    state = {'documents': selected[0]['state']['documents']}
    questions = {}
    for case in selected:
        question = request_for(case, permutation)['questions']['decision']
        question['instructions'] += '\nQuery: ' + case['state']['query']
        questions[case['id']] = question
    payload = dict(model='jev-1.13.0', state=state, questions=questions)
    request = urllib.request.Request('https://api.typesafe.ai/v1/systemone',
        data=json.dumps(payload).encode(), headers={'Authorization': 'Bearer ' + key,
                                                   'Content-Type': 'application/json'})
    start = time.monotonic()
    with urllib.request.urlopen(request, timeout=30) as response:
        result = json.load(response)
    elapsed = time.monotonic() - start
    if set(result['answers']) != set(questions):
        raise ValueError('Batch answer IDs differ from request')
    correct = sum(result['answers'][c['id']]['choice'] == c['expected'] for c in selected)
    return dict(request=payload, response=result, expected={c['id']: c['expected'] for c in selected},
                elapsed=elapsed, correct=correct, questions=len(selected))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--permutation', type=int, default=0)
    parser.add_argument('--family', action='append')
    parser.add_argument('--dry-run', action='store_true')
    parser.add_argument('--followup', action='store_true')
    parser.add_argument('--retrieval', action='store_true')
    parser.add_argument('--batch', action='store_true', help='Pack retrieval questions over shared documents')
    parser.add_argument('--workers', type=int, default=4, choices=range(1, 5))
    args = parser.parse_args()
    if args.followup and args.retrieval:
        parser.error('Choose one suite')
    if args.batch and not args.retrieval:
        parser.error('--batch requires --retrieval')
    suite = retrieval_cases() if args.retrieval else followups() if args.followup else cases()
    selected = [c for c in suite if not args.family or c['family'] in args.family]
    if not selected:
        parser.error('No matching cases')
    # Reserve the output before retrieving credentials or making billable calls.
    with args.output.open('x') as output:
        if args.dry_run:
            json.dump(selected, output, indent=2)
            print(f'{len(selected)} authored cases; no API calls')
            return
        key = os.environ.get('TYPESAFE_API_KEY')
        if not key:
            secret = subprocess.run(['passage', 'show', 'tokens/typesafe/personal/key-for-astra'],
                                    capture_output=True, text=True, timeout=20)
            if secret.returncode or not secret.stdout.strip():
                raise SystemExit('Credential retrieval failed; details suppressed')
            key = secret.stdout.strip().splitlines()[0]
        start = time.monotonic()
        if args.batch:
            result = batched_retrieval(selected, key, args.permutation)
            json.dump(result, output, indent=2)
            print(json.dumps({k: v for k, v in result.items() if k not in ('request', 'response', 'expected')} | {'usage': result['response']['usage']}))
            return
        with ThreadPoolExecutor(max_workers=args.workers) as pool:
            rows = list(pool.map(lambda c: evaluate(c, key, args.permutation), selected))
        summary = summarize(rows)
        summary['wall_seconds'] = time.monotonic() - start
        json.dump(dict(rows=rows, summary=summary), output, indent=2)
        print(json.dumps(summary, indent=2))


if __name__ == '__main__':
    main()
