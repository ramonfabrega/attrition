import struct,unittest
from typed_height_precision import measure,value

class PrecisionTests(unittest.TestCase):
    def test_adjacent_singles_can_have_the_same_decimal_print(self):
        raw=struct.pack('<3I',0x3f800000,0x3f800001,0x41800000)
        r=measure(raw)
        self.assertEqual(r['corners'],3);self.assertEqual(r['ambiguous_corners'],2)
        self.assertEqual(r['different_indices'],[1]);self.assertFalse(r['initial_exact_bits_established'])
    def test_negative_values_and_ordinary_unique_prints(self):
        r=measure(struct.pack('<2f',-1.0,1024.0))
        self.assertEqual(r['ambiguous_corners'],1);self.assertEqual(r['nearest_decimal_reconstruction_differs'],0)
    def test_missing_malformed_and_nonfinite_refuse(self):
        for raw in (b'',b'x',struct.pack('<I',0x7fc00001),struct.pack('<I',0x7f800000)):
            with self.assertRaises(ValueError):measure(raw)

if __name__=='__main__':unittest.main()
