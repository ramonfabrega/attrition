"""Account for printed occurrences and distinct compared storage references.

Repeated logger output increases occurrence coverage, never distinct storage
coverage. Unbridged decoded fields are not automatically absent from the logger.
"""
import argparse,collections,json
from pathlib import Path
from typed_state import require

MATCHES={'exact_integer_match','logger_transform_match'}


def summarize(state,report):
    require(state['snapshot']['sha256']==report['snapshot_sha256'],'snapshot identity differs')
    statuses=collections.Counter();families=collections.defaultdict(collections.Counter);references=collections.defaultdict(list)
    for row in report['rows']:
        status=row['status'];statuses[status]+=1;families[row['logger_path']][status]+=1
        if status in MATCHES or status=='integer_mismatch':
            require(type(row.get('address')) is int and isinstance(row.get('pdb_path'),str),'comparison lacks storage identity')
            references[(row['address'],row['pdb_path'])].append(status)
    require(sum(statuses.values())==report['observable_occurrences'],'occurrence denominator differs')
    require(dict(statuses)==report['status_counts'],'reported status counts differ')
    matched=sum(statuses[s] for s in MATCHES)
    require(matched==report['matched_occurrences'],'reported match count differs')
    distinct_agree=sum(all(s in MATCHES for s in seen) for seen in references.values())
    distinct_disagree=sum('integer_mismatch' in seen for seen in references.values())
    roots={name:dict(collections.Counter(r['status'] for r in root['rows'])) for name,root in state.get('roots',{}).items()}
    units=collections.Counter()
    for unit in state.get('unit_registry',{}).get('units',[]):
        if unit.get('logger_active_flag') is True and unit.get('identity')=='agrees':units.update(r['status'] for r in unit['rows'])
    return dict(schema='typed-state-coverage-v1',snapshot_sha256=state['snapshot']['sha256'],
                observable_occurrences=report['observable_occurrences'],matched_occurrences=matched,status_counts=dict(statuses),
                compared_storage_references=len(references),distinct_agreeing_storage_references=distinct_agree,
                distinct_mismatching_storage_references=distinct_disagree,
                repeated_compared_occurrences=sum(len(v)-1 for v in references.values()),
                families=[dict(path=path,occurrences=sum(counts.values()),matched=sum(counts[s] for s in MATCHES),statuses=dict(counts))
                          for path,counts in sorted(families.items(),key=lambda x:-sum(x[1].values()))],
                decoded_root_row_statuses=roots,active_identity_checked_unit_row_statuses=dict(units),
                exact_height_words=state.get('heights',{}).get('count'),
                live_allocation_coverage=None,whole_frame_parity=False,
                limitations=['Storage references are (address, PDB path), not byte ranges or live allocations.',
                             'Decoded root rows include inactive leaders; unit rows are filtered by active flag and registry identity.',
                             'Unbridged fields may already be printed under aliases; no absent-from-logger claim follows.',
                             'Exact height words are available, not counted as logger matches by this report.'])


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('state',type=Path);p.add_argument('comparison',type=Path);a=p.parse_args()
    print(json.dumps(summarize(json.loads(a.state.read_text()),json.loads(a.comparison.read_text())),indent=2))
