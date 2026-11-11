"""Importable reader and checker for the Authoritative Dependency Graph (ADG).

Source of truth read here: the published tables of docs/production/module-boundaries.md
(Part C task table, Part D gate table). Nothing in the repository is modified.

Status: audit aid. It is NOT wired into CI; exit codes of the command line wrapper
(adg_check.py) prove nothing about a CI gate.

Public interface
    load(path=None) -> Graph
    Graph.tasks / .gates / .external / .edges
    Graph.ancestors(node), Graph.find_cycle(), Graph.unknown_references(), ...
    apply_g1_merge_rule(graph) -> Graph      (proposal R-MERGE-G1, returns a copy)
    apply_release_closure(graph, exclusions) -> Graph  (proposal, returns a copy)
    run_checks(graph, ...) -> list[Check]
"""
from __future__ import annotations

import collections
import copy
import dataclasses
import os
import re
from typing import Dict, Iterable, List, Optional, Set, Tuple

HERE = os.path.dirname(os.path.abspath(__file__))
DEFAULT_PATH = os.path.join(HERE, '..', 'production', 'module-boundaries.md')
EXTERNAL = ('EXT-GPS', 'EXT-VA-RG1', 'EXT-DEV')
# Rule R-MERGE-G1 exempts decision tasks, spikes and device acquisition.
G1_EXEMPT_PREFIXES = ('GOV', 'SPK', 'DEV')


class AdgFormatError(Exception):
    """The markdown no longer has the expected table layout."""


def stage_num(stage: str) -> int:
    return int(stage.split('-')[1])


@dataclasses.dataclass
class Task:
    id: str
    stage: str
    owner: str
    start: List[str]   # S edges: start-after
    merge: List[str]   # I edges: merge-after


@dataclasses.dataclass
class Graph:
    tasks: Dict[str, Task]
    gates: Dict[str, Tuple[str, List[str]]]   # gate -> (stage, member task ids)
    external: Tuple[str, ...] = EXTERNAL

    # ---- structure -------------------------------------------------
    @property
    def nodes(self) -> Set[str]:
        return set(self.tasks) | set(self.gates) | set(self.external)

    @property
    def edges(self) -> Dict[str, Set[str]]:
        e: Dict[str, Set[str]] = collections.defaultdict(set)
        for t in self.tasks.values():
            e[t.id].update(t.start)
            e[t.id].update(t.merge)
        for g, (_, members) in self.gates.items():
            e[g].update(members)
        for n in self.nodes:
            e.setdefault(n, set())
        return e

    def ancestors(self, node: str) -> Set[str]:
        e = self.edges
        seen: Set[str] = set()
        stack = list(e[node])
        while stack:
            x = stack.pop()
            if x in seen:
                continue
            seen.add(x)
            stack.extend(e.get(x, ()))
        return seen

    def find_cycle(self) -> Optional[List[str]]:
        e = self.edges
        state: Dict[str, int] = {}
        for root in sorted(e):
            if root in state:
                continue
            stack = [(root, iter(sorted(e[root])))]
            path = [root]
            state[root] = 1
            while stack:
                node, it = stack[-1]
                nxt = next(it, None)
                if nxt is None:
                    state[node] = 2
                    stack.pop()
                    path.pop()
                    continue
                if state.get(nxt) == 1:
                    return path[path.index(nxt):] + [nxt]
                if nxt not in state:
                    state[nxt] = 1
                    stack.append((nxt, iter(sorted(e[nxt]))))
                    path.append(nxt)
        return None

    def unknown_references(self) -> List[Tuple[str, str]]:
        nodes = self.nodes
        out = []
        for t in self.tasks.values():
            out += [(t.id, x) for x in t.start + t.merge if x not in nodes]
        for g, (_, members) in self.gates.items():
            out += [(g, x) for x in members if x not in self.tasks]
        return sorted(out)

    def stage_inversions(self) -> List[Tuple[str, str]]:
        return sorted((t.id, x) for t in self.tasks.values() for x in t.start + t.merge
                      if x in self.tasks and stage_num(self.tasks[x].stage) > stage_num(t.stage))

    def gate_membership_problems(self) -> List[str]:
        out, seen = [], collections.Counter()
        for g, (stage, members) in self.gates.items():
            for m in members:
                seen[m] += 1
                if m in self.tasks and self.tasks[m].stage != stage:
                    out.append('%s: member %s belongs to %s, gate stage is %s' % (g, m, self.tasks[m].stage, stage))
        out += ['%s is in %d gates' % (t, c) for t, c in sorted(seen.items()) if c > 1]
        return out

    def ungated(self) -> List[str]:
        gated = {m for _, (_, ms) in self.gates.items() for m in ms}
        return sorted(t for t in self.tasks if t not in gated)

    def engineering_tasks_after_ps1(self) -> List[str]:
        return sorted(t.id for t in self.tasks.values()
                      if stage_num(t.stage) >= 2 and not t.id.startswith(G1_EXEMPT_PREFIXES))

    def longest_path(self, target: str = 'G14') -> Tuple[int, List[str]]:
        e = self.edges
        memo: Dict[str, Tuple[int, Tuple[str, ...]]] = {}

        def f(n: str):
            if n in memo:
                return memo[n]
            best = (0, ())
            for x in sorted(e[n]):
                c = f(x)
                if c[0] > best[0]:
                    best = c
            memo[n] = (best[0] + (1 if n in self.tasks else 0), best[1] + (n,))
            return memo[n]

        n, p = f(target)
        return n, list(p)


# ---- loading ---------------------------------------------------------

