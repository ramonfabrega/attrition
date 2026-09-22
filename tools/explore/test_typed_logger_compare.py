import copy,unittest
from typed_state import Types
from typed_logger_compare import compare

class CompareTests(unittest.TestCase):
    def fixture(self):
        types=Types({'TpiStream':{'Records':[dict(Kind='LF_CLASS',Class=dict(Name='SubObjectData',UniqueName='toy',Options=[],FieldList=0,Size=24))]}})
        lines=list(enumerate(['  BEGIN UNITDATA','   BEGIN OBJECT','    BEGIN SUBOBJECT',
                             '     who 1','     o 2','     x_internal 3480','     unknown 42'],1))
        rows=[dict(path='unit::<base:1000>.'+name,type_index=0x74,address=0x10000+i*4,status='value',value=value)
              for i,(name,value) in enumerate([('who',1),('o',2),('x_internal.value',3480^0x63637)])]
        state=dict(snapshot=dict(frame=24,sha256='authored',logger_return_roots_unchanged=True),
                   unit_registry=dict(units=[dict(registry_owner=1,registry_slot=2,logger_active_flag=True,identity='agrees',rows=rows)]))
        return lines,state,types
    def test_coordinate_transform_and_unknown_key_are_explicit(self):
        lines,state,t=self.fixture();r=compare(lines,24,state,t)
        self.assertEqual(r['status_counts'],dict(exact_integer_match=2,logger_transform_match=1,unmapped=1))
        self.assertEqual(r['observable_occurrences'],4);self.assertEqual(r['linked_unitdata_records'],1)
        self.assertFalse(r['logger_parity_established'])
    def test_wrong_memory_and_duplicate_storage_do_not_pass(self):
        lines,state,t=self.fixture();state['unit_registry']['units'][0]['rows'][2]['value']^=1
        self.assertEqual(compare(lines,24,state,t)['status_counts']['integer_mismatch'],1)
        state['unit_registry']['units'][0]['rows'].append(copy.deepcopy(state['unit_registry']['units'][0]['rows'][0]))
        self.assertEqual(compare(lines,24,state,t)['status_counts']['ambiguous_storage_match'],1)
    def test_wrong_frame_missing_identity_and_empty_scope_refuse_or_remain_unlinked(self):
        lines,state,t=self.fixture()
        with self.assertRaises(ValueError):compare(lines,25,state,t)
        with self.assertRaises(ValueError):compare([(1,' x 3')],24,state,t)
        state['unit_registry']['units']=[];r=compare(lines,24,state,t)
        self.assertEqual(r['matched_occurrences'],0);self.assertEqual(len(r['unlinked_unitdata_records']),1)

    def test_flattened_world_container_names_are_not_scalar_agreement(self):
        lines,state,t=self.fixture()
        lines += list(enumerate(['  BEGIN WORLD','   size 3600','   size 96','   forest_size 3'],100))
        state['roots']=dict(world=dict(rows=[dict(path='world.size',status='value',value=3600,address=1),
                                            dict(path='world.forest_size',status='value',value=3,address=2)]))
        r=compare(lines,24,state,t)
        self.assertEqual(r['status_counts']['ambiguous_logger_ownership'],2)
        self.assertNotIn('integer_mismatch',r['status_counts'])
        state['roots']['world']['rows'][1]['value']=4
        self.assertEqual(compare(lines,24,state,t)['status_counts']['integer_mismatch'],1)

if __name__=='__main__':unittest.main()
