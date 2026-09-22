"""Experimental PE32 RTTI/PDB join and retained-payload object census.

No live process access. All exports, caches and decoded output stay outside Git.
Vtable hits are candidates, never proof of allocation liveness.
"""
import argparse,collections,hashlib,json,re,struct,time,uuid
from pathlib import Path


def require(ok,message):
    if not ok:raise ValueError(message)


class PE:
    def __init__(self,raw):
        raw=bytes(raw);self.raw=raw;require(raw[:2]==b'MZ','not PE')
        pe=struct.unpack_from('<I',raw,60)[0]
        require(raw[pe:pe+4]==b'PE\0\0','not PE')
        opt=pe+24;require(struct.unpack_from('<H',raw,opt)[0]==0x10b,'PE32 required')
        self.base=struct.unpack_from('<I',raw,opt+28)[0]
        count=struct.unpack_from('<H',raw,pe+6)[0];osize=struct.unpack_from('<H',raw,pe+20)[0]
        self.sections=[]
        for i in range(count):
            p=opt+osize+40*i;vsize,rva,size,off=struct.unpack_from('<4I',raw,p+8)
            require(off+size<=len(raw),'truncated PE section')
            self.sections.append(dict(address=self.base+rva,size=size,offset=off,virtual_size=vsize,
                                      flags=struct.unpack_from('<I',raw,p+36)[0]))
        debug_rva,debug_size=struct.unpack_from('<2I',raw,opt+96+6*8)
        ids=[]
        for at in range(0,debug_size,28):
            entry=self.read(self.base+debug_rva+at,28)
            if struct.unpack_from('<I',entry,12)[0]!=2:continue
            n,rva,_=struct.unpack_from('<3I',entry,16);cv=self.read(self.base+rva,n)
            if cv[:4]==b'RSDS':ids.append((str(uuid.UUID(bytes_le=cv[4:20])).upper(),struct.unpack_from('<I',cv,20)[0]))
        require(len(ids)==1,'unique RSDS identity required');self.identity=ids[0]
    def read(self,address,size):
        for s in self.sections:
            delta=address-s['address']
            if 0<=delta and delta+size<=s['size']:
                return self.raw[s['offset']+delta:s['offset']+delta+size]
        raise ValueError('PE bytes unavailable')
    def executable(self,address):
        return any(s['flags']&0x20000000 and s['address']<=address<s['address']+s['size'] for s in self.sections)
    def words(self):
        for s in self.sections:
            if s['flags']&0x20000000:continue
            data=self.raw[s['offset']:s['offset']+s['size']]
            for offset in range(0,len(data)-3,4):yield s['address']+offset,struct.unpack_from('<I',data,offset)[0]
    def rtti(self):
        # PE32 absolute-pointer COL: signature, subobject offset, constructor
        # displacement, type descriptor, class hierarchy descriptor.
        result={}
        for slot,locator in self.words():
            try:
                sig,offset,cd,td,chd=struct.unpack('<5I',self.read(locator,20))
                if sig!=0 or offset>1048576 or cd:continue
                hs,attrs,count,array=struct.unpack('<4I',self.read(chd,16))
                if hs or attrs&~7 or not 1<=count<=4096:continue
                bases=self.read(array,count*4);first=struct.unpack_from('<I',bases)[0]
                if struct.unpack_from('<I',self.read(first,4))[0]!=td:continue
                name_bytes=bytearray()
                for i in range(4096):
                    c=self.read(td+8+i,1)
                    if c==b'\0':break
                    name_bytes+=c
                else:continue
                name=name_bytes.decode('ascii')
                if not re.fullmatch(r'\.\?A[UV].+@@',name):continue
                vtable=slot+4;target=struct.unpack('<I',self.read(vtable,4))[0]
                if not self.executable(target):continue
                result[vtable]=dict(locator=locator,offset=offset,type_descriptor=td,unique_name=name)
            except (ValueError,UnicodeError,struct.error):continue
        return result