def _section(text: str, start: str, end: str) -> str:
    try:
        i = text.index(start)
        j = text.index(end, i)
    except ValueError:
        raise AdgFormatError('section markers not found: %r .. %r' % (start, end))
    return text[i:j]


def _split(cell: str) -> List[str]:
    cell = cell.strip()
    return [] if cell == '-' else [x.strip() for x in cell.split(',')]


def load(path: Optional[str] = None) -> Graph:
    text = open(path or DEFAULT_PATH, encoding='utf8').read()
    part_c = _section(text, '## Part C', '### C.1')
    tasks: Dict[str, Task] = {}
    for line in part_c.splitlines():
        if not line.startswith('| ') or line.startswith(('| Task', '|---')):
            continue
        p = [x.strip() for x in line.strip().strip('|').split('|')]
        if len(p) != 5:
            raise AdgFormatError('Part C row does not have 5 columns: %r' % line[:80])
        if p[0] in tasks:
            raise AdgFormatError('duplicate task id %s' % p[0])
        tasks[p[0]] = Task(p[0], p[1], p[2], _split(p[3]), _split(p[4]))
    part_d = _section(text, '| Gate | Stage', 'Generated from the task edges')
    gates: Dict[str, Tuple[str, List[str]]] = {}
    for line in part_d.splitlines():
        m = re.match(r'\| (G\d+) \| (PS-\d+) \| (.*) \|\s*$', line)
        if m:
            gates[m.group(1)] = (m.group(2), _split(m.group(3)))
    if not tasks or not gates:
        raise AdgFormatError('no tasks or gates parsed')
    return Graph(tasks, gates)


# ---- proposals (return copies; never touch the source) ----------------

def apply_g1_merge_rule(g: Graph) -> Graph:
    """R-MERGE-G1: every engineering task of PS-2..PS-14 merges after G1."""
    h = copy.deepcopy(g)
    for tid in h.engineering_tasks_after_ps1():
        t = h.tasks[tid]
        if 'G1' not in t.start + t.merge:
            t.merge.append('G1')
    return h


def apply_release_closure(g: Graph, exclusions: Iterable[str] = ()) -> Graph:
    """QA-04 (the release validation task) merges after every gate G1..G13 that is not excluded."""
    h = copy.deepcopy(g)
    excl = set(exclusions)
    qa = h.tasks['QA-04']
    already = h.ancestors('G14')
    for gate in sorted(h.gates, key=lambda x: int(x[1:])):
        if gate in ('G0', 'G14') or gate in excl or gate in already:
            continue
        qa.merge.append(gate)
    return h


# ---- checks ----------------------------------------------------------

@dataclasses.dataclass
class Check:
    name: str
    ok: bool
    detail: str = ''
    kind: str = 'structure'   # 'structure' must pass; 'policy' = finding against a proposed rule


def run_checks(g: Graph, exclusions: Iterable[str] = ()) -> List[Check]:
    excl = set(exclusions)
    c: List[Check] = []
    cyc = g.find_cycle()
    c.append(Check('no cycles', cyc is None, ' -> '.join(cyc) if cyc else ''))
    unk = g.unknown_references()
    c.append(Check('all referenced tasks/gates/external nodes exist', not unk, str(unk[:5])))
    inv = g.stage_inversions()
    c.append(Check('no task depends on a later stage', not inv, str(inv[:5])))
    mp = g.gate_membership_problems()
    c.append(Check('gate membership consistent (stage match, one gate per task)', not mp, '; '.join(mp[:3])))
    anc14 = g.ancestors('G14')
    missing = [x for x in sorted(g.gates, key=lambda y: int(y[1:])) if x != 'G14' and x not in anc14 and x not in excl]
    c.append(Check('G14 release closure: every gate is a predecessor of G14 or an owner-approved exclusion',
                   not missing, 'missing: ' + ', '.join(missing), 'policy'))
    gates_after = [x for x in g.gates if int(x[1:]) >= 2]
    no_g1 = [x for x in gates_after if 'G1' not in g.ancestors(x)]
    c.append(Check('G1 is a predecessor of G2..G14', not no_g1, 'missing for: ' + ', '.join(sorted(no_g1, key=lambda y: int(y[1:]))), 'policy'))
    tasks_no_g1 = [t for t in g.engineering_tasks_after_ps1() if 'G1' not in g.ancestors(t) and 'G1' not in g.tasks[t].start + g.tasks[t].merge]
    c.append(Check('G1 merge policy: every engineering task of PS-2+ has G1 as a predecessor',
                   not tasks_no_g1, '%d tasks without G1 ancestor, e.g. %s' % (len(tasks_no_g1), ', '.join(tasks_no_g1[:6])), 'policy'))
    c.append(Check('explicit exclusions refer to existing gates', all(x in g.gates for x in excl), str(sorted(excl))))
    return c


def format_checks(checks: List[Check]) -> str:
    return '\n'.join('%s [%s] %s %s' % ('PASS' if k.ok else 'FAIL', k.kind, k.name, ('(' + k.detail + ')') if (k.detail and not k.ok) else '') for k in checks)


def render_report(g: Graph, exclusions: Iterable[str] = ()) -> str:
    """Deterministic text report (no timestamps, sorted output); used for the reproducibility check."""
    lines = ['tasks %d gates %d' % (len(g.tasks), len(g.gates)), format_checks(run_checks(g, exclusions))]
    n, p = g.longest_path('G14')
    lines.append('longest path to G14: %d tasks' % n)
    for gate in sorted(g.gates, key=lambda x: int(x[1:])):
        lines.append('%s predecessor gates: %s' % (gate, ' '.join(x for x in sorted(g.gates, key=lambda y: int(y[1:])) if x in g.ancestors(gate))))
    return '\n'.join(lines)
