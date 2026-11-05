import os
import collections,copy,difflib,re,sys
__file__=os.path.join(os.path.dirname(os.path.abspath(__file__)),'ia.py')
exec(open(__file__,encoding='utf8').read().split("print('tasks'")[0])
def build(T):
    E=collections.defaultdict(set)
    for t,v in T.items():
        for x in v['S']+v['I']: E[t].add(x)
    for g,(s,m) in G.items():
        for x in m: E[g].add(x)
    return E
def anc(E,n,seen=None):
    seen=set() if seen is None else seen
    for x in E[n]:
        if x not in seen: seen.add(x); anc(E,x,seen)
    return seen
def cyc(E):
    st={}
    def d(n):
        st[n]=1
        for x in E[n]:
            if st.get(x)==1: return True
            if x not in st and d(x): return True
        st[n]=2;return False
    return any(d(n) for n in list(E) if n not in st)
def report(T,label):
    E=build(T); gs=['G%d'%i for i in range(15)]
    print('==',label,'cycle:',cyc(E))
    print(' gates not ancestor of G14:',[g for g in gs[:-1] if g not in anc(E,'G14')])
    print(' G1 ancestor of:',[g for g in gs[2:] if 'G1' in anc(E,g)])
    miss=[t for t,v in T.items() if num(v['stage'])>=2 and not t.startswith(('GOV','SPK','DEV')) and 'G1' not in anc(E,t)]
    print(' PS-2+ engineering tasks without G1 ancestor:',miss)
    return E
num=lambda s:int(s.split('-')[1])
report(T,'BEFORE')
T1=copy.deepcopy(T)
for t in ['PROC-01','CORE-15','CORE-16']: T1[t]['I']=T1[t]['I']+['G1']
report(T1,'AFTER D-01 only')
T2=copy.deepcopy(T1)
T2['QA-04']['I']=T2['QA-04']['I']+['G1','G5','G6','G7','G8','G9','G11','G12']
E2=report(T2,'AFTER D-01 + D-02')
# consecutive-gate chain check
gs=['G%d'%i for i in range(15)]
print(' every Gn has G(n-1) as ancestor?',{g:(gs[i-1] in anc(E2,g)) for i,g in enumerate(gs) if i})
# longest chain changes
import functools
def lp(E):
    @functools.lru_cache(None)
    def f(n):
        c=[f(x) for x in E[n]]
        if not c: return (1 if n in T else 0,(n,))
        l,p=max(c,key=lambda z:z[0]); return (l+(1 if n in T else 0),p+(n,))
    return f('G14')
print(' longest before',lp(build(T))[0],' after',lp(E2)[0])
# diff of rows
def row(t,v): return '| %s | %s | %s | %s | %s |'%(t,v['stage'],v['owner'],', '.join(v['S']) or '-',', '.join(v['I']) or '-')
out=[]
for t in ['PROC-01','CORE-15','CORE-16','QA-04']:
    out+= ['--- %s'%t,'- '+row(t,T[t]),'+ '+row(t,T2[t])]
print('\n'.join(out))
print('== RULE variant')
T3=copy.deepcopy(T)
n=0
for t,v in T3.items():
    if num(v['stage'])>=2 and not t.startswith(('GOV','SPK','DEV')) and 'G1' not in v['I']+v['S']:
        v['I']=v['I']+['G1']; n+=1
T3['QA-04']['I']=T3['QA-04']['I']+['G5','G6','G7','G8','G9','G11','G12']
E3=build(T3); print(' tasks given I:G1 =',n,' cycle:',cyc(E3),' longest',lp(E3)[0],' gates not anc of G14:',[g for g in gs[:-1] if g not in anc(E3,'G14')])
print(' eng tasks w/o G1 anc:',[t for t,v in T3.items() if num(v['stage'])>=2 and not t.startswith(('GOV','SPK','DEV')) and 'G1' not in anc(E3,t) and 'G1' not in v['I']])
print('== prior-gate ancestry now (before fix)')
for i in range(1,15):
    print(' G%d'%i, [g for g in gs[:i] if g in anc(build(T),'G%d'%i)])
