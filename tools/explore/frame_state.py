"""Decode a validated end-frame packet through PDB roots and explicit extents.

Registry occupancy and the logger's active flag are separate from allocator
liveness. Original-derived results must remain outside Git.
"""
import argparse,hashlib,json,re,struct
from pathlib import Path
from typed_state import load_bound_types,Payload,require
from typed_fields import Decoder,globals_from_dump
from typed_state_pilots import root_decode,select,array_extent
from frame_snapshot import validate


def has_base(types,dynamic,declared):
    wanted=types.resolve(declared);pending=[dynamic];seen=set()
    while pending:
        current=types.resolve(pending.pop())
        if current==wanted:return True
        if current in seen:continue
        seen.add(current);require(len(seen)<=1024,'inheritance traversal budget exceeded')
        for field in types.fields(current):
            body=next(v for k,v in field.items() if k!='Kind')
            if field['Kind']=='LF_BCLASS':pending.append(body['Type'])
            elif field['Kind'] in ('LF_VBCLASS','LF_IVBCLASS'):pending.append(body['BaseType'])
    return False


def unit_registry(types,memory,rows,leader_rows,joined):
    paths=[re.sub(r'::<base:[0-9a-f]+>','',r['path']) for r in rows]
    owners=sorted({int(m[1]) for p in paths if (m:=re.fullmatch(r'units\.lists\[([0-9]+)\]\.list',p))})
    require(owners and owners==list(range(len(owners))),'noncontiguous/missing declared unit registries')
    result=[];empty=0;slots=0;inactive=[]
    for owner in owners:
        leader=select(leader_rows,f'leaders.list[{owner}].leader_flags')
        require(leader['status']=='value','leader activity unresolved')
        if not leader['value']&1:
            inactive.append(owner);continue
        length,pointer=array_extent(rows,f'units.lists[{owner}]',32768)
        element=types.record(types.resolve(pointer['pointee']))
        require(element['Kind']=='LF_POINTER','unit registry element is not a pointer')
        require(Decoder(types,memory).size(pointer['pointee'])==4,'unit registry pointer width differs')
        target=element['Pointer']['ReferentType'];slots+=length
        require(slots<=327680,'registry slot budget exceeded')
        for index in range(length):
            address=struct.unpack('<I',memory.read(pointer['value']+4*index,4))[0]
            if not address:empty+=1;continue
            dynamic=None;complete=address;declared=target
            try:
                vptr=struct.unpack('<I',memory.read(address,4))[0]
                dynamic=joined.get(vptr)
                require(dynamic is not None,'registry vtable not joined')
                require(has_base(types,dynamic['type_index'],target),'RTTI type does not contain declared pointer type')
                complete=address-dynamic['offset'];declared=dynamic['type_index']
            except ValueError as error:
                result.append(dict(address=address,registry_owner=owner,registry_slot=index,identity='unresolved',logger_active_flag=None,reason=str(error),rows=[]));continue
            d=Decoder(types,memory);d.decode(declared,complete,'unit')
            active=None;identity='unresolved'
            try:
                who=select(d.rows,'unit.who');oid=select(d.rows,'unit.o');flags=select(d.rows,'unit.flags')
                require(who['status']==oid['status']==flags['status']=='value','unit identity unresolved')
                active=bool(flags['value']&1)
                identity='agrees' if (who['value'],oid['value'])==(owner,index) else 'differs'
            except ValueError:pass
            result.append(dict(address=address,registry_owner=owner,registry_slot=index,
                               identity=identity,dynamic_type=dynamic['name'],complete_address=complete,logger_active_flag=active,**d.report()))
    return dict(declared_registries=len(owners),inactive_leader_registries=inactive,slots=slots,null_slots=empty,
                occupied_slots=len(result),active_flag_slots=sum(u['logger_active_flag'] is True for u in result),
                unresolved_slots=sum(u['logger_active_flag'] is None for u in result),
                active_identity_mismatches=sum(u['logger_active_flag'] is True and u['identity']!='agrees' for u in result),
                live_allocation_count=None,units=result)


def experiment(install,capture,export,plan):
    pe,types,data,raw=load_bound_types(install,export/'types.json')
    require(hashlib.sha256(raw).hexdigest()==plan['image_sha256'],'plan image differs')
    require(hashlib.sha256((export/'types.json').read_bytes()).hexdigest()==plan['types_sha256'],'plan types differ')
    g=(export/'globals.txt').read_bytes()
    require(hashlib.sha256(g).hexdigest()==data['_source']['globals_sha256'],'globals differ')
    symbols=globals_from_dump(g.decode(),pe);report=validate(capture,plan);memory=Payload(capture/'frame-snapshot.bin',report)
    try:
        roots={}
        for root in plan['roots']:
            choices=[s for s in symbols if s['name']==root['name']]
            require(len(choices)==1 and choices[0]['address']==root['address'] and choices[0]['type_index']==root['type_index'],'root plan differs')
            d=root_decode(root['name'],symbols,types,memory)
            require(d.size(root['type_index'])==root['size'],'root size differs')
            roots[root['name']]=d.report()
        count,pointer=array_extent(roots['terrain']['rows'],'terrain.master_land_heights',1000000)
        require(pointer['pointee']==0x40,'height element is not float32')
        width=select(roots['world']['rows'],'world.tile_xs')['value']+1
        height=select(roots['world']['rows'],'world.tile_ys')['value']+1
        require(width>0 and height>0 and count==width*height,'height dimensions differ')
        bits=memory.read(pointer['value'],count*4)
        return dict(schema='typed-frame-state-v1',snapshot=report,types_sha256=plan['types_sha256'],roots=roots,
                    heights=dict(count=count,width=width,height=height,exact_float32_le_hex=bits.hex(),
                                 sha256=hashlib.sha256(bits).hexdigest()),
                    unit_registry=unit_registry(types,memory,roots['units']['rows'],roots['leaders']['rows'],types.join(pe.rtti())[0]),
                    logger_parity_established=False)
    finally:memory.close()

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    for n in ('install','capture','export','plan'):p.add_argument(n,type=Path)
    a=p.parse_args();print(json.dumps(experiment(a.install,a.capture,a.export,json.loads(a.plan.read_text())),indent=2))
