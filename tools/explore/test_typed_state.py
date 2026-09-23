import copy,io,struct,unittest,uuid
from typed_state import PE,Types,Payload,scan_candidates,union_size
from typed_fields import Decoder,globals_from_dump
from typed_state_pilots import array_extent,select


def image():
    raw=bytearray(0xc00);raw[:2]=b'MZ';struct.pack_into('<I',raw,60,0x80);raw[0x80:0x84]=b'PE\0\0'
    struct.pack_into('<H',raw,0x86,2);struct.pack_into('<H',raw,0x94,224)
    opt=0x98;struct.pack_into('<H',raw,opt,0x10b);struct.pack_into('<I',raw,opt+28,0x400000)
    struct.pack_into('<2I',raw,opt+96+48,0x2000,28)
    for i,(rva,size,off,flags) in enumerate(((0x1000,0x200,0x200,0x60000020),(0x2000,0x800,0x400,0x40000040))):
        p=opt+224+i*40;struct.pack_into('<4I',raw,p+8,size,rva,size,off);struct.pack_into('<I',raw,p+36,flags)
    struct.pack_into('<4I',raw,0x40c,2,28,0x2040,0x440)
    raw[0x440:0x444]=b'RSDS';raw[0x444:0x454]=uuid.UUID(int=1).bytes_le;struct.pack_into('<I',raw,0x454,1)
    struct.pack_into('<5I',raw,0x600,0,0,0,0x402240,0x402280)
    raw[0x648:0x652]=b'.?AVToy@@\0'
    struct.pack_into('<4I',raw,0x680,0,0,1,0x4022b0);struct.pack_into('<I',raw,0x6b0,0x4022c0)
    struct.pack_into('<I',raw,0x6c0,0x402240);struct.pack_into('<2I',raw,0x700,0x402200,0x401000)
    return raw


def toy(fields=None):
    fields=fields if fields is not None else [dict(Kind='LF_MEMBER',DataMember=dict(Type=0x74,FieldOffset=4,Name='count'))]
    return {'TpiStream':{'Records':[
        dict(Kind='LF_FIELDLIST',FieldList=fields),
        dict(Kind='LF_CLASS',Class=dict(Name='Toy',UniqueName='.?AVToy@@',Options=[],Size=12,FieldList=4096)),
        dict(Kind='LF_CLASS',Class=dict(Name='Toy',UniqueName='.?AVToy@@',Options=['ForwardReference'],Size=0,FieldList=0))]}}


class Memory:
    def __init__(self,data):self.data=data
    def read(self,address,size):
        for base,data in self.data.items():
            if base<=address and address+size<=base+len(data):return data[address-base:address-base+size]
        raise ValueError('retained bytes unavailable')


