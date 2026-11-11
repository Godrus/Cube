"""Command line wrapper around adg.py (audit aid, not a CI gate).

Run from the repository root:
    python docs/archive/audit/adg_check.py                 # checks on the committed ADG (policy findings are reported)
    python docs/archive/audit/adg_check.py --proposed      # same checks on the ADG with the proposed D-01/D-02 patch applied in memory
    python docs/archive/audit/adg_check.py --table         # markdown table: predecessors now vs required
    python docs/archive/audit/adg_check.py --determinism   # runs the report twice and compares
Exit code: 1 if a *structure* check fails; with --strict also if a *policy* check fails.
Nothing is written to the repository.
"""
import argparse
import hashlib
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import adg  # noqa: E402


def gate_table(cur, new, exclusions):
    order = sorted(cur.gates, key=lambda x: int(x[1:]))
    rows = ['| Task/Gate | Current predecessors (gates) | Required predecessors (gates) | Reason | Proposed change |', '|---|---|---|---|---|']
    for g in order:
        c = [x for x in order if x in cur.ancestors(g)]
        n = [x for x in order if x in new.ancestors(g)]
        if g == 'G14':
            reason = 'release validation must follow every in-scope stage; owner-approved exclusions: %s' % (', '.join(sorted(exclusions)) or 'none')
            change = 'QA-04 merge-after += ' + ', '.join(x for x in order if x not in ('G0', 'G14') and x not in cur.tasks['QA-04'].merge and x not in exclusions)
        elif int(g[1:]) >= 2:
            reason = 'mutation-boundary tests (TEST-06) and INT-01 sit in G1; later merges must not overtake them'
            change = 'rule R-MERGE-G1 (engineering tasks of this stage merge after G1)' if 'G1' not in c else 'none'
        elif g == 'G1':
            reason = 'closes PS-1; becomes a predecessor of every later gate through the proposed rules'
            change = 'none for G1 itself'
        else:
            reason = 'start of the chain'
            change = 'none'
        rows.append('| %s | %s | %s | %s | %s |' % (g, ', '.join(c) or '-', ', '.join(n) or '-', reason, change))
    for t in ('PROC-01', 'CORE-15', 'CORE-16', 'PERS-01', 'API-01', 'QA-04'):
        tk = cur.tasks[t]
        nk = new.tasks[t]
        rows.append('| %s | S: %s; M: %s | S: %s; M: %s | %s | %s |' % (
            t, ', '.join(tk.start) or '-', ', '.join(tk.merge) or '-', ', '.join(nk.start) or '-', ', '.join(nk.merge) or '-',
            'representative task of the rule' if t != 'QA-04' else 'release closure', 'merge-after += G1' if 'G1' in nk.merge and 'G1' not in tk.merge else ('see G14 row' if t == 'QA-04' else 'none')))
    return '\n'.join(rows)


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument('--path', help='module-boundaries.md (default: the repository copy)')
    ap.add_argument('--proposed', action='store_true', help='apply the proposed D-01/D-02 rules in memory')
    ap.add_argument('--exclude', action='append', default=[], help='gate excluded from release closure (needs an owner decision; default none)')
    ap.add_argument('--strict', action='store_true')
    ap.add_argument('--table', action='store_true')
    ap.add_argument('--determinism', action='store_true')
    a = ap.parse_args(argv)
    cur = adg.load(a.path)
    g = adg.apply_release_closure(adg.apply_g1_merge_rule(cur), a.exclude) if a.proposed else cur
    if a.table:
        print(gate_table(cur, adg.apply_release_closure(adg.apply_g1_merge_rule(cur), a.exclude), a.exclude))
        return 0
    rep = adg.render_report(g, a.exclude)
    if a.determinism:
        rep2 = adg.render_report(adg.load(a.path) if not a.proposed else adg.apply_release_closure(adg.apply_g1_merge_rule(adg.load(a.path)), a.exclude), a.exclude)
        same = rep == rep2
        print('deterministic:', same, hashlib.sha256(rep.encode()).hexdigest()[:16])
        return 0 if same else 1
    print(rep)
    checks = adg.run_checks(g, a.exclude)
    bad_struct = [c for c in checks if c.kind == 'structure' and not c.ok]
    bad_policy = [c for c in checks if c.kind == 'policy' and not c.ok]
    return 1 if bad_struct or (a.strict and bad_policy) else 0


if __name__ == '__main__':
    sys.exit(main())
