"""Exact field comparison of the production Rust engine against TSX oracle."""
import json,subprocess,pathlib,collections
r=pathlib.Path(__file__).resolve().parents[1]
cs=json.loads((r/'fixtures/oracle/results.json').read_text());exe=r/'target/debug/examples/convert'
out=subprocess.check_output([str(exe)],input=''.join(json.dumps(c['input'])+'\n' for c in cs),text=True)
actual=[json.loads(l) for l in out.splitlines()];err=[];count=collections.Counter()
for c,a in zip(cs,actual):
 if 'Err'in a:err.append([c['input'],a]);continue
 a=a['Ok'];e=c['data']
 if len(a)!=len(e):err.append([c['input'],'length',len(e),len(a)]);continue
 diffs=[]
 for expected,actual in zip(e,a):
  for k,v in expected.items():
   if actual.get(k)!=v:diffs.append([expected['format'],k,v,actual.get(k)]);count[k]+=1
 if diffs:err.append([c['input'],diffs])
(r/'validation-output').mkdir(exist_ok=True)
(r/'validation-output/diff.json').write_text(json.dumps(err,indent=2)+'\n');print(dict(count),len(err));print(json.dumps(err,indent=2))
