import struct,unittest
from bounded_call import Region
from modeled_lifetime import LifetimeCall
from continued_call import continue_once,ENTRY
ESP,STOP,OBJ,ARENA=0x300100,0x400000,0x200000,0x600000
MALLOC,FILL,COPY,FREE=0x500000,0x501000,0x502000,0x503000
REGS=(1,2,3,ESP,4,5,6,7,0x202)
def w(n):return struct.pack('<I',n)
def call(target,offset):return b'\xe8'+struct.pack('<i',target-(ENTRY+0x100+offset+5))
ALLOC=b'\x6a\x04'+call(MALLOC,2)+b'\x83\xc4\x04\xa3'+w(OBJ)+b'\xc7\x00'+w(7)
BOUNDARY=dict(callee_arguments=w(1),modes=w(300)+w(1))
def runner(second,first=ALLOC+b'\xc3',max_allocations=64):
 code=second+b'\x90'*(0x100-len(second))+first
 r=LifetimeCall([Region('code',ENTRY,code,executable=True),Region('object_pointer',OBJ,w(0),writable=True),Region('return',ESP,w(STOP)),Region('callee_arguments',ESP+4,w(1),writable=True),Region('scratch',ESP-256,bytes(256),writable=True,scratch=True),Region('modes',0x220000,w(300)+w(1),writable=True),Region('fs',0x70000000,w(0xffffffff),writable=True)],STOP,malloc_target=MALLOC,memset_target=FILL,memcpy_target=COPY,free_target=FREE,arena_address=ARENA,arena_size=128,max_allocations=max_allocations,fs_address=0x70000000,gdt_address=0x71000000,budget=128)
 r.run(ENTRY+0x100,REGS);return r
class Tests(unittest.TestCase):
 def test_heap_value_and_ownership_survive(self):
  r=runner(b'\xa1'+w(OBJ)+b'\x8b\x00\x40\xc3');before=(r.cursor,r.allocations.copy());result=continue_once(r,REGS,BOUNDARY)
  self.assertTrue(result['returned']);self.assertEqual(result['eax'],8);self.assertEqual((r.cursor,r.allocations),before)
 def test_losing_ownership_or_definedness_refuses(self):
  for lost in ('allocations','initialized'):
   r=runner(b'\xa1'+w(OBJ)+b'\x8b\x00\xc3');getattr(r,lost).clear();result=continue_once(r,REGS,BOUNDARY);self.assertFalse(result['returned'])
   self.assertIn('outside modeled allocation' if lost=='allocations' else 'uninitialized scratch',result['reason'])
 def test_retired_object_remains_retired(self):
  first=ALLOC+b'\x68'+w(ARENA);first+=call(FREE,len(first))+b'\x83\xc4\x04\xc3'
  r=runner(b'\xa1'+w(ARENA)+b'\xc3',first);result=continue_once(r,REGS,BOUNDARY)
  self.assertFalse(result['returned']);self.assertIn('retired object',result['reason'])
 def test_erasing_retirement_is_a_detected_bad_model_control(self):
  first=ALLOC+b'\x68'+w(ARENA);first+=call(FREE,len(first))+b'\x83\xc4\x04\xc3'
  r=runner(b'\xa1'+w(ARENA)+b'\xc3',first);r.retired.clear()
  result=continue_once(r,REGS,BOUNDARY)
  self.assertTrue(result['returned']);self.assertEqual(result['eax'],7)
 def test_stack_does_not_inherit_previous_definedness(self):
  r=runner(b'\x8b\x44\x24\xfc\xc3');result=continue_once(r,REGS,BOUNDARY)
  self.assertFalse(result['returned']);self.assertIn('uninitialized scratch',result['reason'])
 def test_boundary_validation_precedes_writes(self):
  r=runner(b'\xc3');before=bytes(r.uc.mem_read(0x220000,8))
  with self.assertRaises(ValueError):continue_once(r,REGS,dict(BOUNDARY,extra=b'x'))
  self.assertEqual(bytes(r.uc.mem_read(0x220000,8)),before)
 def test_invalid_registers_refuse_before_boundary_writes(self):
  for bad in (-1,1<<32,True,'1'):
   r=runner(b'\xc3');before=bytes(r.uc.mem_read(0x220000,8))
   with self.assertRaises(ValueError):continue_once(r,(*REGS[:-1],bad),dict(BOUNDARY,modes=w(95)+w(1)))
   self.assertEqual(bytes(r.uc.mem_read(0x220000,8)),before)
 def test_allocation_budget_is_cumulative(self):
  second=b'\x6a\x04\xe8'+struct.pack('<i',MALLOC-(ENTRY+7))+b'\x83\xc4\x04\xc3'
  r=runner(second,max_allocations=1);result=continue_once(r,REGS,BOUNDARY)
  self.assertFalse(result['returned']);self.assertIn('allocation',result['reason'])
 def test_unreturned_call_refuses(self):
  from unicorn import x86_const as x
  r=runner(b'\xc3');r.uc.reg_write(x.UC_X86_REG_EIP,ENTRY)
  with self.assertRaisesRegex(ValueError,'prior call did not return'):continue_once(r,REGS,BOUNDARY)
if __name__=='__main__':unittest.main()
