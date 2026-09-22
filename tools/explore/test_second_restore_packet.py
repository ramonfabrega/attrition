import copy
import json
from pathlib import Path
import struct
import subprocess
import tempfile
import unittest
from unittest.mock import patch
from second_restore_packet import intervals,project,validate_pair,materialize,FILES,COMMON
from search_census import records
from restore_poststate import decode_post

ROOT=Path(__file__).resolve().parent

class SecondTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.temp=tempfile.TemporaryDirectory();p=Path(cls.temp.name);cls.directory=p
  subprocess.run(['clang','-std=gnu11','-Wall','-Wextra','-Wno-unused-function',
                  '-Wno-int-to-void-pointer-cast','-Wno-pointer-to-int-cast',
                  str(ROOT/'test_live_restore_second.c'),'-o',str(p/'test')],check=True)
  subprocess.run([str(p/'test'),str(p/'packets'),str(p/'trace')],check=True)
  raw=(p/'packets').read_bytes();cls.raw=(raw[:692],raw[692:]);cls.rows=list(records(p/'trace'))
 @classmethod
 def tearDownClass(cls):cls.temp.cleanup()
 def fixture(self,p):
  for index,raw in enumerate(self.raw):
   stem='second-' if index else ''
   for name in FILES:(p/(stem+name)).write_bytes(b'authored fixture')
   (p/(stem+'restore-prefix.bin')).write_bytes(raw[32:248])
   (p/(stem+'restore-poststate.bin')).write_bytes(raw)
  for name in COMMON:(p/name).write_bytes(b'authored fixture')
  (p/'rontrace.log').write_bytes((self.directory/'trace').read_bytes())
 def test_c_callbacks_and_python_pair_binding(self):
  with tempfile.TemporaryDirectory() as temp:
   p=Path(temp);self.fixture(p);a,b=validate_pair(p,self.rows)
   self.assertIsNotNone(a['intervention']);self.assertIsNone(b['intervention'])
   self.assertEqual((a['frame'],b['frame']),(224,225))
 def test_projection_keeps_every_native_call_and_startup_record(self):
  rows=[(5,160,2,0x688f40,0x688fa5,0x682f30,0,0),*self.rows]
  for i in (0,1):
   selected=project(rows,i)
   self.assertEqual([r for r in selected if r[0] in (7,8)],[r for r in rows if r[0] in (7,8)])
   self.assertEqual(selected[0],rows[0])
   self.assertEqual(sum(r[:2]==(7,0) and r[3]==0x160b8 for r in selected),1)
   self.assertEqual(sum(r[:2]==(5,181) for r in selected),1)
   self.assertEqual(sum(r[:2]==(5,183) for r in selected),1-i)
 def test_missing_reordered_and_failure_markers_refuse(self):
  markers=[i for i,r in enumerate(self.rows) if r[:2] in ((5,190),(5,191))]
  for idx in markers:
   with self.assertRaises(ValueError):intervals(self.rows[:idx]+self.rows[idx+1:])
  for index in markers:
   for field in (2,3,7):
    bad=copy.deepcopy(self.rows);r=list(bad[index]);r[field]^=1;bad[index]=tuple(r)
    with self.assertRaises(ValueError):intervals(bad)
  for bad in (self.rows+self.rows,[(5,192,0,1,0,0,0,224)]+self.rows,
              [(5,167,0,0,0,0,0,224)]+self.rows):
   with self.assertRaises(ValueError):intervals(bad)
 def test_packet_contents_must_match_markers(self):
  with tempfile.TemporaryDirectory() as temp:
   p=Path(temp);self.fixture(p)
   (p/'second-restore-prefix.bin').write_bytes(self.raw[0][32:248])
   with self.assertRaises(ValueError):validate_pair(p,self.rows)
   self.fixture(p);r=bytearray(self.raw[0]);r[284+0x104:284+0x118]=bytes(20)
   (p/'restore-poststate.bin').write_bytes(r)
   with self.assertRaises(ValueError):validate_pair(p,self.rows)
 def test_materialization_retains_provenance_and_validates_payload(self):
  with tempfile.TemporaryDirectory() as temp:
   p=Path(temp);source=p/'source';source.mkdir();self.fixture(source)
   with patch('memory_payload.validate',return_value={'sha256':'fixture-payload'}) as validator:
    result=materialize(p/'install',source,p/'second',1)
    validator.assert_called_once_with(p/'install',p/'second')
   self.assertFalse((p/'second'/'.incomplete').exists())
   self.assertEqual((p/'second'/'restore-poststate.bin').read_bytes(),self.raw[1])
   self.assertEqual((source/'restore-poststate.bin').read_bytes(),self.raw[0])
   self.assertEqual(json.loads((p/'second'/'packet-projection.json').read_text()),result)
   self.assertEqual(len(result['source_trace_sha256']),64)
   with self.assertRaises(ValueError):materialize(p/'install',source,p/'second',1)
 def test_changed_post_state_during_copy_refuses(self):
  with tempfile.TemporaryDirectory() as temp:
   p=Path(temp);source=p/'source';source.mkdir();self.fixture(source)
   def mutate(install,destination):
    target=destination/'restore-poststate.bin';raw=bytearray(target.read_bytes());raw[-1]^=1;target.write_bytes(raw)
    return {'sha256':'fixture-payload'}
   with patch('memory_payload.validate',side_effect=mutate):
    with self.assertRaisesRegex(ValueError,'changed during projection'):materialize(p/'install',source,p/'bad',1)
   self.assertTrue((p/'bad'/'.incomplete').exists())
 def test_payload_failure_leaves_explicit_incomplete_projection(self):
  with tempfile.TemporaryDirectory() as temp:
   p=Path(temp);source=p/'source';source.mkdir();self.fixture(source)
   with patch('memory_payload.validate',side_effect=ValueError('payload mismatch')):
    with self.assertRaisesRegex(ValueError,'payload mismatch'):materialize(p/'install',source,p/'bad',0)
   self.assertTrue((p/'bad'/'.incomplete').exists());self.assertFalse((p/'bad'/'packet-projection.json').exists())
if __name__=='__main__':unittest.main()
