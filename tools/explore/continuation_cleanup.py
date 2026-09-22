"""Bounded cleanup/recycler ablations between two native-matched replay models.

Diagnostic omissions only: nulling roots skips retirement and is not a valid
world transition or an implementation proposal. Retain generated JSON outside Git.
"""
import argparse,hashlib,json,struct,time
from pathlib import Path
from unicorn import UC_HOOK_CODE,UC_HOOK_MEM_READ
from compact_chain import load
from caller_collide import CallerCollide
from continued_call import continue_once
from defined_bytes import observe_bytes
from path_intervention import execute,input_patch
from payload_replay import explore,observe_unit_path
from replay_capsule import require
from restore_poststate import compare_boundary,require_agreement
from resume_frontier import registers

# Existing pinned-image PathFinderData.validlist/blocklist pointers and the
# two recycler Stack.length words. Root offsets are PDB-backed, never inferred.
VALID,BLOCK=0xe85e90,0xe85e8c
POOLS=(0xc8d9b8,0xc8d828)


def first_gap(a,b):
    index=next((i for i,(x,y) in enumerate(zip(a,b)) if x!=y),min(len(a),len(b)))
    if index==len(a)==len(b):return None
    return dict(index=index,left=a[max(0,index-4):index+3],right=b[max(0,index-4):index+3],
                left_count=len(a),right_count=len(b))


def patch_words(regions,changes):
    """Preserve every byte except declared whole words; combine shared regions."""
    patches={};seen=set()
    for address,value in changes:
        require(type(address) is int and address not in seen and address%4==0 and
                type(value) is int and 0<=value<2**32,'invalid word patch')
        seen.add(address)
        candidates=[r for r in regions if r.address<=address and address+4<=r.address+len(r.data)]
        require(len(candidates)==1,'patch word needs one declared region');r=candidates[0]
        require(r.writable and not r.scratch and not r.executable,'patch needs writable captured data')
        data=patches.setdefault(r.name,bytearray(r.data));struct.pack_into('<I',data,address-r.address,value)
    return {name:bytes(data) for name,data in patches.items()}


def initial_word(r,address):
    region=next(x for x in r.regions if x.address<=address and address+4<=x.address+len(x.data))
    return struct.unpack_from('<I',region.data,address-region.address)[0]


def require_findings(chain,trials):
    require(chain['instructions']==33494,'chain count changed')
    require([t['patched_words'] for t in trials]==[0,1,2,3,4,4],'ablation coverage differs')
    require([t['result']['instructions'] for t in trials]==[34845,33675,33339,33416,33572,33494],
            'measured ablation counts changed')
    require([None if t['first_gap_from_chain'] is None else t['first_gap_from_chain']['index'] for t in trials]==[445,480,5796,6220,3719,None],
            'measured first divergences changed')
    for t in trials:
        require(t['repeat_matches'],'ablation repeat failed');require_agreement(t['result']['comparison'])


def word(r,address):return struct.unpack('<I',bytes(observe_bytes(r,address,4)))[0]


def traced(r,run,unit,native):
    sequence=[];reads=[];current=None
    def code(u,address,size,opaque):
        nonlocal current
        current=address;sequence.append([address,size])
    def read(u,kind,address,size,value,opaque):
        if current in (0x454003,0x683b19,0x47a6ca,0x47a76a):
            reads.append(dict(instruction=current,address=address,size=size,bytes=bytes(u.mem_read(address,size)).hex()))
    hooks=[r.uc.hook_add(UC_HOOK_CODE,code),r.uc.hook_add(UC_HOOK_MEM_READ,read)]
    try:result=run()
    finally:
        for hook in hooks:r.uc.hook_del(hook)
    result.pop('seconds',None)
    require(result['returned'],'ablation refused: '+str(result['reason']))
    result['unit_path_observation']=observe_unit_path(r,unit)
    result['return_modes']=[word(r,0xe85ec0),word(r,0xe85ec4)]
    result['comparison']=compare_boundary(native,result);require_agreement(result['comparison'])
    result['instruction_trace_sha256']=hashlib.sha256(b''.join(struct.pack('<II',*row) for row in sequence)).hexdigest()
    result['branch_reads']=reads
    return result,sequence


