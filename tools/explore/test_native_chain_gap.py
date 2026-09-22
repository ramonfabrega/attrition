import unittest
from native_chain_gap import unit_delta,require_frozen_gap,OFFSET
from restore_poststate import compare,compare_boundary
from test_restore_poststate import packet,replay
from test_restore_prefix import fixture
from restore_poststate import decode_post

class GapTests(unittest.TestCase):
 def test_native_unit_delta_is_narrow_and_nonempty(self):
  a=bytes(344);b=bytearray(a);b[OFFSET]=1
  self.assertEqual(unit_delta(a,bytes(b)),[OFFSET])
  for offset in (0,OFFSET-1,OFFSET+2,343):
   bad=bytearray(b);bad[offset]=1
   with self.assertRaises(ValueError):unit_delta(a,bytes(bad))
  for left,right in ((a,a),(a,a[:-1]),(bytearray(a),bytes(b))):
   with self.assertRaises(ValueError):unit_delta(left,right)
 def test_gap_assertion_rejects_an_unexplained_difference(self):
  result=dict(unit_changes_outside_path_pointer=[OFFSET],outer_return_equal=True,return_modes_equal=True,
              native_length=8,replay_length=8,native_capacity=40,replay_capacity=40,
              active_path_equal=True,all_path_slots_equal=True)
  require_frozen_gap(result)
  for key,bad in (('unit_changes_outside_path_pointer',[]),('unit_changes_outside_path_pointer',[OFFSET,100]),
                  ('outer_return_equal',False),('return_modes_equal',None),('return_modes_equal',False),
                  ('native_length',9),('native_capacity',41),('active_path_equal',False),('all_path_slots_equal',False)):
   with self.subTest(key=key,bad=bad),self.assertRaises(ValueError):require_frozen_gap({**result,key:bad})
 def test_cross_input_output_helper_does_not_relax_strict_binding(self):
  native=decode_post(packet(),fixture(2));r=replay(native)
  self.assertEqual(compare_boundary(native,r['last']),compare(native,r,'bound'))
  with self.assertRaisesRegex(ValueError,'another payload'):compare(native,r,'other')
  r['native_intervention']={'fabricated':True}
  with self.assertRaisesRegex(ValueError,'provenance'):compare(native,r,'bound')

if __name__=='__main__':unittest.main()
