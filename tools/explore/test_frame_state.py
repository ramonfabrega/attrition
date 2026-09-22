import struct,unittest
from typed_state import Types
from frame_state import unit_registry

class Memory:
    def __init__(self,data):self.data=data
    def read(self,address,size):
        for base,raw in self.data.items():
            if base<=address and address+size<=base+len(raw):return raw[address-base:address-base+size]
        raise ValueError('retained bytes unavailable')

class RegistryTests(unittest.TestCase):
    def fixture(self):
        fields=[dict(Kind='LF_MEMBER',DataMember=dict(Type=t,FieldOffset=o,Name=n)) for t,o,n in
                ((0x20,8,'flags'),(0x20,9,'who'),(0x11,10,'o'))]
        records=[dict(Kind='LF_FIELDLIST',FieldList=fields),
                 dict(Kind='LF_CLASS',Class=dict(Name='Unit',UniqueName='unit',Options=[],FieldList=4096,Size=12)),
                 dict(Kind='LF_POINTER',Pointer=dict(ReferentType=4097,Attrs=32778)),
                 dict(Kind='LF_FIELDLIST',FieldList=[dict(Kind='LF_BCLASS',BaseClass=dict(Type=4097,Offset=0))]),
                 dict(Kind='LF_CLASS',Class=dict(Name='Animal',UniqueName='animal',Options=[],FieldList=4099,Size=16)),
                 dict(Kind='LF_CLASS',Class=dict(Name='Unrelated',UniqueName='unrelated',Options=[],FieldList=0,Size=4))]
        rows=[dict(path='units.lists[0].'+k,status='value',value=1) for k in ('length','size')]
        rows.append(dict(path='units.lists[0].list',status='pointer',value=0x10000,pointee=4098))
        leaders=[dict(path='leaders.list[0].leader_flags',status='value',value=1)]
        memory=Memory({0x10000:struct.pack('<I',0x20000),0x20000:struct.pack('<IIBBhI',0x400000,0,1,0,0,0)})
        joined={0x400000:dict(offset=0,type_index=4100,name='Animal')}
        return Types({'TpiStream':{'Records':records}}),memory,rows,leaders,joined
    def test_dynamic_subtype_and_identity(self):
        r=unit_registry(*self.fixture())
        self.assertEqual((r['active_flag_slots'],r['unresolved_slots'],r['active_identity_mismatches']),(1,0,0))
        self.assertEqual(r['units'][0]['dynamic_type'],'Animal')
        self.assertIsNone(r['live_allocation_count'])
    def test_inactive_registry_is_not_dereferenced(self):
        t,m,r,l,j=self.fixture();l[0]['value']=0;m.data={}
        out=unit_registry(t,m,r,l,j)
        self.assertEqual(out['inactive_leader_registries'],[0]);self.assertEqual(out['slots'],0)
    def test_unrelated_rtti_and_unknown_vptr_are_unresolved(self):
        t,m,r,l,j=self.fixture();j[0x400000]['type_index']=4101
        self.assertEqual(unit_registry(t,m,r,l,j)['unresolved_slots'],1)
        self.assertEqual(unit_registry(t,m,r,l,{})['unresolved_slots'],1)
    def test_wrong_registry_identity_remains_visible(self):
        t,m,r,l,j=self.fixture();data=bytearray(m.data[0x20000]);data[9]=1;m.data[0x20000]=bytes(data)
        self.assertEqual(unit_registry(t,m,r,l,j)['active_identity_mismatches'],1)

if __name__=='__main__':unittest.main()
