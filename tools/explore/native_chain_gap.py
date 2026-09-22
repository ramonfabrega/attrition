"""Bounded native-output comparison of a frozen-world two-call model.

Validates a paired capture and both projections. Run the first call at its
witnessed limit 95, then reuse that model's heap/global state. Compare a frozen
re-entry and a re-entry with only the original caller's collide instruction.
The second native input is a comparison target, never a replacement world.
Generated reports contain capture-derived state and belong outside git.
"""
import argparse,json,struct
from pathlib import Path
from caller_collide import CallerCollide,OFFSET,RETURN
from continued_call import continue_once
from defined_bytes import observe_bytes
from memory_inventory import decode as decode_inventory
from memory_payload import validate
from native_limit_replay import experiment as native_witness
from path_intervention import execute,input_patch
from payload_replay import explore,observe_unit_path,CapturedPages
from replay_capsule import require,digest
from restore_context import decode_context
from restore_poststate import compare_boundary,require_agreement
from resume_frontier import registers
from search_census import records
from search_graph import parse,words
from second_restore_packet import validate_pair,FILES,COMMON


def unit_delta(before,after):
    require(isinstance(before,bytes) and isinstance(after,bytes) and len(before)==len(after)==344,
            'complete immutable unit records required')
    changed=[i for i,(a,b) in enumerate(zip(before,after)) if a!=b]
    require(changed and all(OFFSET<=i<OFFSET+2 for i in changed),
            'native boundary change is not confined to collide')
    return changed



def require_frozen_gap(result):
    require(result['unit_changes_outside_path_pointer']==[OFFSET],
            'frozen gap is not the observed byte')
    require(result['outer_return_equal'] and result['return_modes_equal'] is True and
            result['native_length']==result['replay_length'] and
            result['native_capacity']==result['replay_capacity'] and
            result['active_path_equal'] and result['all_path_slots_equal'],
            'frozen chain differs beyond the unit field and explicit path relocation')


def prepare(install,root):
    source=root/'map-14';first=root/'packet-0';second=root/'packet-1'
    natives=validate_pair(source,list(records(source/'rontrace.log')))
    contexts=[];reports=[]
    for index,p in enumerate((first,second)):
        stem='second-' if index else ''
        for name in FILES:
            require(digest(p/name)==digest(source/(stem+name)),'projection content differs: '+name)
        for name in COMMON:
            require(digest(p/name)==digest(source/name),'projection image binding differs: '+name)
        manifest=json.loads((p/'packet-projection.json').read_text())
        require(manifest['packet_index']==index and manifest['source_trace_sha256']==digest(source/'rontrace.log')
                and manifest['files']['rontrace.log']==digest(p/'rontrace.log'),'projection trace provenance differs')
        reports.append(validate(install,p))
        contexts.append(decode_context((p/'restore-context.bin').read_bytes(),(p/'restore-prefix.bin').read_bytes()))
    a,b=contexts
    require(a['unit']==b['unit'] and a['frame']<b['frame'],'native call identity/order differs')
    require(a['prefix']['stack'][0]==b['prefix']['stack'][0]==RETURN,'unexpected native restore caller')
    require(a['prefix']['after']==b['prefix']['after'],'delegated register boundary differs')
    require(a['prefix']['out'][:6]==b['prefix']['out'][:6],'callee arguments differ')
    changed=unit_delta(bytes.fromhex(natives[0]['unit_bytes']),b['unit_data'])
    pointer,capacity,length=struct.unpack_from('<3I',a['unit_data'],0xb8)
    require(0<length<=capacity<=4096,'invalid first path extent')
    borrowed=[dict(source='captured unit path capacity',address=pointer,size=capacity*16)]
    items=parse((first/'search-graph.bin').read_bytes())[1]
    pool=[words(data) for kind,owner,address,data in items if (kind,owner)==(6,5)]
    require(len(pool)==1,'missing recycler header')
    pointer,capacity,length,increment=pool[0]
    require(0<capacity<=4096 and length<=capacity,'invalid recycler extent')
    borrowed.append(dict(source='captured pool-5 pointer array',address=pointer,size=capacity*4))
    inv=decode_inventory((first/'memory-inventory.bin').read_bytes(),(first/'restore-prefix.bin').read_bytes())
    with (first/'memory-payload.bin').open('rb') as stream:
        pages=CapturedPages(stream,reports[0]['spans'],inv['rows'])
        header=pages.read(0xc8d8c0,16);require(header is not None,'missing recycler header 0xc8d8c0')
        pointer,capacity,length,increment=struct.unpack('<4I',header)
    require(0<capacity<=4096 and length<=capacity,'invalid extra recycler extent')
    borrowed.append(dict(source='captured pointer-array header 0xc8d8c0',address=pointer,size=capacity*4))
    return natives,contexts,reports,borrowed,changed


