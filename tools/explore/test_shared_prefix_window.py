import copy,json,struct,tempfile,unittest
from pathlib import Path
from shared_prefix_window import extract_headers,require_unchanged_window,packet_view
from shared_search import SIZES
from second_restore_packet import FILES,COMMON

class Pages:
 def __init__(self):
  pf=bytearray(136);struct.pack_into('<6I',pf,0,0x30000,0x30100,0x30200,0x30300,0x30400,0x90000)
  self.data={0xe85e80:bytes(pf),0x20000:bytes(344),0xc8d9b0:bytes(16),0xc8d820:bytes(16)}
  for address,size in zip(range(0x30000,0x30500,0x100),SIZES[2:7]):self.data[address]=bytes(size)
 def read(self,address,size):return self.data.get(address)

class PrefixWindowTests(unittest.TestCase):
 def test_selected_unit_is_not_previous_pathfinder_owner(self):
  blocks=extract_headers(Pages(),0x20000)
  self.assertEqual(blocks[1]['address'],0x20000)
  self.assertEqual(struct.unpack_from('<I',bytes.fromhex(blocks[0]['bytes']),20)[0],0x90000)
  require_unchanged_window(blocks,copy.deepcopy(blocks))
 def test_every_unit_header_and_pool_byte_is_compared(self):
  blocks=extract_headers(Pages(),0x20000)
  for index in range(1,9):
   for offset in range(len(bytes.fromhex(blocks[index]['bytes']))):
    changed=copy.deepcopy(blocks);data=bytearray.fromhex(changed[index]['bytes']);data[offset]^=1;changed[index]['bytes']=data.hex()
    with self.subTest(index=index,offset=offset),self.assertRaises(ValueError):require_unchanged_window(blocks,changed)
  changed=copy.deepcopy(blocks);changed[2]['address']+=4
  with self.assertRaises(ValueError):require_unchanged_window(blocks,changed)
 def test_missing_short_and_setup_differences_are_explicit(self):
  pages=Pages();pages.data.pop(0x30000)
  with self.assertRaises(ValueError):extract_headers(pages,0x20000)
  pages=Pages();pages.data[0x20000]=bytes(343)
  with self.assertRaises(ValueError):extract_headers(pages,0x20000)
  blocks=extract_headers(Pages(),0x20000);changed=copy.deepcopy(blocks)
  data=bytearray.fromhex(changed[0]['bytes']);data[20]^=1;changed[0]['bytes']=data.hex()
  self.assertEqual(require_unchanged_window(blocks,changed)[0]['changed_byte_offsets'],[20])
 def test_packet_view_projects_receipts_without_copying_or_mutating_payload(self):
  rows=[(5,190,0,0x20000,0,16,0,224),(5,191,0,0x20000,0xffffffff,0,0,224),
        (5,190,1,0x20000,0,16,0,225),(5,191,1,0x20000,8,0,0,225)]
  with tempfile.TemporaryDirectory() as temp:
   source=Path(temp)
   for stem in ('','second-'):
    for name in FILES:(source/(stem+name)).write_bytes((stem+name).encode())
   for name in COMMON:
    if name!='capsule-image.json':(source/name).write_bytes(name.encode())
   (source/'receipt.json').write_text(json.dumps({'sha256':dict(zip(('riseofnations.exe','rontrace.dll','riseofnations_trace.exe'),('source','tracer','traced')))}))
   (source/'rontrace.log').write_bytes(bytes(32)+b''.join(struct.pack('<8I',*r) for r in rows))
   original={p.name:p.read_bytes() for p in source.iterdir()}
   with packet_view(source,rows,1) as view:
    self.assertTrue((view/'memory-payload.bin').is_symlink())
    self.assertEqual((view/'memory-payload.bin').read_bytes(),b'second-memory-payload.bin')
    self.assertEqual(json.loads((view/'capsule-image.json').read_text())['sha256']['source'],'source')
   self.assertFalse(view.exists());self.assertEqual(original,{p.name:p.read_bytes() for p in source.iterdir()})
   with self.assertRaises(ValueError):
    with packet_view(source,rows[:-1],1):pass

if __name__=='__main__':unittest.main()
