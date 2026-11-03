import glob,re,os,collections
rows=[]
for f in sorted(glob.glob(r'*.runs.txt')):
    label=f.replace('.runs.txt','')
    runs=[l for l in open(f,encoding='utf8',errors='replace') if l.startswith('label=') and 'verdict=' in l]
    n=len(runs); ok=sum('verdict=OK' in l for l in runs)
    segv=sum('sigsegv=yes' in l for l in runs); anr=sum('anr=yes' in l for l in runs)
    sigs=collections.Counter()
    for t in glob.glob(f'{label}.run*.tombstone.txt'):
        txt=open(t,encoding='utf8',errors='replace').read()
        m=re.findall(r'signal (\d+) \((SIG\w+)\)',txt)
        for _,name in m[:1]: sigs[name]+=1
        if 'berberis_HandleNoExec' in txt: sigs['(in berberis_HandleNoExec)']+=1
    rows.append((label,n,ok,n-ok,segv,anr,dict(sigs)))
for r in rows: print(r)
