"""Prints the stage/gate progress table of PROJECT_STATUS 5.4 from the ADG (does not edit any file).
Run from the repository root: python docs/archive/audit/gen.py"""
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import adg  # noqa: E402

NAMES = {0: 'Governance and Architecture Freeze', 1: 'Geometry Core Foundation (P0-alpha)', 2: 'Processing Foundation', 3: 'Editing API and Contracts',
         4: 'Editing Orchestration', 5: 'Geometry Backends and Derived Providers', 6: 'Tools: Sculpting and Boolean', 7: 'History', 8: 'Composite Geometry',
         9: 'Resampling and Resolution Refinement', 10: 'Persistence', 11: 'World Generation', 12: 'Client Runtime and Engine Adapter',
         13: 'Server Runtime and Networking', 14: 'Full QA and Release Validation'}
g = adg.load()
text = open(adg.DEFAULT_PATH, encoding='utf8').read()
d = text[text.index('| Stage | Start-after'):text.index('## Part E')]
closing = {}
for line in d.splitlines():
    m = re.match(r'\| (PS-\d+) \| (.*?) \| (.*?) \| (.*?) \| (G\d+: .*) \|$', line)
    if m:
        closing[m.group(1)] = m.groups()[1:]
rows = ['| Stage / Gate | Name | Gate tasks | Acceptance (closing condition, ADG Part D) | Start-after | Merge-after |', '|---|---|---|---|---|---|']
for i in range(15):
    ps, gate = 'PS-%d' % i, 'G%d' % i
    mem = g.gates[gate][1]
    s, mg, _, close = closing[ps]
    rows.append('| %s / %s | %s | %d: %s | %s | %s | %s |' % (ps, gate, NAMES[i], len(mem), ', '.join(mem), close.split(': ', 1)[1], s, mg))
print('\n'.join(rows))
