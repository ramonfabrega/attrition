"""Offline LeaderDataEncrypted bridge; preserves the full logger denominator.

The five masks are getter transformations, not inferred from equal values.
Pointer reachability and agreement do not establish allocation liveness or
coherence of unanchored referents across the snapshot copy.
"""
import argparse,collections,hashlib,json,re
from pathlib import Path
from typed_fields import Decoder
from typed_state import Payload,Types,require
from typed_state_pilots import select

# Owned getter readings: resources_get@0047da20, income_get@0046ee60,
# resource_cap_get@0046ee80, bucket_get@0046f200, support_get@0047da00. Only formulas, never exported bodies, live here.
MASKS={'resources':0x872,'income':0x90236,'resource_cap':0x1281,'bucket':0x8221,'support':0x26076}


def array_rows(rows,field):
    found={}
    for row in rows:
        path=re.sub(r'::<base:[0-9a-f]+>','',row['path'])
        m=re.fullmatch(r'encrypted\.'+re.escape(field)+r'\[([0-9]+)\]',path)
        if not m:continue
        i=int(m[1]);require(i not in found,'duplicate array index')
        require(row['status']=='value','unreadable array element')
        require(row['type_index'] in (0x12,0x22,0x74,0x75),'array element is not a 32-bit integer')
        found[i]=row
    require(found and sorted(found)==list(range(len(found))),'missing or noncontiguous array extent')
    return [found[i] for i in range(len(found))]


def apply(report,state,decode):
    """Mutate only explicit LEADERDATA array occurrences; report every refusal."""
    require(report['snapshot_sha256']==state['snapshot']['sha256'],'snapshot identity differs')
    blocks=collections.defaultdict(list)
    for row in report['rows']:
        if row['logger_path']=='GAME/FRAME/LEADERDATA':blocks[row['owner_block_line']].append(row)
    identities={};errors=[];before=report['observable_occurrences']
    for block,rows in blocks.items():
        who=[r['value'] for r in rows if r['key']=='who']
        try:
            require(len(who)==1,'ambiguous leader identity')
            identities[block]=int(who[0])
        except ValueError as e:errors.append(dict(block=block,reason=str(e)))
    counts=collections.Counter(identities.values())
    for block,rows in blocks.items():
        targets=[r for r in rows if r['key'] in MASKS]
        try:
            require(block in identities,'missing leader identity')
            who=identities[block];require(counts[who]==1,'duplicate leader identity')
            p=select(state['roots']['leaders']['rows'],f'leaders.list[{who}].data_encrypted')
            require(p['status']=='pointer' and p['value']!=0,'missing encrypted pointer')
            decoded=decode(p)
        except (ValueError,KeyError) as e:
            for row in targets:row['status']='leader_bridge_unresolved'
            errors.append(dict(block=block,reason=str(e)));continue
        for field,mask in MASKS.items():
            printed=[r for r in targets if r['key']==field]
            try:
                actual=array_rows(decoded,field)
                require(len(printed)==len(actual),'logger/storage array cardinality differs')
                require(all(r['status']=='unmapped' for r in printed),'array occurrence already classified')
                values=[int(r['value']) for r in printed]
            except ValueError as e:
                for row in printed:row['status']='leader_bridge_unresolved'
                errors.append(dict(block=block,who=who,field=field,reason=str(e)));continue
            for index,(row,c,logged) in enumerate(zip(printed,actual,values)):
                transformed=(c['value']&0xffffffff)^mask
                if transformed>=2**31:transformed-=2**32
                row.update(status='logger_transform_match' if transformed==logged else 'integer_mismatch',
                           leader=who,array_index=index,pdb_path=c['path'],address=c['address'],
                           decoded=c['value'],printed=logged,transformed=transformed,
                           logger_transform=f'LeaderDataEncrypted {field} XOR {mask:#x}, signed32')
    if not blocks:errors.append(dict(reason='no LEADERDATA records'))
    status=dict(collections.Counter(r['status'] for r in report['rows']))
    require(sum(status.values())==before,'bridge lost an occurrence')
    report.update(status_counts=status,matched_occurrences=status.get('exact_integer_match',0)+status.get('logger_transform_match',0),
                  leader_bridge=dict(blocks=len(blocks),errors=errors,referents_anchored=False),logger_parity_established=False)
    report['comparison_scope']+='; explicit LeaderDataEncrypted resources/income/resource_cap/bucket/support ordered arrays'
    return report


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    for name in ('state','comparison','types','payload'):p.add_argument(name,type=Path)
    a=p.parse_args();state=json.loads(a.state.read_text())
    require(hashlib.sha256(a.types.read_bytes()).hexdigest()==state['types_sha256'],'type export differs')
    with a.payload.open('rb') as f:require(hashlib.file_digest(f,'sha256').hexdigest()==state['snapshot']['sha256'],'payload differs')
    types=Types(json.loads(a.types.read_text()));memory=Payload(a.payload,state['snapshot'])
    def decode(pointer):
        d=Decoder(types,memory);d.decode(pointer['pointee'],pointer['value'],'encrypted');return d.rows
    try:print(json.dumps(apply(json.loads(a.comparison.read_text()),state,decode),indent=2))
    finally:memory.close()
