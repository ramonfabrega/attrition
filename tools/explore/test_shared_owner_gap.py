import copy,struct,unittest
from shared_owner_gap import read_owner,require_gap

class Pages:
    def __init__(self):
        pf=bytearray(136);struct.pack_into('<I',pf,20,0x20000)
        unit=bytearray(344);unit[9]=2;struct.pack_into('<H',unit,10,59)
        self.data={0xe85e80:bytes(pf),0x20000:bytes(unit),
                   0xc0aeb4+2*28:struct.pack('<4I',60,64,8,0x30000),
                   0x30000+59*4:struct.pack('<I',0x20000)}
    def read(self,address,size):return self.data.get(address)

class OwnerGapTests(unittest.TestCase):
    def test_identity_requires_live_registry_membership(self):
        p=Pages();r=read_owner(p)
        self.assertEqual((r['owner'],r['id'],r['address']),(2,59,0x20000))
        p.data[0x30000+59*4]=struct.pack('<I',0x21000)
        with self.assertRaisesRegex(ValueError,'registry slot'):read_owner(p)
    def test_missing_or_truncated_record_refuses(self):
        original=Pages()
        for address,data in original.data.items():
            for replacement in (None,data[:-1]):
                p=copy.deepcopy(original);p.data[address]=replacement
                with self.subTest(address=address),self.assertRaises(ValueError):read_owner(p)
    def test_identity_and_registry_bounds_refuse(self):
        for offset,value in ((9,8),(10,64)):
            p=Pages();unit=bytearray(p.data[0x20000]);unit[offset]=value;p.data[0x20000]=bytes(unit)
            with self.assertRaises(ValueError):read_owner(p)
        p=Pages();p.data[0xc0aeb4+56]=struct.pack('<4I',60,59,8,0x30000)
        with self.assertRaises(ValueError):read_owner(p)
    def test_census_counterexample_refuses_without_attributing_headers(self):
        calls=[dict(path=x+0xb8) for x in (0x21000,0x22000,0x21000)]
        rows=[dict(address=0x23000),dict(address=0x20000)]
        result=require_gap(calls,rows)
        self.assertFalse(result['shared_header_writes_attributed'])
        self.assertFalse(result['writer_instruction_identified'])
        for address in (0x21000,0x22000):
            rows[1]['address']=address
            with self.assertRaises(ValueError):require_gap(calls,rows)

if __name__=='__main__':unittest.main()
