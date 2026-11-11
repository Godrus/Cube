"""Consistency checks for the GOV-01 package (audit aid, not a CI gate).
Run from the repository root:  python docs/archive/audit/gov01_check.py
Exit code 1 if a check fails. Writes nothing."""
import os
import re
import subprocess
import sys

H = os.path.dirname(os.path.abspath(__file__))
R = os.path.abspath(os.path.join(H, '..', '..'))
sys.path.insert(0, H)
import adg  # noqa: E402

G = os.path.join(R, 'docs', 'gov01')
results = []


def rd(path):
    return open(os.path.join(R, path), encoding='utf8').read()


def check(name, ok, detail=''):
    results.append((name, ok, detail))


# Git is always called with explicit end-of-line settings, so the result does not depend on the user's global core.autocrlf.
GIT_CFG = ['-c', 'core.autocrlf=true', '-c', 'core.safecrlf=false']


def git(*args):
    return subprocess.run(['git', '-C', R] + GIT_CFG + list(args), capture_output=True, text=True).stdout


CR_LF = bytes([13, 10])
LF_ONLY = bytes([10])


def lf(b):
    return b.replace(CR_LF, LF_ONLY)


def crlf(b):
    return lf(b).replace(LF_ONLY, CR_LF)


def patch_reproduction(patch_path, base_rel):
    """Apply the proposed patch to COPIES of the baseline, with explicit line endings and explicit git settings.
    Returns (ok, detail). Nothing in the repository is written. --ignore-whitespace is NOT used."""
    import hashlib
    import tempfile
    base_wt = open(os.path.join(R, base_rel), 'rb').read()
    blob = subprocess.run(['git', '-C', R, 'show', 'HEAD:' + base_rel.replace(os.sep, '/')], capture_output=True).stdout
    if lf(base_wt) != lf(blob):
        return False, 'working-tree baseline differs from the committed blob (after line-ending normalisation)'
    patch = open(patch_path, 'rb').read()
    results = {}
    for name, enc in (('LF', lf), ('CRLF', crlf)):          # base and patch get the SAME explicit ending
        with tempfile.TemporaryDirectory() as d:
            target = os.path.join(d, base_rel)
            os.makedirs(os.path.dirname(target))
            open(target, 'wb').write(enc(base_wt))
            pp = os.path.join(d, 'p.patch')
            open(pp, 'wb').write(enc(patch))
            for cfg in ('false', 'true'):                  # result must not depend on core.autocrlf either
                tgt_copy = open(target, 'rb').read()
                r = subprocess.run(['git', '-c', 'core.autocrlf=' + cfg, '-c', 'core.safecrlf=false', 'apply', '--unidiff-zero', '--check', pp], cwd=d, capture_output=True, text=True)
                if r.returncode != 0 and not (cfg == 'true' and name == 'CRLF'):
                    return False, '%s/%s autocrlf=%s: %s' % (name, name, cfg, r.stderr.strip()[:100])
            r = subprocess.run(['git', '-c', 'core.autocrlf=false', '-c', 'core.safecrlf=false', 'apply', '--unidiff-zero', pp], cwd=d, capture_output=True, text=True)
            if r.returncode != 0:
                return False, '%s apply: %s' % (name, r.stderr.strip()[:100])
            out = lf(open(target, 'rb').read())
            g = adg.load(target)
            ch = adg.run_checks(g)
            results[name] = (hashlib.sha256(out).hexdigest()[:12], all(c.ok for c in ch), g.find_cycle() is None)
    ok = len({v[0] for v in results.values()}) == 1 and all(v[1] and v[2] for v in results.values())
    return ok, str(results)


gov_files = sorted(f for f in os.listdir(G) if f.endswith('.md'))
texts = {f: open(os.path.join(G, f), encoding='utf8').read() for f in gov_files}
reg = texts['GOV-01-DECISION-REGISTER.md']

