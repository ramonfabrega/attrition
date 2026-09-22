"""Export owned PDB metadata outside the repository, bound to its source hash."""
import argparse,hashlib,json,subprocess,time
from pathlib import Path
import yaml


def sha(path):
    h=hashlib.sha256()
    with path.open('rb') as stream:
        for chunk in iter(lambda:stream.read(1024*1024),b''):h.update(chunk)
    return h.hexdigest()


def export(pdb,output,llvm):
    output=output.resolve();repo=Path(__file__).resolve().parents[2]
    if output==repo or repo in output.parents:raise ValueError('exports must stay outside the repository')
    output.mkdir(parents=True,exist_ok=False);start=time.monotonic();before=sha(pdb)
    commands={'types.yaml':['pdb2yaml','--minimal','--no-file-headers','--pdb-stream','--tpi-stream'],
              'identity.yaml':['pdb2yaml','--no-file-headers','--pdb-stream'],
              'globals.txt':['dump','--globals']}
    for name,args in commands.items():
        with (output/name).open('wb') as out:subprocess.run([llvm,*args,str(pdb)],stdout=out,check=True)
    data=yaml.load((output/'types.yaml').read_bytes(),Loader=yaml.CSafeLoader)
    identity=yaml.load((output/'identity.yaml').read_bytes(),Loader=yaml.CSafeLoader)
    data['PdbStream']=identity['PdbStream']
    if sha(pdb)!=before:raise ValueError('PDB changed during export')
    data['_source']=dict(pdb_sha256=before,types_yaml_sha256=sha(output/'types.yaml'),
                         globals_sha256=sha(output/'globals.txt'),
                         llvm_version=subprocess.check_output([llvm,'--version'],text=True).strip())
    (output/'types.json').write_text(json.dumps(data))
    manifest={p.name:sha(p) for p in output.iterdir() if p.is_file()}
    (output/'manifest.json').write_text(json.dumps(dict(files=manifest,seconds=time.monotonic()-start),indent=2)+'\n')
    return dict(output=str(output),type_records=len(data['TpiStream']['Records']),seconds=time.monotonic()-start)


def main():
    ap=argparse.ArgumentParser(description=__doc__);ap.add_argument('pdb',type=Path);ap.add_argument('output',type=Path)
    ap.add_argument('--llvm-pdbutil',default='llvm-pdbutil');a=ap.parse_args()
    print(json.dumps(export(a.pdb,a.output,a.llvm_pdbutil)))
if __name__=='__main__':main()
