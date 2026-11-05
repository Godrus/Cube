"""Read-only consistency check of the status documents against the ADG. Run: python docs/audit/val.py  (exit 1 on failure)"""
import re, os, sys
H = os.path.dirname(os.path.abspath(__file__))
exec(open(os.path.join(H, 'ia.py'), encoding='utf8').read().split("print('tasks'")[0])
D = os.path.join(H, '..'); R = os.path.join(H, '..', '..')
files = ['PROJECT_STATUS.md','DEVELOPMENT_ROADMAP.md','RELEASE_CRITERIA.md','AGENT_HANDOFF.md','audit/Independent-Production-Audit.md','audit/PROJECT-STATUS-VALIDATION.md']
pref = r'(?:GOV|CORE|PROC|API|ORCH|VOX|SDF|MSH|SCU|BOO|HIST|COMP|RES|PERS|WGEN|LEG|CLI|ADP|NET|SRV|TEST|XT|QA|ITG|SPK|DEV|RND|INT)'
known = set(T) | set(G) | {'EXT-GPS','EXT-VA-RG1','EXT-DEV'}
FUTURE = {'docs/production/decision-registry.md'}   # intentionally not yet present
fail = 0
for f in files:
    s = open(os.path.join(D, f), encoding='utf8').read()
    bad = sorted(i for i in set(re.findall(r'\b' + pref + r'-(?:\d+[ab]?|P\d)\b', s)) if i not in known)
    miss = sorted(p for p in set(re.findall(r'`((?:docs|crates)/[A-Za-z0-9_./-]+\.(?:md|rs|py))`', s)) if not os.path.exists(os.path.join(R, p)) and p not in FUTURE)
    ph = re.findall(r'\{\{\w+\}\}', s)
    ok = not (bad or miss or ph); fail += not ok
    print('PASS' if ok else 'FAIL', f, bad, miss, ph)
sys.exit(1 if fail else 0)
