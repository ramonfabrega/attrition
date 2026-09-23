"""Compare an explicit Rust UNREAD literal with observed logger occurrences.

This is evidence on one frame, not adoption into the main parser/comparator.
Unrecognized literal syntax refuses rather than silently shrinking the pin.
"""
import argparse,collections,gzip,hashlib,json,re
from pathlib import Path
from typed_state import require
from typed_coverage_report import MATCHES


def parse_pin(source):
    match=re.search(r'const UNREAD\s*:\s*&\[\(&str,\s*&str\)\]\s*=\s*&\[([\s\S]*?)\n\];',source)
    require(match is not None,'UNREAD literal not found')
    # This pinned literal uses plain strings and line comments only. A scanner
    # distinguishes comments from quoted content; unsupported syntax stays residue.
    body=re.sub(r'"(?:\\.|[^"\\])*"|//[^\n]*',lambda m:'' if m[0].startswith('//') else m[0],match[1])
    string=r'"(?:\\.|[^"\\])*"';pattern=r'\(\s*('+string+r')\s*,\s*('+string+r')\s*,?\s*\)\s*,?'
    rows=[]
    def consume(m):
        path,keys=json.loads(m[1]),json.loads(m[2])
        rows.extend((path,key) for key in keys.split());return ''
    residue=re.sub(pattern,consume,body)
    require(not residue.strip() and rows,'unparsed or empty UNREAD literal')
    require(len(rows)==len(set(rows)),'duplicate pin key')
    return rows


def compare_pin(source,report):
    keys=parse_pin(source);observed=collections.defaultdict(collections.Counter)
    for row in report['rows']:observed[(row['logger_path'],row['key'])][row['status']]+=1
    rows=[]
    for path,key in keys:
        counts=observed[(path,key)]
        status='absent' if not counts else 'all_observed_occurrences_match' if set(counts)<=MATCHES else 'not_fully_matched'
        rows.append(dict(path=path,key=key,status=status,occurrences=dict(counts)))
    return dict(schema='typed-pin-coverage-v1',pin_source_sha256=hashlib.sha256(source.encode()).hexdigest(),
                snapshot_sha256=report['snapshot_sha256'],frame=report['frame'],pin_keys=len(keys),
                pin_paths=len({p for p,_ in keys}),counts=dict(collections.Counter(r['status'] for r in rows)),rows=rows,
                main_parser_changed=False,whole_frame_parity=False,
                limitation='All observed occurrences on this frame only; absent keys are not agreement.')


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('coverage_source',type=Path);p.add_argument('comparison',type=Path);a=p.parse_args()
    opener=gzip.open if a.comparison.suffix=='.gz' else open
    with opener(a.comparison,'rt') as f:report=json.load(f)
    print(json.dumps(compare_pin(a.coverage_source.read_text(),report),indent=2))