class Types:
    def __init__(self,data):
        self.records=data['TpiStream']['Records'];self.unique=collections.defaultdict(list);self.names=collections.defaultdict(list)
        for i,r in enumerate(self.records,4096):
            if r['Kind'] not in ('LF_CLASS','LF_STRUCTURE','LF_UNION'):continue
            body=r.get('Class',r.get('Union'))
            if 'ForwardReference' in body.get('Options',[]):continue
            self.unique[body.get('UniqueName','')].append(i);self.names[body['Name']].append(i)
    def record(self,index):
        require(4096<=index<4096+len(self.records),'unknown type index')
        return self.records[index-4096]
    def resolve(self,index):
        r=self.record(index)
        if r['Kind'] in ('LF_CLASS','LF_STRUCTURE','LF_UNION'):
            b=r.get('Class',r.get('Union'))
            if 'ForwardReference' in b.get('Options',[]):
                targets=self.unique.get(b.get('UniqueName',''),[])
                require(len(targets)==1,'missing or ambiguous forward type')
                return targets[0]
        return index
    def fields(self,index):
        r=self.record(self.resolve(index));body=r.get('Class',r.get('Union'))
        require(body is not None,'not a class/struct/union')
        field_index=body['FieldList']
        if not field_index:return []
        field=self.record(field_index);require(field['Kind']=='LF_FIELDLIST','not a field list')
        return field['FieldList']
    def join(self,rtti):
        joined={};unmatched=collections.Counter()
        for address,r in rtti.items():
            targets=self.unique.get(r['unique_name'],[])
            if len(targets)!=1:unmatched['ambiguous' if targets else 'missing']+=1;continue
            t=targets[0];body=self.record(t).get('Class',self.record(t).get('Union'))
            if r['offset']>=body['Size']:unmatched['invalid_extent']+=1;continue
            joined[address]=dict(**r,type_index=t,name=body['Name'],size=body['Size'])
        return joined,dict(unmatched)


class Payload:
    def __init__(self,path,report):
        self.stream=path.open('rb');self.spans=report['spans']
    def close(self):self.stream.close()
    def read(self,address,size):
        require(0<=size and 0<=address<=address+size<=2**32,'invalid retained extent')
        for s in self.spans:
            base=int(s['base'],16)
            if base<=address and address+size<=base+s['bytes']:
                self.stream.seek(s['file_offset']+address-base);data=self.stream.read(size)
                require(len(data)==size,'short retained read');return data
        raise ValueError('retained bytes unavailable')


def union_size(extents):
    total=0;end=0
    for start,stop in sorted(extents):
        total+=max(0,stop-max(start,end));end=max(end,stop)
    return total


def scan_candidates(payload,joined,rows,cap=200000):
    import numpy as np
    require(joined,'no joined vtables')
    lo,hi=min(joined),max(joined);lookup=np.zeros(hi-lo+1,dtype=np.bool_)
    for address in joined:lookup[address-lo]=True
    objects={};hits=0;incomplete=0
    for span in payload.spans:
        base=int(span['base'],16);kind=rows[span['inventory_index']][6]
        for off in range(0,span['bytes'],1024*1024):
            size=min(1024*1024,span['bytes']-off);data=payload.read(base+off,size)
            require((base+off)%4==0 and size%4==0,'unaligned census span')
            words=np.frombuffer(data,dtype='<u4');positions=np.flatnonzero((words>=lo)&(words<=hi))
            positions=positions[lookup[words[positions]-lo]]
            for pos in positions:
                address=base+off+int(pos)*4;vtable=int(words[pos]);meta=joined[vtable];start=address-meta['offset'];hits+=1
                try:payload.read(start,meta['size'])
                except ValueError:incomplete+=1;continue
                key=(start,meta['type_index'])
                if key not in objects:
                    require(len(objects)<cap,'candidate cap exceeded')
                    objects[key]=dict(address=start,type_index=meta['type_index'],name=meta['name'],size=meta['size'],memory_kind=kind,vptrs=[])
                objects[key]['vptrs'].append(dict(address=address,vtable=vtable,offset=meta['offset']))
    result=list(objects.values());private=[r for r in result if r['memory_kind']==0x20000]
    return dict(objects=result,vptr_hits=hits,incomplete_extents=incomplete,private_candidates=len(private),
                private_candidate_union_bytes=union_size((r['address'],r['address']+r['size']) for r in private),
                candidate_classes=dict(collections.Counter(r['name'] for r in result)),
                live_allocation_count=None,live_allocation_coverage=None,liveness_proven=False)


