"""Before/after comparison of the proposed D-01/D-02 correction, in memory only (see docs/gov01/ADG-D01-D02-PATCH-PROPOSAL.md).
Run from the repository root: python docs/audit/adg-fix-simulation.py"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import adg  # noqa: E402

cur = adg.load()
rule = adg.apply_g1_merge_rule(cur)
full = adg.apply_release_closure(rule)
for label, graph in (('BEFORE (committed ADG)', cur), ('AFTER R-MERGE-G1 only', rule), ('AFTER R-MERGE-G1 + release closure', full)):
    print('==', label)
    print(adg.format_checks(adg.run_checks(graph)))
    print('longest path to G14:', graph.longest_path('G14')[0], 'tasks')
print('== task rows changed by R-MERGE-G1: %d (every engineering task of PS-2+ gets merge-after G1)' % sum(
    1 for t in rule.tasks if rule.tasks[t].merge != cur.tasks[t].merge))
print('== QA-04 merge-after before:', cur.tasks['QA-04'].merge)
print('== QA-04 merge-after after :', full.tasks['QA-04'].merge)
