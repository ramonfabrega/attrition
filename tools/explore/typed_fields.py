"""PDB-driven retained field decoding with explicit unsupported/missing rows.

Pointers preserve addresses and pointee types; they do not imply an array extent
or allocation liveness. Float fields retain exact bits, not host float values.
"""
import collections,re,struct
from typed_state import require

# LLVM CodeView SimpleTypeKind; unsupported simple kinds are visible errors.
SIMPLE={0x10:(1,True),0x20:(1,False),0x70:(1,True),0x71:(2,False),
        0x68:(1,True),0x69:(1,False),0x11:(2,True),0x21:(2,False),
        0x72:(2,True),0x73:(2,False),0x12:(4,True),0x22:(4,False),
        0x74:(4,True),0x75:(4,False),0x13:(8,True),0x23:(8,False),
        0x76:(8,True),0x77:(8,False),0x30:(1,False),0x31:(2,False),
        0x32:(4,False),0x33:(8,False),0x40:(4,False),0x41:(8,False),
        0x7a:(2,False),0x7b:(4,False),0x7c:(1,False)}
NON_STORAGE={'LF_ONEMETHOD','LF_METHOD','LF_NESTTYPE','LF_ENUMERATE'}


def globals_from_dump(text,pe):
    pattern=r'S_GDATA32 \[size = \d+\] `(.+)`\n\s+type = (0x[0-9A-Fa-f]+).*?, addr = ([0-9]+):([0-9]+)'
    result=[];matches=re.findall(pattern,text)
    require(len(matches)==len(re.findall(r'\bS_GDATA32 \[',text)),'unparsed global symbol declaration')
    for name,typ,section,offset in matches:
        i=int(section)-1;offset=int(offset)
        valid=0<=i<len(pe.sections) and offset<pe.sections[i]['virtual_size']
        result.append(dict(name=name,type_index=int(typ,16),
                           address=pe.sections[i]['address']+offset if valid else None,
                           status='mapped' if valid else 'unresolved_symbol_address'))
    return result


