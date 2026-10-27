"""Actual original TSX execution with a per-process watchdog for its infinite loop."""
import json,subprocess,pathlib
r=pathlib.Path(__file__).resolve().parents[1];cases=['rgb(1e999 0 0)','rgb(0 0 0 / -1e999)','oklch(.5 1e999 0)'];out=[]
for text in cases:
 try:
  p=subprocess.run(['node',str(r/'scripts/oracle.cjs'),'--input',text],capture_output=True,text=True,timeout=2,check=True)
  out.append({'input':text,'original':json.loads(p.stdout),'nativePolicy':'explicit non-finite input error'})
 except subprocess.TimeoutExpired:
  out.append({'input':text,'original':'watchdog timeout after 2 seconds (original unbounded chroma fallback loop)','nativePolicy':'explicit non-finite input error'})
(r/'fixtures/oracle/nonfinite.json').write_text(json.dumps(out,indent=2)+'\n')
print('Original non-finite cases recorded with bounded subprocess execution:',len(out))