def load_bound_types(install,types_json):
    raw=(install/'riseofnations.exe').read_bytes();pe=PE(raw)
    metadata=types_json.read_bytes();manifest=json.loads((types_json.parent/'manifest.json').read_text())
    require(manifest['files'].get(types_json.name)==hashlib.sha256(metadata).hexdigest(),'type export manifest differs')
    data=json.loads(metadata);types=Types(data)
    require(data.get('_source',{}).get('pdb_sha256')==hashlib.sha256((install/'sbl/rise.pdb').read_bytes()).hexdigest(),'unbound or different PDB export')
    pdb=data['PdbStream'];require(pe.identity[0]==pdb['Guid'].strip('{}').upper() and pe.identity[1]==pdb['Age'],'PDB/PE identity differs')
    return pe,types,data,raw


def load_inputs(install,capture,types_json):
    # Revalidate source packet; do not trust a stale index/cache of payload offsets.
    from memory_payload import validate
    from memory_inventory import decode
    report=validate(install,capture)
    inv=decode((capture/'memory-inventory.bin').read_bytes(),(capture/'restore-prefix.bin').read_bytes())
    pe,types,data,raw=load_bound_types(install,types_json)
    return pe,types,report,inv,data,raw


def main():
    ap=argparse.ArgumentParser(description=__doc__)
    ap.add_argument('install',type=Path);ap.add_argument('capture',type=Path);ap.add_argument('types_json',type=Path)
    ap.add_argument('--globals',dest='globals_path',type=Path)
    ap.add_argument('--root',action='append',default=[])
    a=ap.parse_args();start=time.monotonic()
    pe,types,report,inv,data,raw=load_inputs(a.install,a.capture,a.types_json)
    rtti=pe.rtti();joined,unmatched=types.join(rtti);payload=Payload(a.capture/'memory-payload.bin',report)
    try:
        scan=scan_candidates(payload,joined,inv['rows']);roots=[]
        if a.globals_path:
            from typed_fields import Decoder,globals_from_dump
            require(hashlib.sha256(a.globals_path.read_bytes()).hexdigest()==data['_source']['globals_sha256'],'globals export differs')
            globals=globals_from_dump(a.globals_path.read_text(),pe)
            for name in a.root:
                selected=[g for g in globals if g['name']==name]
                require(len(selected)==1 and selected[0]['address'] is not None,'missing or ambiguous global root '+name)
                root=selected[0];decoder=Decoder(types,payload)
                decoder.decode(root['type_index'],root['address'],name)
                roots.append(dict(root=root,**decoder.report()))
        else:require(not a.root,'root selection requires bound globals export')
    finally:payload.close()
    print(json.dumps(dict(schema='typed-state-census-v1',image_sha256=hashlib.sha256(raw).hexdigest(),
        types_json_sha256=hashlib.sha256(a.types_json.read_bytes()).hexdigest(),payload_sha256=report['sha256'],
        frame=report['frame'],capture_boundary='restore delegation, not end frame',atomic_snapshot=False,
        type_records=len(types.records),complete_unique_names=len(types.unique),rtti_vtables=len(rtti),
        joined_vtables=len(joined),unmatched_vtables=unmatched,vtables=joined,scan=scan,roots=roots,seconds=time.monotonic()-start),indent=2))
if __name__=='__main__':main()
