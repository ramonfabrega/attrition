"""Explicit LeaderData logger projections over retained, PDB-decoded roots.

The resource loop prints seven direct arrays, doubles gather_slots_high, and
prints bonus_cap's final element under a distinct key. A diplomacy loop repeats
one scalar. These are logger contracts, not same-name equality heuristics.
"""
import argparse,collections,json,re
from pathlib import Path
from typed_state import require
from typed_state_pilots import select

RESOURCE_ARRAYS=('econ','escrow','escrow_rate','tributes','base_rate',
                 'gather_slots','filled_gather_slots')
RESOURCE_KEYS={f+'[scan]' for f in RESOURCE_ARRAYS}|{
    'gather_slots_high[scan]','bonus_cap[scan]','bonus_cap[NUM_COMMON]'}


def array(rows,prefix,field):
    found={}
    for row in rows:
        path=re.sub(r'::<base:[0-9a-f]+>','',row['path'])
        m=re.fullmatch(re.escape(prefix+'.'+field)+r'\[([0-9]+)\]',path)
        if not m:continue
        i=int(m[1]);require(i not in found,'ambiguous array storage')
        require(row['status']=='value' and type(row.get('value')) is int,'unreadable integer array')
        found[i]=row
    require(found and sorted(found)==list(range(len(found))),'missing or noncontiguous array')
    return [found[i] for i in range(len(found))]


def resource_plan(rows,prefix):
    arrays={name:array(rows,prefix,name) for name in (*RESOURCE_ARRAYS,'gather_slots_high','bonus_cap')}
    n=len(arrays['econ'])
    require(all(len(arrays[name])==n for name in (*RESOURCE_ARRAYS,'gather_slots_high')),'resource extent differs')
    require(len(arrays['bonus_cap'])==n+1,'cap extent differs')
    plan={name+'[scan]':[(i,r) for i,r in enumerate(arrays[name])] for name in RESOURCE_ARRAYS}
    plan['gather_slots_high[scan]']=[(i,r) for i,r in enumerate(arrays['gather_slots_high']) for _ in range(2)]
    plan['bonus_cap[scan]']=list(enumerate(arrays['bonus_cap'][:-1]))
    plan['bonus_cap[NUM_COMMON]']=[(n,arrays['bonus_cap'][-1])]
    return plan


def diplomacy_plan(rows,prefix):
    # The logger iterates the diplomacy slots, printing the same scalar each time.
    n=len(array(rows,prefix,'chat_status'))
    scalar=select(rows,prefix+'.got_diplo_message')
    require(scalar['status']=='value' and type(scalar.get('value')) is int,'unreadable diplomacy scalar')
    return {'got_diplo_message':[(None,scalar)]*n}


def apply(report,state):
    require(report['snapshot_sha256']==state['snapshot']['sha256'],'snapshot identity differs')
    blocks=collections.defaultdict(list);identities={};errors=[]
    for row in report['rows']:
        if row['logger_path']=='GAME/FRAME/LEADERDATA':blocks[row['owner_block_line']].append(row)
    for block,rows in blocks.items():
        who=[r['value'] for r in rows if r['key']=='who']
        try:
            require(len(who)==1,'ambiguous leader identity');identities[block]=int(who[0])
        except ValueError as e:errors.append(dict(block=block,reason=str(e)))
    counts=collections.Counter(identities.values())
    for block,rows in blocks.items():
        for family,keys,build in (('resources',RESOURCE_KEYS,resource_plan),
                                  ('diplomacy',{'got_diplo_message'},diplomacy_plan)):
            targets=[r for r in rows if r['key'] in keys]
            require(all(r['status']=='unmapped' for r in targets),'projection would overwrite classified occurrence')
            try:
                require(block in identities,'missing leader identity')
                who=identities[block];require(counts[who]==1,'duplicate leader identity')
                plan=build(state['roots']['leaders']['rows'],f'leaders.list[{who}]')
                paired=[]
                for key,storage in plan.items():
                    printed=[r for r in targets if r['key']==key]
                    require(len(printed)==len(storage),'logger projection cardinality differs: '+key)
                    for occurrence,(row,(index,c)) in enumerate(zip(printed,storage)):
                        paired.append((row,index,c,int(row['value']),occurrence))
            except (ValueError,KeyError) as e:
                for row in targets:row['status']='leader_projection_unresolved'
                errors.append(dict(block=block,family=family,reason=str(e)));continue
            for row,index,c,logged,occurrence in paired:
                row.update(status='exact_integer_match' if c['value']==logged else 'integer_mismatch',
                           leader=who,array_index=index,projection_occurrence=occurrence,
                           pdb_path=c['path'],address=c['address'],decoded=c['value'],printed=logged,
                           logger_projection=family+' loop; LeaderData::log_data@006e5110')
    if not blocks:errors.append(dict(reason='no LEADERDATA records'))
    status=dict(collections.Counter(r['status'] for r in report['rows']))
    require(sum(status.values())==report['observable_occurrences'],'projection lost an occurrence')
    report.update(status_counts=status,matched_occurrences=status.get('exact_integer_match',0)+status.get('logger_transform_match',0),
                  leader_projection=dict(blocks=len(blocks),errors=errors),logger_parity_established=False)
    report['comparison_scope']+='; explicit leader resource and repeated diplomacy-scalar projections'
    return report


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('state',type=Path);p.add_argument('comparison',type=Path);a=p.parse_args()
    print(json.dumps(apply(json.loads(a.comparison.read_text()),json.loads(a.state.read_text())),indent=2))
