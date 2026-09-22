import struct,unittest
from bounded_call import BoundedCall,Region
from recycler_reuse import audit_pathnode_reuse as audit,POOL
ENTRY,ESP,STOP,ARRAY,NODE=0x100000,0x300000,0x400000,0x200000,0x210000
REGS=(0,0,0,ESP,0,0,0,0,0x202)
def w(n):return struct.pack('<I',n)
def store(a,v):return b'\xc7\x05'+w(a)+w(v)
POP=store(POOL+8,0);PUSH=store(POOL+8,1)
READ=b'\xa1'+w(NODE);WRITE=store(NODE,9)
def run(code,slots=(NODE,),length=1):
 r=BoundedCall([Region('code',ENTRY,code+b'\xc3',executable=True),Region('return',ESP,w(STOP)),Region('pool',POOL,w(ARRAY)+w(len(slots))+w(length)+w(1),writable=True),Region('array',ARRAY,b''.join(w(p) for p in slots),writable=True),Region('node',NODE,bytes(36),writable=True)],STOP)
 r.run(ENTRY+len(code),REGS)
 def invoke():return dict(run=r.run(ENTRY,REGS),returned=True)
 return audit(r,invoke)
class Tests(unittest.TestCase):
 def test_old_read(self):
  x=run(POP+READ);self.assertEqual(x['reuse_epochs']['epochs'][0]['read_before_write_offsets'],list(range(4)))
 def test_write_first(self):
  x=run(POP+WRITE+READ);self.assertEqual(x['reuse_epochs']['epochs'][0]['read_before_write_offsets'],[])
 def test_reacquisition_invalidates_prior_writes(self):
  x=run(POP+WRITE+READ+PUSH+POP+READ)['reuse_epochs']['epochs']
  self.assertEqual(len(x),2);self.assertTrue(x[0]['released']);self.assertEqual(x[0]['read_before_write_offsets'],[])
  self.assertEqual(x[1]['read_before_write_offsets'],list(range(4)))
 def test_partial_length_store_refuses(self):
  with self.assertRaisesRegex(ValueError,'partial recycler length'):run(b'\xc6\x05'+w(POOL+8)+b'\x00')
 def test_invalid_initial_identity_refuses(self):
  for slots,length in (((NODE,NODE),2),((NODE,NODE+4),2),((0,),1)):
   with self.assertRaises(ValueError):run(b'',slots,length)
 def test_duplicate_release_refuses(self):
  with self.assertRaisesRegex(ValueError,'duplicate pooled release'):run(store(POOL+8,2),(NODE,NODE),1)
 def test_no_acquisition_makes_no_initialization_claim(self):
  x=run(READ)['reuse_epochs'];self.assertEqual(x['epochs'],[]);self.assertTrue(x['successful_invocation'])
 def test_failed_invocation_not_success(self):
  # A guard refusal is allowed to carry diagnostics, never a success claim.
  from unittest.mock import patch
  with patch('recycler_reuse.observe_bytes',side_effect=lambda r,a,n: w(ARRAY)+w(0)+w(0)+w(1)):
   class UC:
    def hook_add(self,*args):return 1
    def hook_del(self,h):pass
   class Runner:uc=UC()
   x=audit(Runner(),lambda:dict(returned=False))['reuse_epochs']
   self.assertFalse(x['successful_invocation'])
if __name__=='__main__':unittest.main()