def experiment(r,c,baseline,natives,contexts,reports,caller):
    first=native_witness(r,c,baseline,natives[0]);require_agreement(first['comparison'])
    first_inputs,_=input_patch(r.regions,c['unit'],'limit',95)
    arguments=next(x for x in r.regions if x.name=='callee_arguments')
    modes=next(x for x in r.regions if x.name=='modes')
    native_before=bytes.fromhex(natives[0]['unit_bytes'])
    trials=[]
    for name in ('frozen','original_caller_collide'):
        attempts=[]
        for repeat in range(2):
            one=execute(r,registers(c),first_inputs);require(one['returned'],'first model refused')
            modeled=bytes(observe_bytes(r,c['unit'],344))
            require(modeled[OFFSET:OFFSET+2]==native_before[OFFSET:OFFSET+2],'first modeled collide differs')
            step=None
            if name=='original_caller_collide':
                advanced,step=caller.run(modeled)
                require(advanced[OFFSET:OFFSET+2]==contexts[1]['unit_data'][OFFSET:OFFSET+2],
                        'original instruction does not explain the native input delta')
                r.uc.mem_write(c['unit']+OFFSET,advanced[OFFSET:OFFSET+2])
            two=continue_once(r,registers(c),dict(callee_arguments=arguments.data,modes=modes.data))
            if two['returned']:
                two['unit_path_observation']=observe_unit_path(r,c['unit'])
                two['return_modes']=list(struct.unpack('<2I',bytes(observe_bytes(r,0xe85ec0,8))))
                two['comparison']=compare_boundary(natives[1],two)
            attempts.append(dict(caller_instruction=step,second=two))
        require(attempts[0]==attempts[1],'chain repeat differs')
        trials.append(dict(name=name,repeat_matches=True,**attempts[0]))
    require(trials[0]['second']['returned'],'frozen chain refused')
    frozen=trials[0]['second']['comparison']
    require_frozen_gap(frozen)
    require(trials[1]['second']['returned'],'caller-step chain refused')
    require_agreement(trials[1]['second']['comparison'])
    restored=execute(r,registers(c))
    require(restored['fingerprint']==baseline['last']['final_fingerprint'],'baseline did not restore')
    return dict(scope='Cross-input output comparison: first-capture world plus one original caller instruction; not native world advancement or same-input second replay',
                first_payload_sha256=reports[0]['sha256'],target_native_second_payload_sha256=reports[1]['sha256'],
                first_native_comparison=first['comparison'],baseline_restored=True,trials=trials)


def main():
    ap=argparse.ArgumentParser(description=__doc__)
    ap.add_argument('install',type=Path);ap.add_argument('capture_root',type=Path)
    args=ap.parse_args();natives,contexts,reports,borrowed,changed=prepare(args.install,args.capture_root)
    caller=CallerCollide(args.install/'riseofnations.exe',contexts[0]['unit'])
    result=explore(args.install,args.capture_root/'packet-0',services='malloc+memset+memcpy+free',
                   borrowed=[(x['address'],x['size']) for x in borrowed],mutable_arguments=True,observe_path=True,
                   on_prepared=lambda r,c,b:experiment(r,c,b,natives,contexts,reports,caller))
    result['native_boundary_delta']=dict(unit_changed_byte_offsets=changed,
        caller_stack_changes=[dict(index=i,first=a,second=b) for i,(a,b) in enumerate(zip(contexts[0]['prefix']['out'],contexts[1]['prefix']['out'])) if a!=b],
        caller_stack_replaced=False,delegated_registers_equal=True,callee_arguments_equal=True)
    result['borrowed_extent_evidence']=borrowed
    print(json.dumps(result,indent=2))

if __name__=='__main__':main()