def experiment(install,fixture,packet):
    started=time.monotonic();r,cs,ns,meta=load(install,fixture);c=cs[0]
    baseline=execute(r,registers(c));require(baseline['fingerprint']==meta['baseline_fingerprint'],'first baseline differs')
    inputs,_=input_patch(r.regions,c['unit'],'limit',95)
    first=execute(r,registers(c),inputs);require(first['returned'],'first call refused')
    updated,_=CallerCollide(install/'riseofnations.exe',c['unit']).run(bytes(observe_bytes(r,c['unit'],344)))
    r.uc.mem_write(c['unit']+0x88,updated[0x88:0x8a])
    roots=[word(r,VALID)+12,word(r,BLOCK)+12]
    fields=roots+list(POOLS);chain_values=[word(r,a) for a in fields]
    require(chain_values[:2]==[0,0],'chain roots are not empty')
    boundary={n:next(x.data for x in r.regions if x.name==n) for n in ('callee_arguments','modes')}
    chain,chain_seq=traced(r,lambda:continue_once(r,registers(c),boundary),c['unit'],ns[1])
    require(execute(r,registers(c))['fingerprint']==meta['baseline_fingerprint'],'first baseline did not restore')
    def prepared(second,context,base):
        require(base['payload_sha256']==meta['source_payloads'][1],'second packet identity differs')
        require(context['unit']==c['unit'] and registers(context)==registers(cs[1]),'second context differs')
        require([initial_word(second,VALID)+12,initial_word(second,BLOCK)+12]==roots,'active tree identities differ')
        # explore leaves its completed model live; input words below come from
        # captured region.data, and execute resets before every actual trial.
        original=[]
        for address in fields:
            region=next(x for x in second.regions if x.address<=address and address+4<=x.address+len(x.data))
            original.append(struct.unpack_from('<I',region.data,address-region.address)[0])
        trials=[]
        for count in range(5):
            patches=patch_words(second.regions,[(a,0) for a in fields[:count]])
            one,seq=traced(second,lambda:execute(second,registers(context),patches),context['unit'],ns[1])
            repeat,repeat_seq=traced(second,lambda:execute(second,registers(context),patches),context['unit'],ns[1])
            require(one==repeat and seq==repeat_seq,'ablation does not repeat')
            trials.append(dict(patched_words=count,result=one,repeat_matches=True,first_gap_from_chain=first_gap(chain_seq,seq)))
        patches=patch_words(second.regions,list(zip(fields,chain_values)))
        matched,seq=traced(second,lambda:execute(second,registers(context),patches),context['unit'],ns[1])
        repeat,repeat_seq=traced(second,lambda:execute(second,registers(context),patches),context['unit'],ns[1])
        require(matched==repeat and seq==repeat_seq==chain_seq,'matched-entry trace differs')
        trials.append(dict(patched_words=4,matched_entry=True,result=matched,repeat_matches=True,
                           first_gap_from_chain=None,
                           fingerprint_differences_from_chain=[k for k in chain['fingerprint']
                               if chain['fingerprint'][k]!=matched['fingerprint'][k]]))
        restored=execute(second,registers(context))
        require(restored['fingerprint']==base['last']['final_fingerprint'],'second baseline did not restore')
        require_findings(chain,trials)
        return dict(fields=[dict(address=a,captured_entry=b,chain_entry=v,diagnostic_override=0) for a,b,v in zip(fields,original,chain_values)],
                    trials=trials,second_baseline_restored=True)
    result=explore(install,packet,services='malloc+memset+memcpy+free',
                   borrowed=[tuple(x) for x in meta['model']['borrowed']],mutable_arguments=True,
                   observe_path=True,on_prepared=prepared)
    return dict(schema='continuation-cleanup-v1',scope='Diagnostic root/length omissions; not valid native state or a world-transition repair',
                first_payload=meta['source_payloads'][0],second_payload=result['payload_sha256'],
                chain=chain,first_baseline_restored=True,experiment=result['prepared_experiments'],
                preparation_seconds=result['seconds'],total_seconds=time.monotonic()-started)


def main():
    ap=argparse.ArgumentParser(description=__doc__)
    for name in ('install','fixture','second_packet'):ap.add_argument(name,type=Path)
    a=ap.parse_args();print(json.dumps(experiment(a.install,a.fixture,a.second_packet),indent=2))
if __name__=='__main__':main()
