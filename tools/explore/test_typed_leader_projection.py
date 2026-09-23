import copy,unittest
from typed_leader_projection import apply,RESOURCE_ARRAYS

class ProjectionTests(unittest.TestCase):
    def fixture(self):
        storage=[];printed=[dict(key='who',value='1')]
        for name in (*RESOURCE_ARRAYS,'gather_slots_high','bonus_cap','chat_status'):
            count=3 if name=='bonus_cap' else 2
            for i in range(count):
                storage.append(dict(path=f'leaders.list[1].{name}[{i}]',status='value',value=i+10,address=100+i*4))
                if name=='chat_status':continue
                key=name+'[scan]' if name!='bonus_cap' or i<2 else 'bonus_cap[NUM_COMMON]'
                for _ in range(2 if name=='gather_slots_high' else 1):printed.append(dict(key=key,value=str(i+10)))
        storage.append(dict(path='leaders.list[1].got_diplo_message',status='value',value=7,address=200))
        printed += [dict(key='got_diplo_message',value='7') for _ in range(2)]
        printed.append(dict(key='unknown',value='0'))
        for i,r in enumerate(printed):r.update(line=i,logger_path='GAME/FRAME/LEADERDATA',owner_block_line=1,status='unmapped')
        s=dict(snapshot=dict(sha256='toy'),roots=dict(leaders=dict(rows=storage)))
        r=dict(snapshot_sha256='toy',rows=printed,observable_occurrences=len(printed),comparison_scope='toy')
        return s,r
    def test_projection_preserves_repeated_and_split_key_occurrences(self):
        s,r=self.fixture();out=apply(r,s)
        self.assertEqual(out['matched_occurrences'],23);self.assertFalse(out['leader_projection']['errors'])
        self.assertEqual(out['status_counts']['unmapped'],2)
        hi=[x for x in out['rows'] if x['key']=='gather_slots_high[scan]']
        self.assertEqual([x['array_index'] for x in hi],[0,0,1,1])
        self.assertFalse(out['logger_parity_established'])
    def test_corrupted_duplicate_and_final_cap_fail_independently(self):
        for key in ('gather_slots_high[scan]','bonus_cap[NUM_COMMON]','got_diplo_message'):
            s,r=self.fixture();next(x for x in r['rows'] if x['key']==key)['value']='999'
            self.assertEqual(apply(r,s)['status_counts']['integer_mismatch'],1)
    def test_missing_or_extra_occurrences_refuse_family(self):
        for case in ('missing','extra','wrong_alias','gap','duplicate_storage'):
            with self.subTest(case=case):
                s,r=self.fixture()
                if case=='missing':r['rows'].pop(1);r['observable_occurrences']-=1
                if case=='extra':r['rows'].append(copy.deepcopy(r['rows'][1]));r['observable_occurrences']+=1
                if case=='wrong_alias':next(x for x in r['rows'] if x['key']=='bonus_cap[NUM_COMMON]')['key']='bonus_cap[scan]'
                if case=='gap':s['roots']['leaders']['rows'][1]['path']='leaders.list[1].econ[3]'
                if case=='duplicate_storage':s['roots']['leaders']['rows'].append(copy.deepcopy(s['roots']['leaders']['rows'][0]))
                out=apply(r,s);self.assertEqual(out['matched_occurrences'],2);self.assertTrue(out['leader_projection']['errors'])
    def test_identity_absence_and_existing_classification(self):
        s,r=self.fixture();r['rows'][0]['key']='other'
        self.assertEqual(apply(r,s)['matched_occurrences'],0)
        s,r=self.fixture();r['rows'][1]['status']='integer_mismatch'
        with self.assertRaises(ValueError):apply(r,s)
        s,r=self.fixture()
        for row in r['rows']:row['logger_path']='OTHER'
        self.assertTrue(apply(r,s)['leader_projection']['errors'])

if __name__=='__main__':unittest.main()
