"""Offline boundaries for opt-in API experiments; never retrieves a credential."""
from concurrent.futures import ThreadPoolExecutor
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
import urllib.error

import jev_evidence_lab as evidence
import jev_search as search


class BudgetTests(unittest.TestCase):
    def test_concurrent_reservations_cannot_exceed_ceiling(self):
        with tempfile.TemporaryDirectory() as tmp:
            path=Path(tmp)/'budget.json'
            path.write_text(json.dumps(dict(prior_usd=.996,ceiling_usd=1,calls={})))
            budget=evidence.Budget(path)
            def reserve(i):
                try:
                    budget.transact(str(i))
                    return True
                except RuntimeError:
                    return False
            with ThreadPoolExecutor(max_workers=8) as pool:
                results=list(pool.map(reserve,range(8)))
            self.assertEqual(sum(results),1)
            self.assertEqual(len(json.loads(path.read_text())['calls']),1)

    def test_transport_failure_retains_reservation_and_records_error(self):
        with tempfile.TemporaryDirectory() as tmp:
            directory=Path(tmp)
            budget=evidence.Budget(directory/'budget.json')
            spec=evidence.packets()[0]
            with patch.object(evidence.urllib.request,'urlopen',side_effect=urllib.error.URLError('offline test')):
                row=evidence.call(spec,'synthetic-test-key',budget,directory)
            self.assertEqual(row['error'],'transport failure')
            ledger=json.loads(budget.path.read_text())
            self.assertEqual(next(iter(ledger['calls'].values()))['charged_or_reserved_usd'],evidence.RESERVE)
            self.assertNotIn('synthetic-test-key',(directory/(spec['id']+'.json')).read_text())

    def test_invalid_accounting_cannot_release_reservation(self):
        with tempfile.TemporaryDirectory() as tmp:
            budget=evidence.Budget(Path(tmp)/'budget.json')
            budget.transact('attempt')
            for tokens in [-1,True,65537]:
                with self.assertRaises(ValueError):
                    budget.transact('attempt',tokens)
            self.assertEqual(json.loads(budget.path.read_text())['calls']['attempt']['status'],'reserved')
            budget.transact('attempt',100)
            self.assertAlmostEqual(json.loads(budget.path.read_text())['calls']['attempt']['charged_or_reserved_usd'],100*evidence.PRICE)


class SearchTests(unittest.TestCase):
    def test_windows_preserve_complete_source_and_line_coordinates(self):
        text=''.join(f'line {i}\n' for i in range(100))+'a very long final line'*30
        lines=text.splitlines(keepends=True)
        covered=set()
        for window in search.windows('example',text,limit=80,overlap=3):
            first,last=window['start_line']-1,window['end_line']
            self.assertEqual(window['text'],''.join(lines[first:last]))
            covered.update(range(first,last))
        self.assertEqual(covered,set(range(len(lines))))

    def test_cached_search_needs_neither_key_nor_network(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp);cache=root/'cache';cache.mkdir()
            request=dict(model=evidence.MODEL,state={'passage':dict(path='example.rs',start_line=1,end_line=1,text='example\n')},questions={'q':dict(type='noul',instructions='example?')})
            spec=dict(id='authored',suite='search',request=request)
            row=dict(spec,response=dict(model=evidence.MODEL,answers={'q':dict(type='noul',noul=.9)}))
            (cache/'authored.json').write_text(json.dumps(row))
            with patch.object(search,'get_key',side_effect=AssertionError('must not fetch key')),patch.object(search,'call',side_effect=AssertionError('must not call API')):
                search.run_search([spec],{'q':'example?'},root/'out',cache,evidence.Budget(root/'budget.json'))
            results=json.loads((root/'out/results.json').read_text())
            self.assertEqual(results['cached'],1)
            self.assertFalse((root/'budget.json').exists())
            spec['request']['state']['passage']['text']='changed\n'
            with self.assertRaises(ValueError):
                search.run_search([spec],{'q':'example?'},root/'bad',cache,evidence.Budget(root/'budget.json'))


if __name__=='__main__':
    unittest.main()
