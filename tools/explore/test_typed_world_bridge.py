import copy,struct,unittest
from typed_state import Types
from typed_world_bridge import ArrayBytes,apply,decode_array,grid_plan

class WorldBridgeTests(unittest.TestCase):
    def fixture(self):
        s=dict(snapshot=dict(sha256='toy'))
        plan={'seen[scan]':[dict(path=f'world.seen[{i}]',address=100+i,value=i) for i in range(2)]}
        rows=[]
        for block in (1,2,3):
            for i in range(2):rows.append(dict(key='seen[scan]',value=str(i),owner_block_line=block,logger_path='GAME/FRAME/WORLD',status='unmapped'))
        return s,dict(rows=rows,snapshot_sha256='toy',observable_occurrences=6,comparison_scope='toy'),plan
    def test_every_copy_compared_and_corruption_visible(self):
        s,r,p=self.fixture();out=apply(r,s,p);self.assertEqual(out['matched_occurrences'],6)
        self.assertEqual(out['world_grid_bridge']['distinct_grid_values'],2)
        s,r,p=self.fixture();r['rows'][3]['value']='99';out=apply(r,s,p)
        self.assertEqual(out['matched_occurrences'],5);self.assertEqual(out['status_counts']['integer_mismatch'],1)
        self.assertEqual(out['world_grid_bridge']['blocks'][1]['status_counts']['integer_mismatch'],1)
    def test_cardinality_missing_scope_and_identity_fail(self):
        s,r,p=self.fixture();r['rows'].pop();r['observable_occurrences']-=1
        self.assertEqual(apply(r,s,p)['status_counts']['world_grid_unresolved'],1)
        s,r,p=self.fixture()
        for row in r['rows']:row['logger_path']='OTHER'
        self.assertTrue(apply(r,s,p)['world_grid_bridge']['errors'])
        s,r,p=self.fixture();r['snapshot_sha256']='wrong'
        with self.assertRaises(ValueError):apply(r,s,p)
    def test_pdb_signedness_and_retained_extent(self):
        memory=ArrayBytes(100,struct.pack('<ii',-7,123));types=Types({'TpiStream':{'Records':[]}})
        pointer=dict(status='pointer',value=100,pointee=0x74)
        self.assertEqual([r['value'] for r in decode_array(types,memory,pointer,2,'grid')],[-7,123])
        with self.assertRaises(ValueError):decode_array(types,memory,pointer,3,'grid')
        pointer['value']=0
        with self.assertRaises(ValueError):decode_array(types,memory,pointer,2,'grid')
    def test_dimension_mismatch_refuses_before_pointer_reads(self):
        rows=[dict(path='world.'+key,status='value',value=value) for key,value in [('size',4),('xs',2),('ys',3)]]
        with self.assertRaisesRegex(ValueError,'dimensions disagree'):
            grid_plan(dict(roots=dict(world=dict(rows=rows))),None,None)


class CellBridgeTests(unittest.TestCase):
    def fixture(self):
        from typed_world_bridge import CELL_FIELDS
        cells=[];rows=[]
        for i in range(2):
            cells.append({key:dict(path=f'world.wdata[{i}].{key}',address=100+i*100+n,status='value',value=n+i) for n,key in enumerate(CELL_FIELDS)})
            rows.append(dict(key='LAND_LABEL',value='',status='unclassified_text'))
            for key,c in cells[-1].items():rows.append(dict(key=key,value=str(c['value']),status='unmapped'))
        rows.append(dict(key='flags',value='123',status='unmapped'))
        for r in rows:r.update(logger_path='GAME/FRAME/WORLD',owner_block_line=1)
        return cells,dict(rows=rows,observable_occurrences=len(rows),comparison_scope='toy')
    def test_record_boundaries_leave_flattened_flags_and_land_alone(self):
        from typed_world_bridge import apply_cells
        cells,r=self.fixture();out=apply_cells(r,cells)
        self.assertEqual(out['matched_occurrences'],30);self.assertEqual(out['rows'][-1]['status'],'unmapped')
        self.assertEqual(out['status_counts']['unclassified_text'],2)
    def test_bad_record_shape_never_partially_promotes_block(self):
        from typed_world_bridge import apply_cells
        for key in ('flags','goods','was_seen','who'):
            cells,r=self.fixture();next(x for x in r['rows'] if x['key']==key)['key']='wrong'
            self.assertEqual(apply_cells(r,cells)['matched_occurrences'],0)
            self.assertTrue(r['world_cell_bridge']['errors'])
    def test_wrong_field_is_visible(self):
        from typed_world_bridge import apply_cells
        cells,r=self.fixture();cells[1]['who']['value']=1000
        self.assertEqual(apply_cells(r,cells)['status_counts']['integer_mismatch'],1)

if __name__=='__main__':unittest.main()
