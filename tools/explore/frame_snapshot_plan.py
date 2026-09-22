"""Generate an external, PDB-bound end-frame collector plan (never commit output)."""
import argparse, hashlib, json
from pathlib import Path
from typed_state import PE, Types, require
from typed_fields import Decoder, globals_from_dump

ROOTS=('world','leaders','game_random','units','groups','terrain')

def generate(install, export, output):
    repo=Path(__file__).resolve().parents[2]
    require(not output.resolve().is_relative_to(repo),'generated plan must stay outside repository')
    raw=(install/'riseofnations.exe').read_bytes();pe=PE(raw)
    encoded=(export/'types.json').read_bytes();data=json.loads(encoded)
    manifest=json.loads((export/'manifest.json').read_text())['files']
    require(hashlib.sha256(encoded).hexdigest()==manifest['types.json'],'type manifest differs')
    require(hashlib.sha256((install/'sbl/rise.pdb').read_bytes()).hexdigest()==data['_source']['pdb_sha256'],'PDB differs')
    require(pe.identity==(data['PdbStream']['Guid'].strip('{}').upper(),data['PdbStream']['Age']),'PE/PDB identity differs')
    text=(export/'globals.txt').read_text()
    require(hashlib.sha256(text.encode()).hexdigest()==data['_source']['globals_sha256'],'globals differ')
    types=Types(data);symbols=globals_from_dump(text,pe);decoder=Decoder(types,None)
    def symbol(name):
        found=[s for s in symbols if s['name']==name and s['address'] is not None]
        require(len(found)==1,'ambiguous/missing symbol '+name);return found[0]
    def member(t,name):
        while types.record(types.resolve(t))['Kind']=='LF_MODIFIER':
            t=types.record(types.resolve(t))['Modifier']['ModifiedType']
        found=[f['DataMember'] for f in types.fields(t) if f['Kind']=='LF_MEMBER' and f['DataMember']['Name']==name]
        require(len(found)==1,'ambiguous/missing direct member '+name)
        require(decoder.size(found[0]['Type'])==4,'boundary member is not four bytes')
        return found[0]['FieldOffset']
    game=symbol('GameAccessConst::gamec');pointer=types.record(game['type_index'])
    require(pointer['Kind']=='LF_POINTER','gamec is not a pointer')
    gt=pointer['Pointer']['ReferentType'];log=symbol('game_log')
    roots=[dict(**symbol(n),size=decoder.size(symbol(n)['type_index'])) for n in ROOTS]
    plan=dict(schema='frame-snapshot-plan-v1',image_sha256=hashlib.sha256(raw).hexdigest(),
              types_sha256=hashlib.sha256(encoded).hexdigest(),roots=roots,
              game_slot=game['address'],frame_offset=member(gt,'frame'),logger=log['address'],
              start_offset=member(log['type_index'],'log_start_frame'),end_offset=member(log['type_index'],'log_end_frame'),
              hook_address=0x9329d0,hook_bytes=pe.read(0x9329d0,6).hex())
    # This exact six-byte absolute MOV is position independent in the non-ASLR image.
    require(bytes.fromhex(plan['hook_bytes'])[:2]==b'\x8b\x0d','unsupported end_frame prologue')
    require(sum(r['size'] for r in roots)+8<=524288,'root anchor budget exceeded')
    output.mkdir(parents=True,exist_ok=False)
    (output/'plan.json').write_text(json.dumps(plan,indent=2)+'\n')
    constants={'GAME_SLOT':plan['game_slot'],'FRAME_OFFSET':plan['frame_offset'],'LOGGER':plan['logger'],
               'START_OFFSET':plan['start_offset'],'END_OFFSET':plan['end_offset']}
    lines=['/* Generated from the owned PDB. Keep outside git. */']
    lines += [f'#define SNAP_{k} {v}u' for k,v in constants.items()]
    lines += ['#define SNAP_ROOTS {'+','.join('{'+str(r['address'])+'u,'+str(r['size'])+'u}' for r in roots)+'}']
    (output/'plan.h').write_text('\n'.join(lines)+'\n')
    return plan

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    for n in ('install','export','output'):p.add_argument(n,type=Path)
    a=p.parse_args();print(json.dumps(generate(a.install,a.export,a.output),indent=2))
