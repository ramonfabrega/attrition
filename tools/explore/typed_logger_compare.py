"""First scalar bridge: account for all lines, compare scoped UnitData integers.

This deliberately reports partial coverage; it is not an oracle replacement.
The logger-family bridge is explicit; candidate storage fields come from PDB.
"""
import argparse,collections,hashlib,json,re
from pathlib import Path
from typed_state import Types,require
from typed_logger_inventory import extract,inventory

FAMILIES={'SUBOBJECT':'SubObjectData','OBJECT':'ObjectData','UNITDATA':'UnitData'}


def scalar_index(unit,types):
    result=collections.defaultdict(list)
    for row in unit['rows']:
        path=row['path'];plain=re.sub(r'::<base:[0-9a-f]+>','',path)
        # Fixed-coordinate wrappers expose their stored integer as .value.
        wrapper=plain.endswith('.value')
        if wrapper:plain=plain[:-6]
        if not re.fullmatch(r'unit\.[A-Za-z_][A-Za-z_0-9]*',plain):continue
        bases=re.findall(r'::<base:([0-9a-f]+)>',path)
        if not bases:continue
        record=types.record(types.resolve(int(bases[-1],16)))
        owner=record.get('Class',{}).get('Name')
        result[(owner,plain[5:])].append(dict(**row,storage_wrapper=wrapper))
    return result


def compare(lines,frame,state,types):
    require(state['snapshot']['frame']==frame,'snapshot/logger frame differs')
    report=inventory(lines,frame);by_line={r['line']:r for r in report['rows']}
    active={};indexes={}
    for u in state['unit_registry']['units']:
        if u['logger_active_flag'] is not True or u['identity']!='agrees':continue
        key=(u['registry_owner'],u['registry_slot']);require(key not in active,'duplicate active registry identity')
        active[key]=u;indexes[key]=scalar_index(u,types)
    blocks=[];current=None
    for number,line in lines:
        text=line.strip();depth=len(line)-len(line.lstrip(' '))
        if current is not None and (depth<current['depth'] or depth==current['depth'] and text.startswith('BEGIN ')):
            blocks.append(current);current=None
        if text=='BEGIN UNITDATA':
            require(current is None,'nested UnitData block')
            current=dict(depth=depth,line=number,rows=[])
        if current is not None:current['rows'].append((number,line))
    if current is not None:blocks.append(current)
    require(blocks,'no UnitData records; cannot claim agreement')
    linked=[];unlinked=[]
    for block in blocks:
        stack=[];fields=[];identity=collections.defaultdict(list)
        for number,line in block['rows']:
            text=line.strip();depth=len(line)-len(line.lstrip(' '))
            while stack and stack[-1][0]>=depth:stack.pop()
            if text.startswith('BEGIN '):stack.append((depth,text[6:]));continue
            if number not in by_line or not stack:continue
            r=by_line[number];family=stack[-1][1]
            scopes=[x[1] for x in stack]
            if scopes in (['UNITDATA'],['UNITDATA','OBJECT'],['UNITDATA','OBJECT','SUBOBJECT']):
                fields.append((r,family))
            if family=='SUBOBJECT' and r['key'] in ('who','o'):identity[r['key']].append(r['value'])
        if len(identity['who'])!=1 or len(identity['o'])!=1:
            unlinked.append(dict(line=block['line'],reason='ambiguous identity'));continue
        try:key=(int(identity['who'][0]),int(identity['o'][0]))
        except ValueError:
            unlinked.append(dict(line=block['line'],reason='noninteger identity'));continue
        if key not in active:
            unlinked.append(dict(line=block['line'],identity=key,reason='no active typed registry match'));continue
        linked.append(key)
        for row,family in fields:
            owner=FAMILIES.get(family)
            if owner is None:continue
            candidates=indexes[key].get((owner,row['key']),[])
            if not candidates:continue
            if len(candidates)!=1:row['status']='ambiguous_storage_match';continue
            candidate=candidates[0]
            if candidate['status'] not in ('value','bitfield'):row['status']='unsupported_storage';continue
            value=row['value'].strip()
            if not re.fullmatch(r'-?(?:[0-9]+|0x[0-9a-fA-F]+)',value):row['status']='unsupported_printed_value';continue
            logged=int(value,16 if '0x' in value else 10);actual=candidate['value']
            transformed=actual;transform=None
            if owner=='SubObjectData' and row['key'] in ('x_internal','y_internal','z_internal') and candidate['storage_wrapper']:
                transformed=(actual&0xffffffff)^0x63637
                if transformed>=2**31:transformed-=2**32
                transform='Coord XOR 0x63637; documented in FORMATS'
            status=('logger_transform_match' if transform else 'exact_integer_match') if logged==transformed else 'integer_mismatch'
            row.update(status=status,logger_transform=transform,transformed=transformed,
                       identity=key,pdb_path=candidate['path'],address=candidate['address'],decoded=actual,printed=logged,
                       storage_wrapper=candidate['storage_wrapper'])
    # WORLD's direct scalar fields are a second explicit record-family bridge.
    world=collections.defaultdict(list)
    for candidate in state.get('roots',{}).get('world',{}).get('rows',[]):
        path=re.sub(r'::<base:[0-9a-f]+>','',candidate['path'])
        if re.fullmatch(r'world\.[A-Za-z_][A-Za-z_0-9]*',path):world[path[6:]].append(candidate)
    world_counts=collections.Counter((r['owner_block_line'],r['key']) for r in report['rows'] if r['logger_path']=='GAME/FRAME/WORLD')
    for row in report['rows']:
        if row['logger_path']!='GAME/FRAME/WORLD' or row['status']!='unmapped':continue
        candidates=world.get(row['key'],[])
        if len(candidates)!=1 or candidates[0]['status']!='value':continue
        if world_counts[(row['owner_block_line'],row['key'])]!=1:
            row['status']='ambiguous_logger_ownership';continue
        value=row['value'].strip()
        if not re.fullmatch(r'-?(?:[0-9]+|0x[0-9a-fA-F]+)',value):continue
        c=candidates[0];logged=int(value,16 if '0x' in value else 10)
        row.update(status='exact_integer_match' if c['value']==logged else 'integer_mismatch',
                   pdb_path=c['path'],address=c['address'],decoded=c['value'],printed=logged)
    counts=dict(collections.Counter(r['status'] for r in report['rows']))
    report.update(status_counts=counts,matched_occurrences=counts.get('exact_integer_match',0)+counts.get('logger_transform_match',0),
                  unitdata_records=len(blocks),linked_unitdata_records=len(linked),unlinked_unitdata_records=unlinked,
                  active_registry_identities_not_dumped=sorted(set(active)-set(linked)),
                  snapshot_sha256=state['snapshot']['sha256'],logger_return_roots_unchanged=state['snapshot']['logger_return_roots_unchanged'],
                  comparison_scope='PDB scalar storage under explicit SubObjectData/ObjectData/UnitData and WORLD logger-family bridges; all other occurrences retained',
                  logger_parity_established=False)
    require(sum(counts.values())==report['observable_occurrences'],'comparison lost a printed occurrence')
    return report

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    for name in ('gamelog','state','types'):p.add_argument(name,type=Path)
    a=p.parse_args();state=json.loads(a.state.read_text());frame=state['snapshot']['frame']
    require(hashlib.sha256(a.types.read_bytes()).hexdigest()==state['types_sha256'],'comparison type export differs')
    print(json.dumps(compare(extract(a.gamelog,frame),frame,state,Types(json.loads(a.types.read_text()))),indent=2))
