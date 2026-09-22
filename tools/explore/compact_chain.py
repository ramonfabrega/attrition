"""Export/replay a derived, bounded L82 fixture without its broad payloads.

This is a local derived cache, not a new native witness or authenticated evidence.
Original code is loaded only from the pinned owned image. Initial region bytes,
permissions, undefined scratch semantics and model quotas survive round-trip.
The source pair is fully validated at build time; replay verifies the retained
fixture hashes and golden results, not the absent broad capture again.
"""
import argparse,hashlib,json,shutil,time
from pathlib import Path
from bounded_call import Region
from caller_collide import CallerCollide
from modeled_lifetime import LifetimeCall
from native_chain_gap import prepare,experiment
from payload_replay import explore,original_code,fingerprint,IMAGE_SHA256
from replay_capsule import require,digest
from restore_context import decode_context
from restore_poststate import decode_post
from resume_frontier import ENTRY,RETURN,registers

SYNTH={'_free_service','_memcpy_service','_memset_service','_malloc_service','_malloc_arena','_thread_gdt'}
MAX_DATA=8*1024*1024
PARAMS=('malloc_target','memset_target','memcpy_target','free_target','arena_address','arena_size',
        'max_allocations','max_fill_bytes','max_fills','max_copy_bytes','max_copies','max_frees','budget')
SIDECARS=tuple(f'{i}-{name}' for i in (0,1) for name in ('restore-prefix.bin','restore-context.bin','restore-poststate.bin'))


def sha(data):return hashlib.sha256(data).hexdigest()


def encode_regions(regions):
    entries=[];blob=bytearray()
    for r in regions:
        if r.name in SYNTH:continue
        require(not r.name.startswith('_'),'unknown generated region')
        storage='image' if r.executable else 'zero' if r.scratch else 'data'
        row=dict(name=r.name,address=r.address,size=len(r.data),writable=r.writable,
                 executable=r.executable,scratch=r.scratch,storage=storage,sha256=sha(r.data))
        if storage=='zero':require(not any(r.data),'nonzero initial scratch backing')
        if storage=='data':row['offset']=len(blob);blob.extend(r.data)
        entries.append(row)
    require(len(entries)<=8192 and len(blob)<=MAX_DATA,'compact input cap exceeded')
    return entries,bytes(blob)


def decode_regions(entries,blob,code):
    require(isinstance(entries,list) and 0<len(entries)<=8192 and len(blob)<=MAX_DATA,'invalid region inventory')
    originals={r.name:r for r in code};used=set();names=set();regions=[];offset=0;total=0
    for row in entries:
        require(set(row)=={'name','address','size','writable','executable','scratch','storage','sha256'}|
                ({'offset'} if row.get('storage')=='data' else set()),'invalid region fields')
        name=row['name'];size=row['size'];address=row['address'];storage=row['storage']
        require(isinstance(name,str) and 0<len(name)<=128 and not name.startswith('_') and name not in names,'invalid region name')
        names.add(name)
        require(type(size) is int and 0<size and type(address) is int and 4096<=address<address+size<=2**32,'invalid region extent')
        require(all(type(row[k]) is bool for k in ('writable','executable','scratch')),'invalid region flags')
        if storage=='image':
            require(name in originals and row['executable'] and not row['writable'] and not row['scratch'],'invalid image region')
            original=originals[name]
            require((address,size)==(original.address,len(original.data)),'image region differs')
            data=original.data;used.add(name)
        else:
            total+=size;require(total<=MAX_DATA and not row['executable'],'invalid data extent')
            if storage=='zero':
                require(row['scratch'] and row['writable'],'invalid scratch region');data=bytes(size)
            else:
                require(storage=='data' and not row['scratch'] and type(row['offset']) is int and row['offset']==offset,
                        'invalid packed data extent')
                require(offset+size<=len(blob),'truncated packed data');data=blob[offset:offset+size];offset+=size
        require(sha(data)==row['sha256'],'region bytes differ')
        regions.append(Region(name,address,data,row['writable'],row['executable'],row['scratch']))
    require(offset==len(blob) and used==set(originals),'missing image region or trailing data')
    # Constructors subsequently check byte overlap, duplicate synthetic regions,
    # borrowed extents, stack/FS setup and model policy invariants.
    return regions


def model_parameters(r,context):
    values={name:getattr(r,name) for name in PARAMS}
    values.update(fs_address=context['teb'],gdt_address=next(x.address for x in r.regions if x.name=='_thread_gdt'),
                  borrowed=[list(x) for x in r.borrowed])
    return values


def validate_parameters(p):
    require(set(p)==set(PARAMS)|{'fs_address','gdt_address','borrowed'},'model parameter fields differ')
    require(all(type(p[k]) is int and 0<p[k]<2**32 for k in p if k!='borrowed'),'invalid model parameter')
    require(p['arena_size']==1024*1024 and p['max_allocations']==1024 and
            p['max_fill_bytes']==p['max_copy_bytes']==1024*1024 and
            p['max_fills']==p['max_copies']==p['max_frees']==1024 and p['budget']<=1000000,'unsupported model quotas')
    require(isinstance(p['borrowed'],list) and len(p['borrowed'])<=64,'borrow count exceeds cap')
    for pair in p['borrowed']:
        require(isinstance(pair,list) and len(pair)==2 and all(type(v) is int for v in pair) and
                4096<=pair[0]<pair[0]+pair[1]<=2**32 and 0<pair[1]<=1024*1024,'invalid borrowed extent')
    require(sum(n for _,n in p['borrowed'])<=1024*1024,'borrow bytes exceed cap')


