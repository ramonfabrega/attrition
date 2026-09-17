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

    def test_gaia_path_change_is_reported_outside_comparator_scope(self):
        control = data()
        control['frames'][0]['units'] = [dict(id='8/0', scope=False, original=[1, 2],
            originalRecord='same', originalPath=[], rust=[1, 2], rustPath=[], rustClocks=[])]
        treatment = copy.deepcopy(control)
        treatment['frames'][0]['units'][0]['rustPath'] = [[3, 4]]
        change = compare(control, treatment)['first_changed_record']
        self.assertEqual(change['unit_changes'], [dict(id='8/0', fields={
            'rustPath': dict(control=[], treatment=[[3, 4]])})])
        self.assertEqual(change['control_differences'], change['treatment_differences'])

    def test_clock_payload_rejects_lossy_or_malformed_values(self):
        control = data()
        clock = dict(cur_time=5, end_time=16, last_time=-1, anim=7, gpiece=60063, stopped=False)
        unit = dict(id='8/0', scope=False, original=[1, 2], originalRecord='same',
                    originalPath=[], rust=[1, 2], rustPath=[], rustClocks=[clock])
        control['frames'][0]['units'] = [unit]
        self.assertTrue(compare(control, copy.deepcopy(control))['all_exported_records_equal'])
        for value in (5.0, True, -1, 2**32):
            bad = copy.deepcopy(control)
            bad['frames'][0]['units'][0]['rustClocks'][0]['cur_time'] = value
            with self.assertRaises(ValueError):
                compare(control, bad)

    def test_different_source_record_is_not_an_intervention(self):
        control = data()
        treatment = copy.deepcopy(control)
        treatment['frames'][0]['index'] += 1
        with self.assertRaises(ValueError):
            compare(control, treatment)


if __name__ == '__main__':
    unittest.main()