# 1. decision count: summary table, sections, GOV-01 task row, checklist rows
summary = re.findall(r'^\| (\d+) \| ([A-Za-z0-9/ .\-]+?) \| .*? \| .*? \| (Blocking PS-0|Blocking PS-1|Deferred(?: \(conditional\))?) \| (confirm|\*\*choose\*\*[^|]*) \|$', reg, re.M)
check('register summary table has 15 numbered decisions', [int(x[0]) for x in summary] == list(range(1, 16)), 'found %d' % len(summary))
cls = [x[2].split(' (')[0] for x in summary]
check('classification counts 11 / 2 / 2', (cls.count('Blocking PS-0'), cls.count('Blocking PS-1'), cls.count('Deferred')) == (11, 2, 2), str((cls.count('Blocking PS-0'), cls.count('Blocking PS-1'), cls.count('Deferred'))))
choose = sum(1 for x in summary if x[3].startswith('**choose'))
check('personal choice 9 / confirm-only 6', (choose, 15 - choose) == (9, 6), str((choose, 15 - choose)))
secs = re.findall(r'^### (\d+)\. ', reg, re.M)
check('register has 15 decision sections', [int(x) for x in secs] == list(range(1, 16)), str(secs))
tb = rd('docs/production/agent-task-breakdown.md')
row = [l for l in tb.splitlines() if l.startswith('| GOV-01 |')][0]
tokens = ['ACR-001/002/003', 'ACR-004 part A', 'OAM-10', 'OD-R1', 'OD-R2', 'OD-R6', 'D-R8', 'D-K', 'D-T', 'D-C', 'D-0', 'D-E', 'branch option']
missing = [t for t in tokens if t not in row]
check('GOV-01 task row names all 15 decisions (13 tokens, ACR-001/002/003 counts 3)', not missing, str(missing))
chk = rd('docs/production/ps-0-exit-checklist.md')
rows = re.findall(r'^\| (\d+) \| .*? \| .*? \| .*? \| NOT DONE \|$', chk[chk.index('## 2.'):chk.index('## 3.')], re.M)
check('PS-0 exit checklist section 2 has rows 1-15', [int(x) for x in rows] == list(range(1, 16)), str(len(rows)))

# 2. X-1..X-4 traceability
adg_g = adg.load()
for x in ('X-1', 'X-2', 'X-3', 'X-4'):
    present = [f for f, t in texts.items() if x in t]
    check('%s appears in register, blockers and corrective documents' % x,
          all(f in present for f in ('GOV-01-DECISION-REGISTER.md', 'PS-0-REMAINING-BLOCKERS.md')) and os.path.exists(os.path.join(G, 'GOV-01-CORRECTIVE-REVIEW.md')) and x in texts.get('GOV-01-CORRECTIVE-REVIEW.md', ''), str(present))
check('X-2/X-3/X-4 homes GOV-02, GOV-03, GOV-05 exist in the ADG and are G0 members',
      all(t in adg_g.tasks and t in adg_g.gates['G0'][1] for t in ('GOV-02', 'GOV-03', 'GOV-05')))
check('GOV-03 has no dependencies (X-3 independent)', adg_g.tasks['GOV-03'].start == [] and adg_g.tasks['GOV-03'].merge == [])

# 3. paths in backticks (existing, or declared future)
FUTURE = {'docs/decisions/decision-registry.md', 'docs/production/decision-registry.md', 'docs/archive/audit/gov01_check.py'}
bad = []
for f, t in texts.items():
    for p in set(re.findall(r'`((?:docs|crates)/[A-Za-z0-9_./-]+\.(?:md|rs|py|patch))`', t)):
        if not os.path.exists(os.path.join(R, p)) and p not in FUTURE:
            bad.append((f, p))
check('repository paths named in the GOV-01 documents exist (or are declared future files)', not bad, str(bad))

# 4. status words: nothing in this package presents an owner decision or contract approval as done
lines = []
for f, t in texts.items():
    for i, l in enumerate(t.splitlines(), 1):
        if re.search(r'\b(is|are|now|was|been) Owner-Approved\b|Status:\*\* *Owner-Approved|\bhas been approved\b', l):
            lines.append((f, i, l.strip()[:90]))
check('no package document asserts an approval status as fact', not lines, str(lines))
check('every GOV-01 document is labelled Proposed', all('**Proposed.**' in t.split('\n', 6)[2] + t.split('\n', 6)[3] or '**Proposed.**' in '\n'.join(t.split('\n')[:6]) for t in texts.values()))
changed = git('diff', '--name-only').split()
untracked = git('ls-files', '--others', '--exclude-standard').split()
forbidden = [p for p in changed if p.startswith(('docs/api-contracts/', 'docs/decisions/', 'docs/production/', 'crates/', 'godot-client/', 'scripts/', 'Cargo'))]
check('no contract, decision, production-plan or code file changed in the working tree', not forbidden, str(forbidden))

