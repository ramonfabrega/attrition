import copy,struct,subprocess,tempfile,unittest
from pathlib import Path
from shared_search import parse,summarize,field_values,block_deltas,require_boundary_gap
ROOT=Path(__file__).resolve().parent

class SharedTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  with tempfile.TemporaryDirectory() as tmp:
   exe=Path(tmp)/'test';out=Path(tmp)/'rows'
   subprocess.run(['clang','-std=gnu11','-Wall','-Wextra','-Wno-int-to-pointer-cast',str(ROOT/'test_live_shared_search.c'),'-o',str(exe)],check=True)
   subprocess.run([str(exe),str(out)],check=True)
   cls.rows=list(struct.iter_unpack('<8I',out.read_bytes()))
 def test_actual_c_producer_and_failure_controls(self):
  calls=parse(self.rows);self.assertEqual(len(calls),1)
  self.assertEqual(field_values(calls[0]['entry']),dict(valid_root=0x40000,block_root=0x40000,open_node_pool_length=2,open_ref_pool_length=2))
 def test_missing_duplicate_reordered_and_failed_records_refuse(self):
  for index in range(len(self.rows)):
   with self.subTest(index=index),self.assertRaises(ValueError):parse(self.rows[:index]+self.rows[index+1:])
  for tag in (204,205):
   with self.assertRaises(ValueError):parse(self.rows+[(5,tag,0,0,0,0,0,224)])
  for index in (1,2,3):
   with self.assertRaises(ValueError):parse(self.rows[:index]+[self.rows[index]]+self.rows[index:])
 def test_identity_order_tail_and_completion_refuse(self):
  for tag,slot,value in ((200,3,2),(200,4,0),(201,4,0),(201,5,4),(202,4,8),(203,5,0)):
   bad=list(self.rows);i=next(i for i,r in enumerate(bad) if r[:2]==(5,tag));row=list(bad[i]);row[slot]=value;bad[i]=tuple(row)
   with self.subTest(tag=tag,slot=slot),self.assertRaises(ValueError):parse(bad)
 def test_summary_keeps_call_order_separate_from_writer_identity(self):
  a=parse(self.rows)[0];a['return']=0xffffffff
  b=copy.deepcopy(a);b.update(sequence=2,path=a['path']+344,return_value=0);b['return']=0
  c=copy.deepcopy(a);c.update(sequence=3,frame=225);c['return']=1
  result=summarize([a,b,c]);self.assertTrue(result['intervening_exit_matches_next_entry'])
  self.assertFalse(result['native_writer_instruction_identified'])
  with self.assertRaises(ValueError):summarize([a,c,b])

 def test_gap_claim_rejects_single_call_attribution(self):
  a=parse(self.rows)[0];a['return']=0xffffffff
  b=copy.deepcopy(a);b.update(sequence=2,path=a['path']+344);b['return']=0
  c=copy.deepcopy(a);c.update(sequence=3,frame=225);c['return']=1
  calls=[a,b,c]
  for call,ident in zip(calls,(16,22,16)):
   for phase in ('entry','exit'):
    data=bytearray.fromhex(call[phase][1]['bytes']);data[9]=0;struct.pack_into('<H',data,10,ident);call[phase][1]['bytes']=data.hex()
  for blocks,values in zip((a['exit'],b['entry'],b['exit'],c['entry']),((0,0,0,1),(10,20,0,1),(11,20,1,0),(12,21,1,2))):
   for (index,offset),v in zip(((6,12),(5,12),(7,8),(8,8)),values):
    data=bytearray.fromhex(blocks[index]['bytes']);struct.pack_into('<I',data,offset,v);blocks[index]['bytes']=data.hex()
  report=summarize(calls);self.assertEqual(len(require_boundary_gap(report)),3)
  bad=copy.deepcopy(report);bad['calls'][2]['entry']=copy.deepcopy(bad['calls'][1]['exit'])
  with self.assertRaises(ValueError):require_boundary_gap(bad)
  bad=copy.deepcopy(report);bad['calls'][1]['entry'][6]['address']+=4
  with self.assertRaises(ValueError):require_boundary_gap(bad)
 def test_deltas_do_not_treat_different_objects_as_same_extent(self):
  a=parse(self.rows)[0]['entry'];b=copy.deepcopy(a);b[1]['address']+=344
  result=block_deltas(a,b)
  self.assertFalse(result[1]['same_extent']);self.assertIsNone(result[1]['changed_byte_offsets'])

if __name__=='__main__':unittest.main()
