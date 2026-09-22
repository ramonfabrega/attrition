import copy,struct,unittest
from bounded_call import Region
from continuation_cleanup import first_gap,patch_words,require_findings

class CleanupTests(unittest.TestCase):
 def test_first_gap_distinguishes_equality_branch_and_ended_prefix(self):
  self.assertIsNone(first_gap([[1,2]],[[1,2]]))
  self.assertEqual(first_gap([[1,2],[3,4]],[[1,2],[5,4]])['index'],1)
  self.assertEqual(first_gap([[1,2]],[[1,2],[3,4]])['index'],1)
  self.assertEqual(first_gap([],[[1,2]])['left'],[])
 def test_word_patches_merge_without_changing_other_bytes(self):
  data=bytes(range(16));r=Region('data',0x200000,data,writable=True)
  patch=patch_words([r],[(0x200000,9),(0x200008,7)])['data']
  self.assertEqual(patch,struct.pack('<I',9)+data[4:8]+struct.pack('<I',7)+data[12:])
  self.assertEqual(r.data,data)
 def test_word_patch_refuses_ambiguous_undefined_readonly_and_duplicate(self):
  r=Region('data',0x200000,bytes(16),writable=True)
  for regions,changes in [([r],[(0x200000,0),(0x200000,1)]),([r],[(0x200001,0)]),
       ([r],[(0x200010,0)]),([r,r],[(0x200000,0)]),([r],[(0x200000,-1)]),
       ([Region('ro',r.address,r.data)],[(r.address,0)]),
       ([Region('scratch',r.address,r.data,writable=True,scratch=True)],[(r.address,0)])]:
   with self.assertRaises(ValueError):patch_words(regions,changes)
 def test_measured_claim_rejects_counterexamples(self):
  comparison=dict(return_modes_equal=True,outer_return_equal=True,native_length=8,replay_length=8,
   native_capacity=40,replay_capacity=40,unit_changes_outside_path_pointer=[],active_path_equal=True,all_path_slots_equal=True)
  trials=[dict(patched_words=i,result=dict(instructions=n,comparison=copy.deepcopy(comparison)),
               first_gap_from_chain=dict(index=gap),repeat_matches=True)
          for i,(n,gap) in enumerate(zip([34845,33675,33339,33416,33572],[445,480,5796,6220,3719]))]
  trials.append(dict(patched_words=4,result=dict(instructions=33494,comparison=copy.deepcopy(comparison)),first_gap_from_chain=None,repeat_matches=True))
  require_findings(dict(instructions=33494),trials)
  for mutate in [lambda t:t['result'].update(instructions=1),lambda t:t['first_gap_from_chain'].update(index=1),
     lambda t:t.update(repeat_matches=False),lambda t:t['result']['comparison'].update(all_path_slots_equal=False)]:
   bad=copy.deepcopy(trials);mutate(bad[0])
   with self.assertRaises(ValueError):require_findings(dict(instructions=33494),bad)
  with self.assertRaises(ValueError):require_findings(dict(instructions=33494),trials[:-1])

if __name__=='__main__':unittest.main()