# 5. authority wording: the Integration Owner is never given approval power outside quoted conflict rows
viol = []
for f, t in texts.items():
    for i, l in enumerate(t.splitlines(), 1):
        if re.search(r'Integration Owner[^.|\n]{0,40}\b(approves|approved|decides|may approve)\b', l, re.I):
            conflict_row = l.startswith('|') and '`' in l.split('|')[1]          # table row naming an older document
            disclaimer = re.search(r'(older|conflict|never|does not|not approve|says|\blines? \d+)', l, re.I) is not None   # a quotation of an older document carries its line citation
            if conflict_row or disclaimer:
                continue
            viol.append((f, i, l.strip()[:100]))
check('Integration Owner is not given approval authority in this package (quotes of older documents excepted)', not viol, str(viol))

# 6. ADG
cur = adg.load()
sc = [c for c in adg.run_checks(cur) if c.kind == 'structure']
check('ADG structure checks pass on the committed ADG', all(c.ok for c in sc), '; '.join(c.name for c in sc if not c.ok))
prop = adg.apply_release_closure(adg.apply_g1_merge_rule(cur))
pc = adg.run_checks(prop)
check('ADG with the proposal: all checks pass, no cycle', all(c.ok for c in pc) and prop.find_cycle() is None)
check('proposal adds no start-after edge', all(prop.tasks[t].start == cur.tasks[t].start for t in cur.tasks))
r1 = adg.render_report(prop)
check('report is deterministic (two renders identical)', r1 == adg.render_report(adg.apply_release_closure(adg.apply_g1_merge_rule(adg.load()))))
patch = os.path.join(G, 'ADG-D01-D02.proposed.patch')
import hashlib
mb_rel = os.path.join('docs', 'production', 'module-boundaries.md')
before = hashlib.sha256(open(os.path.join(R, mb_rel), 'rb').read()).hexdigest()
ok_p, det_p = patch_reproduction(patch, mb_rel)
check('patch applies to copies of the baseline with LF and with CRLF line endings, under core.autocrlf true and false, and the patched graph passes all checks (no --ignore-whitespace)', ok_p, det_p)
check('the repository copy of module-boundaries.md was not modified by the check', hashlib.sha256(open(os.path.join(R, mb_rel), 'rb').read()).hexdigest() == before)
check('module-boundaries.md unchanged since 2c0c1f3', git('diff', '2c0c1f3', 'HEAD', '--', 'docs/production').strip() == '' and 'docs/production/module-boundaries.md' not in changed)

# 7. P1 consumer analysis (CR-10)
proc = [t for t in cur.tasks if t.startswith('PROC') or t == 'TEST-08']
check('no Processing task has GOV-07 or GOV-09 as ancestor (their development does not wait for contract approval)', all('GOV-07' not in cur.ancestors(t) and 'GOV-09' not in cur.ancestors(t) for t in proc))
check('GOV-07 is merge-after PROC-02 and PROC-04 (approval follows implementation)', {'PROC-02', 'PROC-04'} <= set(cur.tasks['GOV-07'].merge))
check('GOV-09 is merge-after API-02/03/04 (approval follows implementation)', {'API-02', 'API-03', 'API-04'} <= set(cur.tasks['GOV-09'].merge))
check('API-02 has GOV-07 and G2 as ancestors (Editing production waits for Processing approval)', 'GOV-07' in cur.ancestors('API-02') and 'G2' in cur.ancestors('API-02'))
ed = rd('docs/api-contracts/processing-contracts.md')
check('processing-contracts.md contains no E-xx reference', re.search(r'\bE-\d\d\b', ed) is None)

fails = [r for r in results if not r[1]]
for n, ok, d in results:
    print('PASS' if ok else 'FAIL', n, ('(' + d + ')') if (d and not ok) else '')
print('%d checks, %d failed' % (len(results), len(fails)))
sys.exit(1 if fails else 0)
