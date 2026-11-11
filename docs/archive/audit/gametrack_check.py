"""Consistency checks for the Game Track documents (audit aid, not a CI gate).
Run from the repository root:  python docs/archive/audit/gametrack_check.py
Exit code 1 if a check fails. Writes nothing."""
import os
import re
import subprocess
import sys

H = os.path.dirname(os.path.abspath(__file__))
R = os.path.abspath(os.path.join(H, '..', '..'))
sys.path.insert(0, H)
import adg  # noqa: E402

DOCS = {
    'GAME_TRACK_ROADMAP': 'docs/archive/production/GAME_TRACK_ROADMAP.md',
    'FIRST_PLAYABLE_SCOPE': 'docs/archive/production/FIRST_PLAYABLE_SCOPE.md',
    'FIRST_PLAYABLE_DEPENDENCIES': 'docs/archive/production/FIRST_PLAYABLE_DEPENDENCIES.md',
    'VERTICAL_SLICE_SCOPE': 'docs/archive/production/VERTICAL_SLICE_SCOPE.md',
    'TECHNOLOGY-GAME-TRACK-INTEGRATION': 'docs/archive/production/TECHNOLOGY-GAME-TRACK-INTEGRATION.md',
    'GOV-01-POST-REVIEW-CORRECTIONS': 'docs/archive/gov01/GOV-01-POST-REVIEW-CORRECTIONS.md',
}
results = []


def check(name, ok, detail=''):
    results.append((name, ok, detail))


def rd(rel):
    return open(os.path.join(R, rel), encoding='utf8').read()


GIT_CFG = ['-c', 'core.autocrlf=true', '-c', 'core.safecrlf=false']


def git(*a):
    return subprocess.run(['git', '-C', R] + GIT_CFG + list(a), capture_output=True, text=True).stdout


texts = {}
for k, p in DOCS.items():
    ex = os.path.exists(os.path.join(R, p))
    check('%s exists' % k, ex)
    if ex:
        texts[k] = rd(p)

check('every document is labelled Proposed in its header', all('**Proposed' in '\n'.join(t.split('\n')[:8]) for t in texts.values()))

g = adg.load()
known = set(g.tasks) | set(g.gates) | set(adg.EXTERNAL)
pref = r'(?:GOV|CORE|PROC|API|ORCH|VOX|SDF|MSH|SCU|BOO|HIST|COMP|RES|PERS|WGEN|LEG|CLI|ADP|NET|SRV|TEST|XT|QA|ITG|SPK|DEV|RND|INT)'
bad_ids = {}
for k, t in texts.items():
    ids = set(re.findall(r'\b' + pref + r'-(?:\d+[ab]?|P\d)\b', t))
    b = sorted(i for i in ids if i not in known)
    if b:
        bad_ids[k] = b
check('task, gate and external identifiers named in the documents exist in the ADG', not bad_ids, str(bad_ids))

FUTURE = {'docs/decisions/decision-registry.md'}
bad_paths = []
for k, t in texts.items():
    for p in set(re.findall(r'`((?:docs|crates)/[A-Za-z0-9_./-]+\.(?:md|rs|py|patch))`', t)):
        if not os.path.exists(os.path.join(R, p)) and p not in FUTURE:
            bad_paths.append((k, p))
check('repository paths named in backticks exist', not bad_paths, str(bad_paths))

# label discipline: new proposals are marked
check('documents use the [NP]/[REC]/[EP] labels or state that everything is a proposal',
      all(('[NP]' in t or 'proposal' in t.lower()) for t in texts.values()))

# M2 scope structure
sc = texts.get('FIRST_PLAYABLE_SCOPE', '')
sec2 = sc[sc.index('## 2.'):sc.index('## 3.')] if sc else ''
nums = [int(x) for x in re.findall(r'^\| (\d+) \| ', sec2, re.M)]
check('M2 scope evaluates requirements 1-13', nums == list(range(1, 14)), str(nums))
fps = re.findall(r'^\| (FPS-\d\d) \|', sc, re.M)
check('M2 acceptance criteria FPS-01...FPS-17 are all defined', fps == ['FPS-%02d' % i for i in range(1, 18)], str(fps))
vs = texts.get('VERTICAL_SLICE_SCOPE', '')
vsg = re.findall(r'^\| (VSG-\d\d)', vs, re.M)
check('M3 acceptance criteria VSG-01...VSG-14 are all defined', vsg == ['VSG-%02d' % i for i in range(1, 15)], str(vsg))

