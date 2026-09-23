import copy,unittest
from typed_guy_bridge import apply

class GuyBridgeTests(unittest.TestCase):
    def fixture(self):
        fields=[('who',1),('o',7),('guy_num',3),('x.value',12),('off_x',-2)]
        storage=[dict(path='guy.'+k,status='value',value=v,address=100+i*4) for i,(k,v) in enumerate(fields)]
        storage.append(dict(path='guy.bank',status='float_bits',bits='00000080',width=32,address=120))
        rows=[]
        for scope,block in [('GAME/FRAME/GUY',1),('GAME/FRAME/UNITDATA/GUY',2)]:
            for key,value in [('who','1'),('o','7'),('guy_num','3'),('x','12'),('(int)off_x','-2'),('*((dword*)','&bank) 2147483648')]:
                rows.append(dict(key=key,value=value,status='unmapped',logger_path=scope,owner_block_line=block))
        s=dict(snapshot=dict(sha256='toy'));r=dict(snapshot_sha256='toy',rows=rows,observable_occurrences=len(rows),comparison_scope='toy')
        return s,r,{(1,7,3):dict(slot=99,rows=storage)}
    def test_identity_not_slot_and_negative_zero_bits(self):
        s,r,g=self.fixture();out=apply(r,s,g)
        self.assertEqual(out['matched_occurrences'],12);self.assertEqual(out['guy_bridge']['complete_observed_records'],2)
        self.assertEqual(out['guy_bridge']['distinct_linked_identities'],1)
        self.assertEqual(out['status_counts']['logger_transform_match'],2)
    def test_nonzero_float_word_and_bad_duplicate_copy(self):
        s,r,g=self.fixture();g[(1,7,3)]['rows'][-1]['bits']='0100807f'
        for row in r['rows']:
            if row['key']=='*((dword*)':row['value']='&bank) 2139095041'
        self.assertEqual(apply(r,s,g)['matched_occurrences'],12)
        s,r,g=self.fixture();r['rows'][-1]['value']='&bank) 0'
        self.assertEqual(apply(r,s,g)['status_counts']['integer_mismatch'],1)
        self.assertEqual(r['guy_bridge']['complete_observed_records'],1)
    def test_duplicate_storage_missing_identity_unknown_field_and_scope(self):
        s,r,g=self.fixture();g[(1,7,3)]['rows'].append(copy.deepcopy(g[(1,7,3)]['rows'][0]))
        self.assertEqual(apply(r,s,g)['status_counts']['guy_bridge_unresolved'],2)
        s,r,g=self.fixture();del g[(1,7,3)]
        self.assertEqual(apply(r,s,g)['matched_occurrences'],0);self.assertTrue(r['guy_bridge']['errors'])
        s,r,g=self.fixture();r['rows'][-1]['value']='&unknown) 0'
        self.assertEqual(apply(r,s,g)['guy_bridge']['complete_observed_records'],1)
        s,r,g=self.fixture()
        for row in r['rows']:row['logger_path']='UNSUPPORTED/GUY'
        self.assertTrue(apply(r,s,g)['guy_bridge']['errors'])



class FigureTraversalTests(unittest.TestCase):
    def fixture(self):
        import struct
        from typed_state import Types
        from test_frame_state import Memory
        fields=[dict(Kind='LF_MEMBER',DataMember=dict(Type=0x74,FieldOffset=i*4,Name=name)) for i,name in enumerate(('who','o','guy_num'))]
        records=[dict(Kind='LF_FIELDLIST',FieldList=fields),
                 dict(Kind='LF_CLASS',Class=dict(Name='Guy',UniqueName='guy',Options=[],FieldList=4096,Size=12)),
                 dict(Kind='LF_POINTER',Pointer=dict(ReferentType=4097,Attrs=32778))]
        rows=[dict(path='unit.guys.'+key,status='value',value=1) for key in ('length','size')]
        rows.append(dict(path='unit.guys.list',status='pointer',value=0x10000,pointee=4098))
        unit=dict(logger_active_flag=True,identity='agrees',registry_owner=1,registry_slot=7,rows=rows)
        return dict(unit_registry=dict(units=[unit])),Types({'TpiStream':{'Records':records}}),Memory({0x10000:struct.pack('<I',0x20000),0x20000:struct.pack('<iii',1,7,3)})
    def test_typed_array_and_ownership_not_slot(self):
        from typed_guy_bridge import decode_figures
        s,t,m=self.fixture();figures=decode_figures(s,t,m)
        self.assertIn((1,7,3),figures);self.assertEqual(figures[(1,7,3)]['slot'],0)
    def test_null_missing_wrong_owner_duplicate_and_inactive(self):
        import struct
        from typed_guy_bridge import decode_figures
        for case in ('null','missing','owner','duplicate'):
            s,t,m=self.fixture()
            if case=='null':m.data[0x10000]=bytes(4)
            if case=='missing':del m.data[0x20000]
            if case=='owner':m.data[0x20000]=struct.pack('<iii',2,7,3)
            if case=='duplicate':s['unit_registry']['units']*=2
            with self.subTest(case=case),self.assertRaises(ValueError):decode_figures(s,t,m)
        s,t,m=self.fixture();s['unit_registry']['units'][0]['logger_active_flag']=False;m.data={}
        self.assertEqual(decode_figures(s,t,m),{})

if __name__=='__main__':unittest.main()
