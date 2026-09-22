import os
from pathlib import Path
import unittest
import tempfile
from caller_collide import CallerCollide,OFFSET,UNIT_BYTES

IMAGE=os.environ.get('RON_CONTEXT_IMAGE')

@unittest.skipUnless(IMAGE and Path(IMAGE).is_file(),'requires owned image')
class CallerTests(unittest.TestCase):
 def test_original_instruction_changes_only_short_and_resets(self):
  step=CallerCollide(IMAGE,0x15000)
  for value in (0,1,2,127,255,32767,32768,65534,65535):
   raw=bytearray((i*13+7)%256 for i in range(UNIT_BYTES))
   raw[OFFSET:OFFSET+2]=value.to_bytes(2,'little');raw=bytes(raw)
   out,report=step.run(raw)
   expected=bytearray(raw);expected[OFFSET:OFFSET+2]=((value+1)&65535).to_bytes(2,'little')
   self.assertEqual(out,bytes(expected));self.assertEqual(report['instructions'],1)
   self.assertEqual(step.run(raw),(out,report))
   self.assertEqual(CallerCollide(IMAGE,0x15000).run(raw),(out,report))
 def test_other_image_refuses(self):
  with tempfile.TemporaryDirectory() as temp:
   other=Path(temp)/'authored-image';other.write_bytes(b'authored negative control')
   with self.assertRaisesRegex(ValueError,'image differs'):CallerCollide(other,0x15000)
 def test_extent_and_address_controls(self):
  for address in (0,0x15001,2**32-10,True):
   with self.assertRaises(ValueError):CallerCollide(IMAGE,address)
  step=CallerCollide(IMAGE,0x15000)
  for data in (b'',bytes(343),bytes(345),bytearray(344)):
   with self.assertRaises(ValueError):step.run(data)

if __name__=='__main__':unittest.main()