class TypedStateTests(unittest.TestCase):
    def test_rtti_chain_and_pdb_forward_join(self):
        pe=PE(image());r=pe.rtti();self.assertEqual(list(r),[0x402304])
        types=Types(toy());joined,miss=types.join(r);self.assertFalse(miss)
        self.assertEqual(joined[0x402304]['type_index'],4097);self.assertEqual(types.resolve(4098),4097)
        self.assertEqual(pe.identity,(str(uuid.UUID(int=1)).upper(),1))
    def test_rtti_corruptions_are_not_promoted(self):
        for at,value in ((0x600,1),(0x604,0x200000),(0x608,4),(0x60c,0),(0x610,0),
                         (0x680,1),(0x684,8),(0x688,0),(0x68c,0),(0x6c0,0),(0x704,0x402280)):
            raw=image();struct.pack_into('<I',raw,at,value)
            with self.subTest(at=at):self.assertFalse(PE(raw).rtti())
    def test_secondary_vptr_normalizes_but_does_not_claim_liveness(self):
        raw=image();struct.pack_into('<I',raw,0x604,4);pe=PE(raw);joined,_=Types(toy()).join(pe.rtti())
        payload=Memory({0x10000:struct.pack('<3I',9,0x402304,7)})
        payload.spans=[dict(base='0x10000',bytes=12,inventory_index=0)]
        rows=[(0,0,0,0,0,0,0x20000)]
        report=scan_candidates(payload,joined,rows)
        self.assertEqual(report['objects'][0]['address'],0x10000)
        self.assertEqual(report['private_candidate_union_bytes'],12)
        self.assertIsNone(report['live_allocation_coverage']);self.assertFalse(report['liveness_proven'])
        with self.assertRaisesRegex(ValueError,'cap'):scan_candidates(payload,joined,rows,cap=0)
    def test_ambiguous_type_match_refuses(self):
        d=toy();d['TpiStream']['Records'].append(copy.deepcopy(d['TpiStream']['Records'][1]))
        t=Types(d);self.assertEqual(t.join(PE(image()).rtti())[1],{'ambiguous':1})
        with self.assertRaises(ValueError):t.resolve(4098)
    def test_every_declared_data_field_is_decoded_or_unresolved(self):
        fields=[dict(Kind='LF_MEMBER',DataMember=dict(Type=t,FieldOffset=o,Name=n)) for t,o,n in
                ((0x74,0,'signed'),(0x40,4,'float'),(0x474,8,'pointer'),(0x74,12,'outside'))]
        fields.append(dict(Kind='LF_VBCLASS',VirtualBaseClass=dict(BaseType=4098,VBPtrOffset=0,VTableIndex=1)))
        d=Decoder(Types(toy(fields)),Memory({0x10000:struct.pack('<3I',0xffffffff,0x7fc00001,0x20000)}))
        d.decode(4097,0x10000,'root');r=d.report();self.assertEqual(len(r['rows']),5)
        self.assertEqual(r['rows'][0]['value'],-1);self.assertEqual(r['rows'][1]['bits'],'0100c07f')
        self.assertEqual(r['rows'][2]['pointee'],0x74)
        self.assertEqual(r['status_counts']['unresolved'],2);self.assertFalse(r['declared_inline_fields_complete'])
    def test_missing_bytes_and_budgets_are_not_success(self):
        d=Decoder(Types(toy()),Memory({}));d.decode(4097,0x10000,'root')
        self.assertFalse(d.report()['declared_inline_fields_complete'])
        d=Decoder(Types(toy()),Memory({0x10000:bytes(12)}),budget=0)
        with self.assertRaisesRegex(ValueError,'budget'):d.decode(4097,0x10000,'root')
    def test_global_symbols_preserve_unresolved_addresses(self):
        text='S_GDATA32 [size = 24] `root`\n type = 0x1001 (Toy), addr = 0002:48\nS_GDATA32 [size = 24] `unknown`\n type = 0x1001 (Toy), addr = 0000:0\n'
        rows=globals_from_dump(text,PE(image()));self.assertEqual(rows[0]['address'],0x402030)
        self.assertIsNone(rows[1]['address']);self.assertEqual(rows[1]['status'],'unresolved_symbol_address')
    def test_embedded_symbol_quotes_and_unparsed_declarations(self):
        text="S_GDATA32 [size = 64] ``helper'::`2'::storage`\n type = 0x0023 (unsigned __int64), addr = 0000:0000\n"
        rows=globals_from_dump(text,PE(image()))
        self.assertEqual(rows[0]['name'],"`helper'::`2'::storage")
        with self.assertRaisesRegex(ValueError,'unparsed'):
            globals_from_dump(text+"S_GDATA32 [size = 24] `lost`\n invalid descriptor\n",PE(image()))
    def test_array_extent_is_explicit_and_refuses_ambiguous_or_invalid_fields(self):
        rows=[dict(path='x::<base:1000>.length',status='value',value=3),
              dict(path='x.size',status='value',value=4),
              dict(path='x.list',status='pointer',value=0x20000,pointee=0x40)]
        self.assertEqual(array_extent(rows,'x',4)[0],3)
        with self.assertRaises(ValueError):array_extent(rows,'x',2)
        with self.assertRaises(ValueError):select(rows+[rows[0]],'x.length')
        rows[0]['value']=5
        with self.assertRaises(ValueError):array_extent(rows,'x',10)
        rows[0]['value']=1;rows[2]['value']=0
        with self.assertRaises(ValueError):array_extent(rows,'x',10)
    def test_fixed_array_and_signed_bitfield_are_type_driven(self):
        d=toy();d['TpiStream']['Records'].extend([
            dict(Kind='LF_ARRAY',Array=dict(ElementType=0x21,Size=6)),
            dict(Kind='LF_BITFIELD',BitField=dict(Type=0x74,BitOffset=4,BitSize=3))])
        dec=Decoder(Types(d),Memory({0x10000:bytes.fromhex('010002000300'),0x20000:struct.pack('<I',0x70)}))
        dec.decode(4099,0x10000,'a');dec.decode(4100,0x20000,'b')
        self.assertEqual([r['value'] for r in dec.rows],[1,2,3,-1])
    def test_union_bytes_do_not_double_count_overlapping_candidates(self):
        self.assertEqual(union_size([(100,112),(104,116),(200,204)]),20)

if __name__=='__main__':unittest.main()
