import re,sys,os,collections
P=os.path.join(os.path.dirname(os.path.abspath(__file__)),'..','production')+os.sep
mb=open(P+'module-boundaries.md',encoding='utf8').read()
c=mb[mb.index('## Part C'):mb.index('### C.1')]
rows=[l for l in c.splitlines() if l.startswith('| ') and not l.startswith('| Task') and not l.startswith('|---')]
T={}
for l in rows:
    p=[x.strip() for x in l.strip('|').split('|')]
    sp=lambda s:[] if s=='-' else [x.strip() for x in s.split(',')]
    T[p[0]]=dict(stage=p[1],owner=p[2],S=sp(p[3]),I=sp(p[4]))
d=mb[mb.index('| Gate | Stage'):mb.index('Generated from the task edges')]
G={}
for l in d.splitlines():
    m=re.match(r'\| (G\d+) \| (PS-\d+) \| (.*) \|',l)
    if m: G[m.group(1)]=(m.group(2),[x.strip() for x in m.group(3).split(',')])
print('tasks',len(T),'gates',len(G))
nodes=set(T)|set(G)|{'EXT-GPS','EXT-VA-RG1','EXT-DEV'}
E=collections.defaultdict(set)
bad=[]
for t,v in T.items():
    for x in v['S']+v['I']:
        if x not in nodes: bad.append((t,x))
        E[t].add(x)
for g,(s,m) in G.items():
    for x in m:
        if x not in T: bad.append((g,x))
        E[g].add(x)
print('unknown refs',bad)
# cycle
st={}
def dfs(n,path):
    st[n]=1
    for x in E[n]:
        if st.get(x)==1: print('CYCLE',path+[n,x]);return
        if x not in st: dfs(x,path+[n])
    st[n]=2
for n in list(nodes): 
    if n not in st: dfs(n,[])
# anc
def anc(n,seen=None):
    seen=set() if seen is None else seen
    for x in E[n]:
        if x not in seen: seen.add(x); anc(x,seen)
    return seen
# stage monotonic: task dep on task in later stage
num=lambda s:int(s.split('-')[1])
inv=[(t,x) for t,v in T.items() for x in v['S']+v['I'] if x in T and num(T[x]['stage'])>num(v['stage'])]
print('stage inversions',inv)
for g,(s,m) in G.items():
    for x in m:
        if x in T and T[x]['stage']!=s: print('gate stage mismatch',g,x)
ing=collections.Counter(x for g,(s,m) in G.items() for x in m)
print('multi-gate',[k for k,v in ing.items() if v>1],'ungated',[t for t in T if t not in ing])
a1=anc('G1'); print('G1 ancestors outside PS0/1:',sorted(x for x in a1 if x in T and num(T[x]['stage'])>1), 'EXT in G1:',[x for x in a1 if x.startswith('EXT')])
for g in G:
    if g!='G14' and 'EXT-DEV' in anc(g): print('EXT-DEV ancestor of',g)
print('G2 anc of ORCH-*', all('G2' in anc(t) for t in T if t.startswith('ORCH')))
print('G2 anc VOX-01,SDF-01,MSH-01,SRV-01,API-02',[ 'G2' in anc(t) for t in ('VOX-01','SDF-01','MSH-01','SRV-01','API-02')])
print('G3 anc of ORCH-01',[t for t in T if t.startswith('ORCH') and 'G3' not in anc(t)])
print('PS-1 owners',collections.Counter(v['owner'] for t,v in T.items() if v['stage']=='PS-1'))
print('stages', collections.Counter(v['stage'] for v in T.values()))
print('unassigned',[(t,v['stage']) for t,v in T.items() if 'UNASSIGNED' in v['owner']])
# does anything depend on a 'GOV' owner decision that lacks P0-alpha: check CORE tasks that depend on GOV-0x via ancestors
for t in ['CORE-01','CORE-10','CORE-11']: print(t,'GOV anc',sorted(x for x in anc(t) if x.startswith('GOV')))
# core listing in breakdown
ab=open(P+'agent-task-breakdown.md',encoding='utf8').read()
print('tasks missing in breakdown',[t for t in T if t not in ab])
print('breakdown ids not in ADG',sorted(set(re.findall(r'\b(?:CORE|PROC|API|ORCH|GOV|TEST|PERS|WGEN|SRV|NET|HIST|COMP|RES|VOX|SDF|MSH|SCU|BOO|CLI|ADP|LEG|QA|ITG|INT|XT|SPK|DEV|RND)-\w+',ab))-set(T)-set(G)))
print('=== extra')
for g in ['G2','G3','G4','G10','G11','G13']:
    print(g,'has G1 ancestor:', 'G1' in anc(g), '| G0:', 'G0' in anc(g))
for t in ['TEST-06','XT-02','INT-01','CORE-11','CORE-09']:
    print(t,'ancestor of G4:',t in anc('G4'),'of G10:',t in anc('G10'),'of G11:',t in anc('G11'))
for t in ['PROC-01','API-02','ORCH-01','PERS-01','WGEN-01','NET-01']:
    print(t,'G1 anc',  'G1' in anc(t),'G0 anc','G0' in anc(t))
# longest path in task nodes
import functools
@functools.lru_cache(None)
def lp(n):
    best=(0,[n])
    for x in E[n]:
        l,p=lp(x)
        if l>best[0]-0: pass
    cand=[lp(x) for x in E[n]]
    if not cand: return (1 if n in T else 0,[n])
    l,p=max(cand,key=lambda z:z[0])
    return (l+(1 if n in T else 0),p+[n])
l,p=lp('G14'); print('longest tasks',l,' -> '.join(p))
print('=== gate chain')
gs=sorted(G,key=lambda g:int(g[1:]))
for i,g in enumerate(gs):
    prev=[h for h in gs[:i] if h in anc(g)]
    print(g,'ancestor gates:',prev)
