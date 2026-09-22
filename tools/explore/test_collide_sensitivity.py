import copy,struct,unittest
from types import SimpleNamespace
from bounded_call import BoundedCall,Region
from collide_sensitivity import AccessTrace,require_passthrough,VALUES

ENTRY,STOP,DATA,ESP=0x100000,0x110000,0x200000,0x300100
REGS=(0,0,0,ESP,0,0,0,0,2)

def runner(code):
 return BoundedCall([Region('code',ENTRY,code,executable=True),
                     Region('data',DATA,bytes(8),writable=True),
                     Region('return',ESP,struct.pack('<I',STOP))],STOP)

class TraceTests(unittest.TestCase):
 def test_guest_overlap_detects_both_edges_and_host_reads_do_not(self):
  # A four-byte read/write intersects the watched middle two bytes.
  r=runner(b'\xa1'+struct.pack('<I',DATA)+b'\xa3'+struct.pack('<I',DATA)+b'\xc3')
  trace=AccessTrace(r.uc,DATA+1,2)
  try:
   r.run(ENTRY,REGS)
   self.assertEqual(trace.result()['events'],[
    dict(kind='read',address=DATA,size=4),dict(kind='write',address=DATA,size=4)])
   r.uc.mem_read(DATA,4);r.uc.mem_write(DATA,b'1234')
   self.assertEqual(len(trace.result()['events']),2)
   self.assertEqual(trace.result()['instructions'],3)
  finally:trace.close()
  old=trace.result();r.run(ENTRY,REGS);self.assertEqual(trace.result(),old)
 def test_nonoverlapping_guest_access_is_not_a_field_read(self):
  r=runner(b'\xa1'+struct.pack('<I',DATA+4)+b'\xc3')
  trace=AccessTrace(r.uc,DATA,2)
  try:r.run(ENTRY,REGS)
  finally:trace.close()
  self.assertEqual(trace.result()['events'],[])
  self.assertEqual(trace.result()['instructions'],2)

 def test_host_service_reads_are_tracked_and_wrapper_restored(self):
  r=runner(b'\xc3');original=lambda at:7;service=SimpleNamespace(checked_word=original)
  trace=AccessTrace(r.uc,DATA+1,2,service)
  try:
   self.assertEqual(service.checked_word(DATA),7)
   service.checked_word(DATA+4)
   self.assertEqual(trace.result()['host_service_reads'],[DATA])
  finally:trace.close()
  self.assertIs(service.checked_word,original)
 def test_passthrough_assertion_rejects_counterexamples(self):
  rows=[]
  for value in VALUES:
   rows.append(dict(value=value,repeat_matches=True,second=dict(returned=True,output_collide=value,
    guest_trace=dict(events=[],host_service_reads=[],instructions=2,instruction_trace_sha256='authored'),
    unit_except_collide_sha256='same',fingerprint=dict(registers=[1],declared_memory_sha256=str(value)),
    comparison=dict(outer_return_equal=True,return_modes_equal=True,all_path_slots_equal=True,
     active_path_equal=True,native_length=1,replay_length=1,native_capacity=1,replay_capacity=1,
     unit_changes_outside_path_pointer=[136]))))
  require_passthrough(rows)
  mutations=[lambda s:s.update(returned=False),lambda s:s.update(output_collide=99),
    lambda s:s['guest_trace']['events'].append('read'),
    lambda s:s['guest_trace']['host_service_reads'].append(DATA),
    lambda s:s['guest_trace'].update(instruction_trace_sha256='changed'),
    lambda s:s.update(unit_except_collide_sha256='changed'),
    lambda s:s['fingerprint'].update(registers=[2]),
    lambda s:s['comparison'].update(all_path_slots_equal=False),
    lambda s:s['comparison'].update(unit_changes_outside_path_pointer=[138])]
  for mutate in mutations:
   bad=copy.deepcopy(rows);mutate(bad[0]['second'])
   with self.assertRaises(ValueError):require_passthrough(bad)
  with self.assertRaises(ValueError):require_passthrough(rows[:-1])

if __name__=='__main__':unittest.main()
