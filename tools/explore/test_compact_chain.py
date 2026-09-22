import copy,struct,unittest
from pathlib import Path
from bounded_call import BoundedCall,Region
from compact_chain import encode_regions,decode_regions,validate_parameters,SYNTH,build

ENTRY,STOP,ESP,DATA=0x100000,0x110000,0x300100,0x200000
REGS=(0,0,0,ESP,0,0,0,0,2)
def word(n):return struct.pack('<I',n)

def fixture(code=b'\xa1'+word(DATA)+b'\xc3',scratch=False,writable=True):
 return [Region('code',ENTRY,code,executable=True),Region('return',ESP,word(STOP)),
         Region('input',DATA,bytes(4) if scratch else word(19),writable=writable,scratch=scratch)]

def roundtrip(regions):
 entries,blob=encode_regions(regions)
 return decode_regions(entries,blob,[r for r in regions if r.executable])

class CompactTests(unittest.TestCase):
 def test_initial_inputs_not_live_outputs_roundtrip(self):
  regions=fixture(b'\xff\x05'+word(DATA)+b'\xa1'+word(DATA)+b'\xc3')
  original=BoundedCall(regions,STOP);first=original.run(ENTRY,REGS)
  self.assertEqual(first[0][7],20)
  restored=BoundedCall(roundtrip(original.regions),STOP)
  self.assertEqual(restored.run(ENTRY,REGS),first)
  self.assertEqual(restored.run(ENTRY,REGS),first)
 def test_undefined_scratch_is_not_promoted(self):
  for regions in (fixture(scratch=True),roundtrip(fixture(scratch=True))):
   with self.assertRaisesRegex(ValueError,'uninitialized scratch'):
    BoundedCall(regions,STOP).run(ENTRY,REGS)
 def test_readonly_still_refuses_write(self):
  regions=fixture(b'\xc7\x05'+word(DATA)+word(7)+b'\xc3',writable=False)
  for value in (regions,roundtrip(regions)):
   with self.assertRaisesRegex(ValueError,'read-only write|write-protected'):
    BoundedCall(value,STOP).run(ENTRY,REGS)
 def test_generated_regions_excluded_and_unknown_refuses(self):
  entries,blob=encode_regions(fixture()+[Region(name,0x400000+i*4096,b'x') for i,name in enumerate(sorted(SYNTH))])
  self.assertEqual([r['name'] for r in entries],['code','return','input'])
  with self.assertRaises(ValueError):encode_regions(fixture()+[Region('_unknown',0x400000,b'x')])
  with self.assertRaises(ValueError):encode_regions([Region('scratch',DATA,b'x',writable=True,scratch=True)])
 def test_corruption_missing_duplicate_and_trailing_refuse(self):
  regions=fixture();entries,blob=encode_regions(regions);code=regions[:1]
  for bad in (blob[:-1],blob+b'x',bytes([blob[0]^1])+blob[1:]):
   with self.assertRaises(ValueError):decode_regions(entries,bad,code)
  for bad in (entries[1:],entries+entries[:1]):
   with self.assertRaises(ValueError):decode_regions(bad,blob,code)
  for key,value in (('size',-1),('address',True),('storage','other'),('writable',1),('executable',True),('offset',1)):
   bad=copy.deepcopy(entries);bad[2][key]=value
   with self.subTest(key=key),self.assertRaises(ValueError):decode_regions(bad,blob,code)
  bad=copy.deepcopy(entries);bad[0]['address']+=1
  with self.assertRaises(ValueError):decode_regions(bad,blob,code)
 def test_build_refuses_repository_output_before_reading_inputs(self):
  with self.assertRaisesRegex(ValueError,'outside repository'):
   build(Path('/missing-install'),Path('/missing-capture'),Path(__file__).parent/'forbidden-fixture')
 def test_model_caps_are_not_silent_defaults(self):
  p=dict(malloc_target=0x500000,memset_target=0x501000,memcpy_target=0x502000,free_target=0x503000,
         arena_address=0x600000,arena_size=1024*1024,max_allocations=1024,max_fill_bytes=1024*1024,
         max_fills=1024,max_copy_bytes=1024*1024,max_copies=1024,max_frees=1024,budget=1000000,
         fs_address=0x70000000,gdt_address=0x71000000,borrowed=[[DATA,4]])
  validate_parameters(p)
  for key,value in (('max_frees',2048),('budget',1000001),('budget',True),('borrowed',[[DATA,-1]]),
                    ('borrowed',[[DATA,4]]*65),('extra',1)):
   with self.subTest(key=key),self.assertRaises(ValueError):validate_parameters({**p,key:value})

if __name__=='__main__':unittest.main()
