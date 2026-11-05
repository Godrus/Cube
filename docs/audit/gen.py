import re
exec(open('ia.py').read().split("print('tasks'")[0])
mb=open(P+'module-boundaries.md',encoding='utf8').read()
names={0:'Governance and Architecture Freeze',1:'Geometry Core Foundation (P0-alpha)',2:'Processing Foundation',3:'Editing API and Contracts',4:'Editing Orchestration',5:'Geometry Backends and Derived Providers',6:'Tools: Sculpting and Boolean',7:'History',8:'Composite Geometry',9:'Resampling and Resolution Refinement',10:'Persistence',11:'World Generation',12:'Client Runtime and Engine Adapter',13:'Server Runtime and Networking',14:'Full QA and Release Validation'}
# part D stage table rows
dd=mb[mb.index('| Stage | Start-after'):mb.index('## Part E')]
D={}
for l in dd.splitlines():
    m=re.match(r'\| (PS-\d+) \| (.*?) \| (.*?) \| (.*?) \| (G\d+: .*) \|$',l)
    if m: D[m.group(1)]=m.groups()[1:]
blk={0:'BLK-01,02,03,07,09',1:'G0 unsatisfied',2:'BLK-04 (EXT-GPS), G1 intent (D-01)',3:'G2',4:'G2, G3',5:'BLK-05 (RG-1), G2, G3',6:'G4, G5',7:'G4; OD-R5 (GOV-25)',8:'G4; GOV-19/23',9:'BLK-05, G4, G5',10:'BLK-05, SPK-PERS, GOV-10/11',11:'BLK-07 (owner), G10, GOV-11/21',12:'BLK-07 (owner), ORCH/HIST',13:'BLK-07, G10, GOV-12/22; SPK-NET',14:'BLK-10 (EXT-DEV), G13'}
rows=['| Stage / Gate | Name | Status | Done | Remaining | Acceptance (closing condition, ADG Part D) | Evidence | Blockers | Dependencies (start-after / merge-after) | To pass on |','|---|---|---|---|---|---|---|---|---|---|']
for i in range(15):
    ps='PS-%d'%i;g='G%d'%i
    mem=G[g][1]; own=[t for t in T if T[t]['stage']==ps]
    extra=[t for t in own if t not in mem]
    st='**Open** (current stage)' if i==0 else 'Not Started'
    s,mg,gin,close=D[ps]
    rows.append('| %s / %s | %s | %s | 0 of %d gate tasks%s | %d gate tasks: %s | %s | none | %s | S: %s; M: %s | %s satisfied |'%(ps,g,names[i],st,len(mem),(' (+%d outside gate: %s)'%(len(extra),', '.join(extra)) if extra else ''),len(mem),', '.join(mem),close.split(': ',1)[1],blk[i],s,mg,g))
tab='\n'.join(rows)
f='C:/Users/Godrus/AppData/Local/Temp/claude/9a9f99d73db6a0d5/fe08ec09-37a6-45d2-8d52-168209f62bea/scratchpad/audit-v3/PROJECT_STATUS.md'
s=open(f,encoding='utf8').read().replace('{{STAGE_TABLE}}',tab)
open(f,'w',encoding='utf8').write(s)
print(len(rows)-2, 'rows'); print(rows[2][:600])
