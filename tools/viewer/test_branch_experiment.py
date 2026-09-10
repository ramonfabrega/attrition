import copy
import unittest
from branch_experiment import compare
from test_lab_demo import data


class BranchTests(unittest.TestCase):
    def test_equal_and_changed_fields_are_distinguished(self):
        control = data()
        self.assertTrue(compare(control, copy.deepcopy(control))['all_exported_records_equal'])
        treatment = copy.deepcopy(control)
        treatment['frames'][0]['differences'][0]['rust'] = '9'
        changed = compare(control, treatment)
        self.assertFalse(changed['all_exported_records_equal'])
        self.assertEqual(changed['first_changed_record']['frame'], 1900)
        self.assertEqual(changed['first_changed_record']['treatment_differences'][0]['rust'], '9')

    def test_different_source_record_is_not_an_intervention(self):
        control = data()
        treatment = copy.deepcopy(control)
        treatment['frames'][0]['index'] += 1
        with self.assertRaises(ValueError):
            compare(control, treatment)


if __name__ == '__main__':
    unittest.main()