class Decoder:
    def __init__(self,types,memory,budget=250000):
        self.types=types;self.memory=memory;self.budget=budget;self.rows=[];self.nonstorage=collections.Counter()
    def size(self,t):
        if t<4096:
            if t&0xf00:
                require(t&0xf00==0x400,'unsupported simple pointer mode');return 4
            require(t in SIMPLE,'unsupported simple type');return SIMPLE[t][0]
        t=self.types.resolve(t);r=self.types.record(t);k=r['Kind'];b=next(v for n,v in r.items() if n!='Kind')
        if k in ('LF_CLASS','LF_STRUCTURE','LF_UNION','LF_ARRAY'):return b['Size']
        if k=='LF_POINTER':
            size=(b['Attrs']>>13)&63;require(size==4,'non-32-bit pointer');return size
        if k=='LF_MODIFIER':return self.size(b['ModifiedType'])
        if k=='LF_ENUM':return self.size(b['UnderlyingType'])
        if k=='LF_BITFIELD':return self.size(b['Type'])
        raise ValueError('unsupported type '+k)
    def row(self,path,address,t,**kwargs):
        require(len(self.rows)<self.budget,'decoded field budget exceeded')
        self.rows.append(dict(path=path,address=address,type_index=t,**kwargs))
    def decode(self,t,address,path,depth=0,ancestors=()):
        require(depth<=64,'type recursion budget exceeded')
        try:self._decode(t,address,path,depth,ancestors)
        except ValueError as e:
            if 'budget exceeded' in str(e):raise
            self.row(path,address,t,status='unresolved',reason=str(e))
    def _decode(self,t,address,path,depth,ancestors):
        if t<4096:
            size=self.size(t);raw=self.memory.read(address,size)
            if t&0xf00:self.row(path,address,t,status='pointer',value=int.from_bytes(raw,'little'),pointee=t&255,extent='unknown');return
            if t in (0x40,0x41):self.row(path,address,t,status='float_bits',bits=raw.hex(),width=size*8);return
            self.row(path,address,t,status='value',value=int.from_bytes(raw,'little',signed=SIMPLE[t][1]));return
        t=self.types.resolve(t);r=self.types.record(t);k=r['Kind'];b=next(v for n,v in r.items() if n!='Kind')
        if k=='LF_MODIFIER':self.decode(b['ModifiedType'],address,path,depth+1,ancestors);return
        if k=='LF_ENUM':self.decode(b['UnderlyingType'],address,path,depth+1,ancestors);return
        if k=='LF_POINTER':
            raw=self.memory.read(address,self.size(t));mode=(b['Attrs']>>5)&7
            require(mode in (0,1,4),'member pointer requires ABI-specific decoding')
            self.row(path,address,t,status='pointer',value=int.from_bytes(raw,'little'),pointee=b['ReferentType'],extent='unknown');return
        if k=='LF_BITFIELD':
            raw=self.memory.read(address,self.size(t));bits=b['BitSize'];shift=b['BitOffset']
            require(0<bits and shift+bits<=len(raw)*8,'invalid bitfield')
            value=(int.from_bytes(raw,'little')>>shift)&((1<<bits)-1)
            require(b['Type'] in SIMPLE,'unsupported bitfield signedness')
            if SIMPLE[b['Type']][1] and value&(1<<(bits-1)):value-=1<<bits
            self.row(path,address,t,status='bitfield',value=value,width=bits);return
        if k=='LF_ARRAY':
            size=self.size(b['ElementType']);require(size>0 and b['Size']%size==0,'invalid array extent')
            for i in range(b['Size']//size):self.decode(b['ElementType'],address+i*size,f'{path}[{i}]',depth+1,ancestors)
            return
        require(k in ('LF_CLASS','LF_STRUCTURE','LF_UNION'),'unsupported type '+k)
        require((t,address) not in ancestors,'recursive inline layout')
        ancestors=(*ancestors,(t,address))
        for f in self.types.fields(t):
            kind=f['Kind'];body=next(v for n,v in f.items() if n!='Kind')
            if kind in NON_STORAGE:self.nonstorage[kind]+=1;continue
            if kind=='LF_MEMBER':
                try:require(0<=body['FieldOffset'] and body['FieldOffset']+self.size(body['Type'])<=b['Size'],'field outside declared layout')
                except ValueError as error:
                    self.row(path+'.'+body['Name'],address+body['FieldOffset'],body['Type'],status='unresolved',reason=str(error));continue
                self.decode(body['Type'],address+body['FieldOffset'],path+'.'+body['Name'],depth+1,ancestors)
            elif kind=='LF_BCLASS':self.decode(body['Type'],address+body['Offset'],path+f'::<base:{body["Type"]:x}>',depth+1,ancestors)
            elif kind in ('LF_VBCLASS','LF_IVBCLASS'):
                # Do not guess virtual-base adjustment until independently tested.
                self.row(path+'::<virtual-base>',address,t,status='unresolved',reason=kind,base_type=body['BaseType'])
            elif kind=='LF_VFUNCTAB':
                self.row(path+'::<vftable>',address,t,status='abi_metadata',reason=kind)
            elif kind=='LF_STMEMBER':
                self.row(path+'.'+body['Name'],None,body['Type'],status='static_member',reason='requires symbol address')
            else:self.row(path,address,t,status='unresolved',reason='unsupported field '+kind)
    def report(self):
        return dict(rows=self.rows,status_counts=dict(collections.Counter(r['status'] for r in self.rows)),
                    nonstorage_records=dict(self.nonstorage),declared_inline_fields_complete=not any(r['status']=='unresolved' for r in self.rows),
                    reachable_state_complete=False,
                    pointer_extent_inferred=False,union_active_member_inferred=False)
