"""Explicit pointer/array pilots over PDB-decoded roots; no logger parity claim."""
import argparse,collections,hashlib,json,re,struct
from pathlib import Path
from typed_state import load_inputs,Payload,require
from typed_fields import Decoder,globals_from_dump


def select(rows,path):
    matches=[r for r in rows if re.sub(r'::<base:[0-9a-f]+>','',r['path'])==path]
    require(len(matches)==1,'missing or ambiguous declared field '+path)
    return matches[0]


def array_extent(rows,path,maximum):
    length=select(rows,path+'.length');capacity=select(rows,path+'.size');pointer=select(rows,path+'.list')
    require(length['status']==capacity['status']=='value' and pointer['status']=='pointer','invalid container field kinds')
    n=length['value'];cap=capacity['value']
    require(0<=n<=cap<=maximum and (not n or pointer['value']>=0x10000),'invalid explicit array extent')
    return n,pointer


def root_decode(name,symbols,types,memory):
    choices=[s for s in symbols if s['name']==name and s['address'] is not None]
    require(len(choices)==1,'missing or ambiguous root '+name);root=choices[0]
    decoder=Decoder(types,memory);decoder.decode(root['type_index'],root['address'],name)
    return decoder


def experiment(install,capture,types_json,globals_path):
    pe,types,report,inv,data,raw=load_inputs(install,capture,types_json)
    require(hashlib.sha256(globals_path.read_bytes()).hexdigest()==data['_source']['globals_sha256'],'globals export differs')
    symbols=globals_from_dump(globals_path.read_text(),pe);memory=Payload(capture/'memory-payload.bin',report)
    try:
        terrain=root_decode('terrain',symbols,types,memory);world=root_decode('world',symbols,types,memory)
        count,pointer=array_extent(terrain.rows,'terrain.master_land_heights',1000000)
        require(pointer['pointee']==0x40,'height element is not CodeView float32')
        width=select(world.rows,'world.tile_xs')['value']+1;height=select(world.rows,'world.tile_ys')['value']+1
        require(width>0 and height>0 and count==width*height,'height dimensions disagree')
        bits=memory.read(pointer['value'],count*4)
        # Follow a plain-struct pointer using only its declared pointee layout.
        wdata=select(world.rows,'world.wdata');require(wdata['status']=='pointer','world data not a pointer')
        cell=Decoder(types,memory);cell.decode(wdata['pointee'],wdata['value'],'world.wdata[0]')
        # One pointer target is observed. This is not a proof of the full dynamic extent.
        return dict(schema='typed-state-pilots-v1',payload_sha256=report['sha256'],frame=report['frame'],
                    capture_boundary='restore delegation',atomic_snapshot=False,logger_compared=False,
                    heights=dict(count=count,width=width,height=height,address=pointer['value'],
                                 exact_float32_le_hex=bits.hex(),sha256=hashlib.sha256(bits).hexdigest(),
                                 extent_contract='Array length/size/list plus world tile dimensions; explicit pilot, not inferred from pointer type'),
                    plain_struct_pointer=dict(declared_pointee=wdata['pointee'],elements_read=1,**cell.report()))
    finally:memory.close()


def main():
    ap=argparse.ArgumentParser(description=__doc__)
    for name in ('install','capture','types_json','globals_path'):ap.add_argument(name,type=Path)
    a=ap.parse_args();print(json.dumps(experiment(a.install,a.capture,a.types_json,a.globals_path),indent=2))
if __name__=='__main__':main()