def load(install,directory,*,building=False):
    require(building or not (directory/'.incomplete').exists(),'incomplete compact fixture')
    meta_path=directory/'fixture.json';require(meta_path.stat().st_size<=4*1024*1024,'fixture metadata exceeds cap')
    meta=json.loads(meta_path.read_text())
    require(meta['schema']=='compact-native-chain-v1' and meta['image_sha256']==IMAGE_SHA256 and
            digest(install/'riseofnations.exe')==IMAGE_SHA256,'fixture/image identity differs')
    require(meta['fxsave_imported'] is False and meta['extended_state_perturbed'] is False,'unsupported CPU state policy')
    require(set(meta['files'])==set(SIDECARS)|{'regions.bin'},'fixture sidecar set differs')
    for name,expected in meta['files'].items():
        path=directory/name;require(path.stat().st_size<=MAX_DATA and digest(path)==expected,'fixture file differs: '+name)
    validate_parameters(meta['model'])
    contexts=[];natives=[]
    for i in (0,1):
        prefix=(directory/f'{i}-restore-prefix.bin').read_bytes()
        contexts.append(decode_context((directory/f'{i}-restore-context.bin').read_bytes(),prefix))
        natives.append(decode_post((directory/f'{i}-restore-poststate.bin').read_bytes(),prefix))
    require(contexts[0]['teb']==meta['model']['fs_address'],'thread boundary differs')
    regions=decode_regions(meta['regions'],(directory/'regions.bin').read_bytes(),original_code((install/'riseofnations.exe').read_bytes()))
    runner=LifetimeCall(regions,RETURN,**meta['model'])
    return runner,contexts,natives,meta


def replay(install,directory,*,building=False):
    started=time.monotonic();r,contexts,natives,meta=load(install,directory,building=building)
    c=contexts[0];r.run(ENTRY,registers(c));base=fingerprint(r,None)
    require(base==meta['baseline_fingerprint'],'compact baseline fingerprint differs')
    baseline=dict(payload_sha256=meta['source_payloads'][0],last=dict(final_fingerprint=base))
    reports=[dict(sha256=h) for h in meta['source_payloads']]
    result=experiment(r,c,baseline,natives,contexts,reports,CallerCollide(install/'riseofnations.exe',c['unit']))
    require(result==meta['reference_experiment'],'compact chain differs from source experiment')
    return dict(schema='compact-native-chain-replay-v1',fixture_sha256=digest(directory/'fixture.json'),
                source_payloads=meta['source_payloads'],baseline_fingerprint_equal=True,
                complete_experiment_equal=True,dependency_discovery=False,broad_payload_reads=0,
                experiment=result,seconds=time.monotonic()-started)


def build(install,root,directory):
    started=time.monotonic()
    require(not directory.resolve().is_relative_to(Path(__file__).resolve().parents[2]),
            'derived game data must stay outside repository')
    require(not directory.exists(),'compact destination must be fresh')
    natives,contexts,reports,borrowed,_=prepare(install,root)
    directory.mkdir(parents=True);(directory/'.incomplete').write_text('Build verification has not finished.\n')
    def export(r,c,baseline):
        require(not baseline['fxsave_imported'] and not baseline['extended_state_perturbed'],'unsupported CPU state policy')
        reference=experiment(r,c,baseline,natives,contexts,reports,CallerCollide(install/'riseofnations.exe',c['unit']))
        entries,blob=encode_regions(r.regions)
        (directory/'regions.bin').write_bytes(blob)
        for i in (0,1):
            for name in ('restore-prefix.bin','restore-context.bin','restore-poststate.bin'):
                shutil.copyfile(root/f'packet-{i}'/name,directory/f'{i}-{name}')
        meta=dict(schema='compact-native-chain-v1',image_sha256=IMAGE_SHA256,
                  fxsave_imported=False,extended_state_perturbed=False,
                  source_capture=str(root.resolve()),source_trace_sha256=digest(root/'map-14/rontrace.log'),
                  source_payloads=[p['sha256'] for p in reports],
                  scope='Derived locally validated cache; no fresh native proof or whole-world fidelity',
                  model=model_parameters(r,c),regions=entries,baseline_fingerprint=baseline['last']['final_fingerprint'],
                  reference_experiment=reference,
                  files={name:digest(directory/name) for name in (*SIDECARS,'regions.bin')})
        (directory/'fixture.json').write_text(json.dumps(meta,indent=2)+'\n')
        verified=replay(install,directory,building=True)
        (directory/'build-verification.json').write_text(json.dumps(verified,indent=2)+'\n')
        (directory/'.incomplete').unlink()
        return dict(directory=str(directory),input_bytes=len(blob),regions=len(entries),
                    complete_experiment_equal=True,cold_replay_seconds=verified['seconds'])
    result=explore(install,root/'packet-0',services='malloc+memset+memcpy+free',
                   borrowed=[(x['address'],x['size']) for x in borrowed],mutable_arguments=True,observe_path=True,on_prepared=export)
    result['preparation_seconds']=result.pop('seconds')
    result['total_build_seconds']=time.monotonic()-started
    return result


def main():
    ap=argparse.ArgumentParser(description=__doc__);ap.add_argument('action',choices=('build','replay'))
    ap.add_argument('install',type=Path);ap.add_argument('directory',type=Path)
    ap.add_argument('--capture-root',type=Path);args=ap.parse_args()
    if args.action=='build':
        if args.capture_root is None:ap.error('build requires --capture-root')
        result=build(args.install,args.capture_root,args.directory)
    else:result=replay(args.install,args.directory)
    print(json.dumps(result,indent=2))

if __name__=='__main__':main()
