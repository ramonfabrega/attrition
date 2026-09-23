"""Candidate direct leader-field joins, kept separate from validated coverage.

Same-name equality is a search aid, not proof of logger ownership. This report
never changes comparison rows or promotes a candidate into parity coverage.
"""
import argparse,collections,json,re
from pathlib import Path
from typed_state import require


def survey(state,comparison):
    require(state['snapshot']['sha256']==comparison['snapshot_sha256'],'snapshot identity differs')
    blocks=collections.defaultdict(list)
    for row in comparison['rows']:
        if row['logger_path']=='GAME/FRAME/LEADERDATA':blocks[row['owner_block_line']].append(row)
    identities={};errors=[]
    for block,rows in blocks.items():
        who=[r['value'] for r in rows if r['key']=='who']
        try:
            require(len(who)==1,'ambiguous leader identity');identities[block]=int(who[0])
        except ValueError as e:errors.append(dict(block=block,reason=str(e)))
    counts=collections.Counter(identities.values());results=[]
    for block,rows in blocks.items():
        if block not in identities:continue
        who=identities[block]
        if counts[who]!=1:errors.append(dict(block=block,reason='duplicate leader identity'));continue
        fields=collections.defaultdict(list)
        for row in state['roots']['leaders']['rows']:
            path=re.sub(r'::<base:[0-9a-f]+>','',row['path'])
            m=re.fullmatch(r'leaders.list\['+str(who)+r'\]\.([A-Za-z_][A-Za-z_0-9]*)(?:\[([0-9]+)\])?',path)
            if m:fields[m[1]].append((int(m[2]) if m[2] is not None else -1,row))
        for field,storage in fields.items():
            printed=[r for r in rows if r['key'] in (field,field+'[scan]')]
            if not printed:continue
            indices=[i for i,_ in storage];status='candidate_equal';reason=None
            actual=[];values=[]
            try:
                require(len(indices)==len(set(indices)),'ambiguous storage ownership')
                require(indices==[-1] or sorted(indices)==list(range(len(indices))),'noncontiguous storage')
                require(all(r['status'] in ('value','bitfield') for _,r in storage),'unsupported storage')
                actual=[r['value'] for _,r in sorted(storage)]
                values=[int(r['value']) for r in printed]
                if len(actual)!=len(values):status='shape_difference'
                elif actual!=values:status='candidate_value_difference'
            except ValueError as e:status='unresolved';reason=str(e)
            results.append(dict(who=who,field=field,status=status,reason=reason,
                                storage_count=len(storage),printed_count=len(printed),
                                storage_paths=[r['path'] for _,r in storage],
                                logger_lines=[r['line'] for r in printed],actual=actual,printed=values))
    if not blocks:errors.append(dict(reason='no LEADERDATA records'))
    return dict(schema='leader-candidate-inventory-v1',snapshot_sha256=state['snapshot']['sha256'],
                candidate_equal_occurrences=sum(r['printed_count'] for r in results if r['status']=='candidate_equal'),
                status_counts=dict(collections.Counter(r['status'] for r in results)),rows=results,errors=errors,
                validated_coverage_added=0,logger_parity_established=False,
                limitation='Name and cardinality joins require logger ownership validation; aliases are not inferred.')


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('state',type=Path);p.add_argument('comparison',type=Path);a=p.parse_args()
    print(json.dumps(survey(json.loads(a.state.read_text()),json.loads(a.comparison.read_text())),indent=2))
