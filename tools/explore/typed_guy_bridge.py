"""Follow typed unit figure arrays and compare each observed GUY record.

Identity is (who, o, guy_num), never pointer-array position. No claim is made
about unlogged figures, all-thread coherence, or allocator liveness.
"""
import argparse,collections,hashlib,json,re,struct
from pathlib import Path
from typed_fields import Decoder
from typed_state import Payload,Types,require
from typed_state_pilots import select,array_extent

SCOPES={'GAME/FRAME/GUY','GAME/FRAME/UNITDATA/GUY','GAME/FRAME/ANIMALDATA/UNITDATA/GUY'}
BITS={'turret_inc','bank','last_bank','pitch','last_pitch'}


def decode_figures(state,types,memory):
    figures={};errors=[];slots=0
    for unit in state['unit_registry']['units']:
        if unit['logger_active_flag'] is not True or unit['identity']!='agrees':continue
        owner=(unit['registry_owner'],unit['registry_slot'])
        try:
            n,p=array_extent(unit['rows'],'unit.guys',1000);slots+=n
            require(slots<=10000,'figure slot budget exceeded')
            element=types.record(types.resolve(p['pointee']))
            require(element['Kind']=='LF_POINTER' and Decoder(types,memory).size(p['pointee'])==4,'figure array is not pointer32')
            for slot in range(n):
                address=struct.unpack('<I',memory.read(p['value']+4*slot,4))[0]
                require(address!=0,'null figure pointer')
                d=Decoder(types,memory);d.decode(element['Pointer']['ReferentType'],address,'guy')
                identity_rows=[select(d.rows,'guy.'+k) for k in ('who','o','guy_num')]
                require(all(r['status']=='value' and type(r.get('value')) is int for r in identity_rows),'unreadable figure identity')
                identity=tuple(r['value'] for r in identity_rows)
                require(identity[:2]==owner,'figure identity differs from owning unit')
                require(identity not in figures,'duplicate figure identity')
                figures[identity]=dict(address=address,slot=slot,rows=d.rows)
        except ValueError as e:errors.append(dict(owner=owner,reason=str(e)))
    # Never keep a possibly ambiguous partial identity map as a valid bridge.
    require(not errors,'figure traversal refused: '+json.dumps(errors))
    return figures


def field_index(rows):
    fields=collections.defaultdict(list)
    for row in rows:
        path=re.sub(r'::<base:[0-9a-f]+>','',row['path'])
        if path.endswith('.value'):path=path[:-6]
        if re.fullmatch(r'guy\.[A-Za-z_][A-Za-z_0-9]*(?:\[[0-9]+\])?',path):fields[path[4:]].append(row)
    return fields


def apply(report,state,figures):
    require(report['snapshot_sha256']==state['snapshot']['sha256'],'snapshot identity differs')
    blocks=collections.defaultdict(list);identities={};errors=[];linked=set();complete=0
    for row in report['rows']:
        if row['logger_path'] in SCOPES:blocks[row['owner_block_line']].append(row)
    for block,rows in blocks.items():
        try:
            identity=[]
            for key in ('who','o','guy_num'):
                values=[r['value'] for r in rows if r['key']==key]
                require(len(values)==1,'ambiguous printed figure identity');identity.append(int(values[0]))
            identities[block]=tuple(identity)
        except ValueError as e:errors.append(dict(block=block,reason=str(e)))
    indexes={identity:field_index(figure['rows']) for identity,figure in figures.items()}
    for block,rows in blocks.items():
        if block not in identities:continue
        identity=identities[block]
        if identity not in figures:errors.append(dict(block=block,identity=identity,reason='no typed figure identity'));continue
        linked.add(identity);fields=indexes[identity]
        normalized=[]
        for row in rows:
            name=row['key'];value=row['value'];bits=False
            if name=='*((dword*)':
                match=re.fullmatch(r'&([A-Za-z_][A-Za-z_0-9]*)\) ([0-9]+)',value)
                if not match or match[1] not in BITS:
                    normalized.append((row,None,None,False));continue
                name,value,bits=match[1],match[2],True
            elif name in ('(int)off_x','(int)off_y','(int)variation'):name=name[5:]
            normalized.append((row,name,value,bits))
        counts=collections.Counter(name for _,name,_,_ in normalized if name is not None)
        for row,name,value,bits in normalized:
            if name is None:continue
            candidates=fields.get(name,[])
            if not candidates:continue
            require(row['status']=='unmapped','figure would overwrite classified occurrence')
            try:
                require(counts[name]==1 and len(candidates)==1,'ambiguous figure field')
                c=candidates[0];printed=int(value)
                if bits:
                    require(c['status']=='float_bits' and c['width']==32 and len(c['bits'])==8,'not an exact float32 word')
                    actual=int.from_bytes(bytes.fromhex(c['bits']),'little')
                else:
                    require(c['status']=='value' and type(c.get('value')) is int,'unsupported figure storage')
                    actual=c['value']
                status=('logger_transform_match' if bits else 'exact_integer_match') if actual==printed else 'integer_mismatch'
                row.update(status=status,figure_identity=identity,pdb_path=c['path'],address=c['address'],
                           decoded=c.get('value',c.get('bits')),printed=printed,transformed=actual,
                           logger_transform='float32 bits as unsigned DWORD' if bits else None)
            except ValueError as e:
                row['status']='guy_bridge_unresolved';errors.append(dict(block=block,key=row['key'],reason=str(e)))
        if all(r['status'] in ('exact_integer_match','logger_transform_match') for r in rows):complete+=1
    if not blocks:errors.append(dict(reason='no supported GUY records'))
    status=dict(collections.Counter(r['status'] for r in report['rows']))
    require(sum(status.values())==report['observable_occurrences'],'figure comparison lost an occurrence')
    report.update(status_counts=status,matched_occurrences=status.get('exact_integer_match',0)+status.get('logger_transform_match',0),
                  guy_bridge=dict(decoded_figures=len(figures),logged_blocks=len(blocks),complete_observed_records=complete,
                                  distinct_linked_identities=len(linked),unlogged_identities=sorted(set(figures)-linked),
                                  errors=errors,referents_anchored=False),logger_parity_established=False)
    report['comparison_scope']+='; identity-linked GUY integers and raw float32 words'
    return report


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    for name in ('state','comparison','types','payload'):p.add_argument(name,type=Path)
    a=p.parse_args();state=json.loads(a.state.read_text())
    require(hashlib.sha256(a.types.read_bytes()).hexdigest()==state['types_sha256'],'type export differs')
    with a.payload.open('rb') as f:require(hashlib.file_digest(f,'sha256').hexdigest()==state['snapshot']['sha256'],'payload differs')
    types=Types(json.loads(a.types.read_text()));memory=Payload(a.payload,state['snapshot'])
    try:print(json.dumps(apply(json.loads(a.comparison.read_text()),state,decode_figures(state,types,memory)),indent=2))
    finally:memory.close()
