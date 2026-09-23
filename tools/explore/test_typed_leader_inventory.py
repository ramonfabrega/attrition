import copy,unittest
from typed_leader_inventory import survey

class InventoryTests(unittest.TestCase):
    def fixture(self):
        storage=[dict(path=f'leaders.list[1].econ[{i}]',status='value',value=i) for i in range(2)]
        printed=[dict(key='who',value='1'),dict(key='econ[scan]',value='0'),dict(key='econ[scan]',value='1')]
        for i,row in enumerate(printed):row.update(line=i,logger_path='GAME/FRAME/LEADERDATA',owner_block_line=1)
        s=dict(snapshot=dict(sha256='toy'),roots=dict(leaders=dict(rows=storage)))
        return s,dict(snapshot_sha256='toy',rows=printed)
    def test_equality_is_never_promoted(self):
        s,r=self.fixture();before=copy.deepcopy(r);out=survey(s,r)
        self.assertEqual(out['candidate_equal_occurrences'],2);self.assertEqual(out['validated_coverage_added'],0)
        self.assertEqual(before,r);self.assertFalse(out['logger_parity_established'])
    def test_shape_value_and_storage_ambiguity(self):
        for case in ('shape','value','duplicate','gap','unreadable'):
            with self.subTest(case=case):
                s,r=self.fixture();rows=s['roots']['leaders']['rows']
                if case=='shape':r['rows'].pop()
                if case=='value':rows[0]['value']=3
                if case=='duplicate':rows.append(copy.deepcopy(rows[0]))
                if case=='gap':rows[1]['path']='leaders.list[1].econ[2]'
                if case=='unreadable':rows[0]['status']='unresolved'
                self.assertEqual(survey(s,r)['candidate_equal_occurrences'],0)
    def test_identity_and_snapshot_controls(self):
        s,r=self.fixture();r['rows'].append(copy.deepcopy(r['rows'][0]))
        self.assertTrue(survey(s,r)['errors'])
        s,r=self.fixture();r['snapshot_sha256']='wrong'
        with self.assertRaises(ValueError):survey(s,r)

if __name__=='__main__':unittest.main()
