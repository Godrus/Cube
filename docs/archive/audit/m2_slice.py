"""Which ADG tasks does the proposed First Playable (M2) need? (audit aid, not a CI gate; nothing is written)

Run from the repository root:  python docs/archive/audit/m2_slice.py

The selection below is a PROPOSAL (docs/archive/production/FIRST_PLAYABLE_DEPENDENCIES.md). The script computes
  (a) the closure over the ADG exactly as published (gate edges included), and
  (b) the closure over task-to-task edges only (gate edges dropped),
so that the effect of stage gates on a milestone slice is visible.
"""
import collections
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import adg  # noqa: E402

g = adg.load()

SELECTION = {
    'Core (all of gate G1)': list(g.gates['G1'][1]) + ['CORE-16'],
    'Processing': ['PROC-01', 'PROC-02', 'PROC-03', 'PROC-04', 'PROC-05', 'TEST-08'],
    'Editing API and orchestration': ['API-01', 'API-02', 'API-03', 'API-04', 'ORCH-01', 'ORCH-02', 'ORCH-03', 'ORCH-04', 'ORCH-05'],
    'Voxel backend and providers (faces provider; greedy is not needed)': ['VOX-01', 'VOX-02', 'VOX-04', 'VOX-05'],
    'Persistence': ['PERS-01', 'PERS-02', 'PERS-03', 'PERS-04', 'PERS-05', 'PERS-06', 'PERS-08', 'ITG-01'],
    'World generation': ['WGEN-01', 'WGEN-02', 'WGEN-03', 'WGEN-04', 'LEG-01', 'ITG-02'],
    'Client runtime and adapter': ['CLI-00', 'CLI-01', 'CLI-02', 'CLI-04', 'CLI-05', 'ADP-01', 'ADP-02', 'ADP-03', 'LEG-02'],
    'Embedded server and in-process transport': ['SRV-01', 'SRV-03', 'SRV-04', 'NET-01', 'NET-02', 'NET-03'],
}

selected = []
for group, ids in SELECTION.items():
    missing = [t for t in ids if t not in g.tasks]
    if missing:
        raise SystemExit('unknown task ids in selection: %s' % missing)
    selected += ids
selected = sorted(set(selected))

with_gates = set(selected)
for t in selected:
    with_gates |= {x for x in g.ancestors(t) if x in g.tasks}
direct = set(selected)
stack = list(selected)
while stack:
    t = stack.pop()
    for x in g.tasks[t].start + g.tasks[t].merge:
        if x in g.tasks and x not in direct:
            direct.add(x)
            stack.append(x)

print('ADG tasks:', len(g.tasks))
print('proposed M2 selection:', len(selected))
for k, v in SELECTION.items():
    print('  %-70s %d' % (k, len(v)))
print('closure with gate edges (as published):', len(with_gates), '-> pulled in beyond the selection:', len(with_gates - set(selected)))
print('closure over task-to-task edges only    :', len(direct), '-> pulled in beyond the selection:', len(direct - set(selected)))
print('gates required (published ADG):', [x for x in sorted(g.gates, key=lambda y: int(y[1:])) if any(x in g.ancestors(t) for t in selected)])
print('external inputs needed (published ADG):', sorted({x for t in selected for x in g.ancestors(t) if x.startswith('EXT')}))
print('external inputs needed (task edges only):', sorted({x for t in direct for x in g.tasks[t].start + g.tasks[t].merge if x.startswith('EXT')}))
extra = sorted(with_gates - set(selected))
print('pulled in by the published ADG, by stage:', dict(sorted(collections.Counter(g.tasks[x].stage for x in extra).items(), key=lambda kv: adg.stage_num(kv[0]))))
print('  engineering tasks:', [x for x in extra if not x.startswith('GOV')])
print('  owner-decision tasks (GOV):', [x for x in extra if x.startswith('GOV')])
print('not needed by M2 under the published ADG (%d):' % len(set(g.tasks) - with_gates), sorted(set(g.tasks) - with_gates))
print('owners of the selected tasks:', dict(collections.Counter(g.tasks[t].owner for t in selected)))
