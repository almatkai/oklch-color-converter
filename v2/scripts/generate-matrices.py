"""Extract ONLY the pinned linear expressions; generated output is native Rust.
No code or JavaScript engine runs in the production port. Regeneration is reviewable.
"""
from pathlib import Path
import re
root=Path(__file__).resolve().parents[1]
src=root/'fixtures/culori/src'
# file, input channel identifiers, output identifiers, input transfer, output transfer
entries=[('xyz65/convertRgbToXyz65.js','rgb','xyz','linear',''),('p3/convertP3ToXyz65.js','rgb','xyz','linear',''),('a98/convertA98ToXyz65.js','rgb','xyz','a98_linear',''),('rec2020/convertRec2020ToXyz65.js','rgb','xyz','rec_linear',''),('prophoto/convertProphotoToXyz50.js','rgb','xyz','pro_linear',''),('xyz65/convertXyz65ToRgb.js','xyz','rgb','','gamma'),('p3/convertXyz65ToP3.js','xyz','rgb','','gamma'),('xyz50/convertXyz50ToRgb.js','xyz','rgb','','gamma'),('xyz65/convertXyz50ToXyz65.js','xyz','xyz','','')]
text='// Generated from immutable Culori 3.3.0 expressions by scripts/generate-matrices.py.\n#![allow(clippy::excessive_precision)] // Preserve the original decimal coefficient spellings.\nuse super::{Color,linear,gamma,sign};\n'
text+='fn a98_linear(v:f64)->f64{v.abs().powf(563.0/256.0)*sign(v)}\nfn rec_linear(v:f64)->f64{let a=v.abs();if a<0.018053968510807*4.5{v/4.5}else{(if sign(v)!=0.0{sign(v)}else{1.0})*((a+1.09929682680944-1.0)/1.09929682680944).powf(1.0/0.45)}}\nfn pro_linear(v:f64)->f64{let a=v.abs();if a>=16.0/512.0{sign(v)*a.powf(1.8)}else{v/16.0}}\n'
for file,ins,outs,pre,post in entries:
 s=(src/file).read_text(); name=re.search(r'convert(\w+)To(\w+)\.js',file)
 a,b=name.groups(); name=re.sub(r'([a-z0-9])([A-Z])',r'\1_\2',a+'To'+b).lower()
 mode={'Rgb':'rgb','P3':'p3','Xyz65':'xyz65','Xyz50':'xyz50'}[b]
 expr=[]
 for ch in outs:
  matches=list(re.finditer(r'\b'+ch+r':\s*([\s\S]*?)(?=,|\n\s*})',s))
  val=matches[-1].group(1).strip()
  val=re.sub(r'(?<![\w.])(\d+)(?![\w.])',lambda m:m[1]+'.0',val)
  expr.append(' '.join(val.split()))
 text+=f'pub(super) fn {name}(c:Color)->Color{{let [{",".join(ins)}]=c.v()'+(f'.map({pre})' if pre else '')+';Color::new("'+mode+'",['+','.join(expr)+']'+(f'.map({post})' if post else '')+',c.alpha)}\n'
(root/'src/convert/matrices.rs').write_text(text)
