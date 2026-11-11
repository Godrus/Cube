"""Independent ADG reading (audit aid, not a CI gate). Run from the repository root: python docs/archive/audit/ia.py
Thin wrapper over adg.py; it prints, it does not enforce (exit code 0 unless the markdown layout is broken)."""
import collections
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import adg  # noqa: E402

g = adg.load()
print('tasks', len(g.tasks), 'gates', len(g.gates))
print('unknown refs', g.unknown_references())
print('stage inversions', g.stage_inversions())
print('gate problems', g.gate_membership_problems(), 'ungated', g.ungated())
a1 = g.ancestors('G1')
print('G1 ancestors outside PS0/1:', sorted(x for x in a1 if x in g.tasks and adg.stage_num(g.tasks[x].stage) > 1), 'EXT in G1:', [x for x in a1 if x.startswith('EXT')])
print('EXT-DEV ancestor of gates:', [x for x in sorted(g.gates) if x != 'G14' and 'EXT-DEV' in g.ancestors(x)])
print('G2 ancestor of every ORCH task:', all('G2' in g.ancestors(t) for t in g.tasks if t.startswith('ORCH')))
print('G2 ancestor of VOX-01,SDF-01,MSH-01,SRV-01,API-02:', ['G2' in g.ancestors(t) for t in ('VOX-01', 'SDF-01', 'MSH-01', 'SRV-01', 'API-02')])
print('PS-1 owners', dict(collections.Counter(t.owner for t in g.tasks.values() if t.stage == 'PS-1')))
print('stages', dict(sorted(collections.Counter(t.stage for t in g.tasks.values()).items(), key=lambda kv: adg.stage_num(kv[0]))))
print('UNASSIGNED owners:', sorted(t.id for t in g.tasks.values() if 'UNASSIGNED' in t.owner))
n, p = g.longest_path('G14')
print('longest path to G14 (task nodes):', n)
order = sorted(g.gates, key=lambda x: int(x[1:]))
for gate in order:
    print(gate, 'predecessor gates:', [x for x in order if x in g.ancestors(gate)])