# the quoted computation matches the tool
out = subprocess.run([sys.executable, os.path.join(H, 'm2_slice.py')], capture_output=True, text=True).stdout
m = {k: int(v) for k, v in re.findall(r'(proposed M2 selection|closure with gate edges \(as published\)|closure over task-to-task edges only)\s*:?\s*(\d+)', out)}
dep = texts.get('FIRST_PLAYABLE_DEPENDENCIES', '')
check('numbers in the dependency document equal the output of m2_slice.py (65 selected, 100 and 84 closures)',
      m.get('proposed M2 selection') == 65 and m.get('closure with gate edges (as published)') == 100 and m.get('closure over task-to-task edges only') == 84
      and '65 of 147' in dep and '**100 of 147**' in dep and '84 tasks' in dep, str(m))
unassigned = int(re.search(r'owners of the selected tasks: (\{.*\})', out).group(1).count('UNASSIGNED') and sum(v for k, v in eval(re.search(r'owners of the selected tasks: (\{.*\})', out).group(1)).items() if 'UNASSIGNED' in k))
check('20 of the selected tasks have no owner, as stated', unassigned == 20 and '20 of the 65' in dep, str(unassigned))

# no invented performance budgets
pat = re.compile(r'\b\d+(?:\.\d+)?\s?(?:ms|fps|FPS|Hz|MB|GB|KB|kB|dp)\b|frames per second|\bunder \d+ ?(?:ms|s)\b')
hits = [(k, mm.group(0)) for k, t in texts.items() for mm in pat.finditer(t)]
check('no performance or capacity budget numbers appear in the documents', not hits, str(hits[:6]))

# no approval claims
claims = [(k, i) for k, t in texts.items() for i, l in enumerate(t.splitlines(), 1)
          if re.search(r'\b(is|are|was|been) Owner-Approved\b|\bhas been approved\b|\bwe approve\b', l)]
check('no document asserts an approval as fact', not claims, str(claims))

# milestone names agree with RELEASE_CRITERIA
rc = rd('docs/RELEASE_CRITERIA.md')
names = {'M2': 'First Playable', 'M3': 'Vertical Slice', 'M4': 'Internal Alpha', 'M5': 'Closed Alpha', 'M6': 'Beta'}
gt = texts.get('GAME_TRACK_ROADMAP', '')
check('milestone names in the Game Track roadmap match RELEASE_CRITERIA.md', all(('%s %s' % (k, v)) in gt and v in rc for k, v in names.items()))

# repository scope
changed = git('diff', '--name-only').split()
untracked = git('ls-files', '--others', '--exclude-standard').split()
allowed = ('docs/archive/gov01/', 'docs/archive/audit/', 'docs/production/')
tracked_prod = [p for p in changed if p.startswith('docs/production/')]
outside = [p for p in changed + untracked if not p.startswith(allowed) and p != '.gitignore']
check('no tracked file under docs/production/ was modified (new Game Track files only)', not tracked_prod, str(tracked_prod))
check('all changes are inside docs/archive/gov01, docs/archive/audit, docs/production (plus the owner\'s .gitignore)', not outside, str(outside))
check('.gitignore change is still the owner\'s single added line', git('diff', '.gitignore').count('\n+') == 2 and 'AGENTS.md' in git('diff', '.gitignore'))
check('AGENTS.md is untouched (untracked and ignored)', git('ls-files', 'AGENTS.md').strip() == '' and 'AGENTS.md' not in ' '.join(changed + untracked))
check('code, contracts and decision records unchanged',
      not [p for p in changed + untracked if p.startswith(('crates/', 'godot-client/', 'scripts/', 'docs/api-contracts/', 'docs/decisions/', 'Cargo'))])

fails = [r for r in results if not r[1]]
for n, ok, d in results:
    print('PASS' if ok else 'FAIL', n, ('(' + d + ')') if (d and not ok) else '')
print('%d checks, %d failed' % (len(results), len(fails)))
sys.exit(1 if fails else 0)
