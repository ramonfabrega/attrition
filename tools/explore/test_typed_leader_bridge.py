import copy,unittest
from typed_leader_bridge import apply,MASKS

class LeaderBridgeTests(unittest.TestCase):
    def fixture(self):
        rows=[dict(key='who',value='1')];decoded=[]
        for field,mask in MASKS.items():
            for i in range(7 if field=='resource_cap' else 6):
                value=i-2
                rows.append(dict(key=field,value=str(value)))
                decoded.append(dict(path=f'encrypted.{field}[{i}]',type_index=0x74,
                                    status='value',value=(value&0xffffffff)^mask,address=100+i*4))
        rows.append(dict(key='unknown',value='42'))
        for r in rows:r.update(logger_path='GAME/FRAME/LEADERDATA',owner_block_line=1,status='unmapped')
        report=dict(rows=rows,snapshot_sha256='toy',observable_occurrences=len(rows),comparison_scope='toy')
        state=dict(snapshot=dict(sha256='toy'),roots=dict(leaders=dict(rows=[dict(path='leaders.list[1].data_encrypted',status='pointer',value=100,pointee=1)])))
        return report,state,decoded
    def test_arrays_signed_projection_and_denominator(self):
        r,s,d=self.fixture();out=apply(r,s,lambda p:d)
        self.assertEqual(out['matched_occurrences'],31)
        self.assertEqual(out['status_counts']['unmapped'],2)
        self.assertFalse(out['logger_parity_established']);self.assertFalse(out['leader_bridge']['referents_anchored'])
    def test_wrong_word_is_a_mismatch(self):
        r,s,d=self.fixture();d[0]['value']^=1
        self.assertEqual(apply(r,s,lambda p:d)['status_counts']['integer_mismatch'],1)
    def test_missing_pointer(self):
        r,s,d=self.fixture();s['roots']['leaders']['rows'][0]['value']=0
        self.assertEqual(apply(r,s,lambda p:d)['status_counts']['leader_bridge_unresolved'],31)
    def test_array_gaps_duplicates_and_short_prints_refuse(self):
        for case in ('gap','duplicate','short','unreadable'):
            with self.subTest(case=case):
                r,s,d=self.fixture()
                if case=='gap':del d[1]
                if case=='duplicate':d.append(copy.deepcopy(d[0]))
                if case=='short':del r['rows'][1];r['observable_occurrences']-=1
                if case=='unreadable':d[0]['status']='unresolved'
                out=apply(r,s,lambda p:d)
                self.assertEqual(out['matched_occurrences'],25)
                self.assertTrue(out['leader_bridge']['errors'])
    def test_duplicate_identity_and_absent_scope(self):
        r,s,d=self.fixture();extra=copy.deepcopy(r['rows'])
        for row in extra:row['owner_block_line']=2
        r['rows']+=extra;r['observable_occurrences']*=2
        self.assertEqual(apply(r,s,lambda p:d)['matched_occurrences'],0)
        r,s,d=self.fixture()
        for row in r['rows']:row['logger_path']='OTHER'
        self.assertEqual(apply(r,s,lambda p:d)['matched_occurrences'],0)
        self.assertTrue(r['leader_bridge']['errors'])

if __name__=='__main__':unittest.main()
