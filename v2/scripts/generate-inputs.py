import json,re,pathlib,random
r=pathlib.Path(__file__).resolve().parents[1]
cases=['','#f00','f00','#0000','#ff000080','ff000080','#FF800080','Figma P3 #FF800080','Figma P3 bad','Figma P3','Figma P3  #ff000080','Figma P3\t#ff000080','figma p3 #ff000080','transparent','Transparent',' red','red ','ReD','rebeccapurple','rgb(255, 0, 127 / .5)','rgba(255, 0, 127, 50%)','rgba(100%, 20%, 50%, -1)','rgb(300 -20 0 / 1.5)','rgb(255 0 0','RGB(255 0 0)','rgb(255 0 0) ','rgb(255 0 0))','rgb(255\r0 0)','rgb(255\n0\t0)','rgb(255 10% .5e2)','rgb(1. 0 0)','rgb(1e2-2+3)','hsl(90 100% 50%)','hsla(-450deg,200%,-50%,2)','hsl(.5turn .3 .6 / 10%)','hwb(60 80% 80%)','hwb(none none none)','oklab(50% 10% -20% / .2)','lab(50% 10% -20%)','lch(50% 10% 1rad)','oklch(100% 0 none)','oklch(0% 0 0)','oklch(53% .12 118.34)','oklch(53% .5 118.34)','oklch(1.5 .9 30 / .123456)','oklch(.5 2deg 90)','oklch(.5 -1 30)','oklch(none none none)','rgb(none none none)','color(display-p3 none none none / none)','color(display-p3 1 .502 0 / 0)','color(display-p3 1.6 1.6 1.6)','color(display-p3 1.600002 1.6 1.6)','color(display-p3 1 0 0 / -.001)','color(display-p3 1 .502 0 / .5)','color(display-p3 1.0005 .5025 -.5 / .2)','vec(0,1,0,1)','lrgb(1 0 0)','none','not-a-color','#gggggg','oklch(.5 .1)','color(srgb 1 0 0 / 20%)','color(srgb 1 0 0 / none)','color(srgb 1 0 0 / 50%','color(srgb 1 0 0 1)','color(--unknown 1 2 3)']
profiles=['srgb','srgb-linear','display-p3','a98-rgb','prophoto-rgb','rec2020','xyz','xyz-d50','xyz-d65','--cubehelix','--din99o-lab','--din99o-lch','--hsi','--hsv','--jzazbz','--jzczhz','--lab-d65','--lch-d65','--lchuv','--luv','--okhsl','--okhsv','--xyb','--yiq']
for p in profiles:
 for vs in ['.2 .3 .4','0 0 0','1 1 1','none none none','20% 30% 40% / 50%','-.2 1.2 .5 / -.1']:
  cases.append(f'color({p} {vs})')
for s in ['lab','lch','oklab','oklch','hsl','hwb']:
 for vs in ['none none none','0 0 0','1 0 0','0.5 none 0','0.5 0 none','0.5 .1 none','0.5 .1 90 / -1','0.5 .1 90 / 2','0.5 .1 90 / none']:
  cases.append(f'{s}({vs})')
names=re.findall(r'(\w+):\s*0x', (r/'fixtures/culori/src/colors/named.js').read_text());cases+=names
rng=random.Random(39871)
for _ in range(320):
 s=rng.choice(['rgb','oklch','oklab','hsl','p3']);vs=[rng.uniform(-.3,1.3) for i in range(3)];a=rng.uniform(-.3,1.3)
 if s=='rgb':cases.append(f'rgb({vs[0]*255:.12g} {vs[1]*255:.12g} {vs[2]*255:.12g} / {a:.12g})')
 elif s=='p3':cases.append(f'color(display-p3 {vs[0]:.12g} {vs[1]:.12g} {vs[2]:.12g} / {a:.12g})')
 elif s=='oklch':cases.append(f'oklch({vs[0]:.12g} {abs(vs[1])*.4:.12g} {vs[2]*360:.12g} / {a:.12g})')
 elif s=='hsl':cases.append(f'hsl({vs[0]*360:.12g} {vs[1]:.12g} {vs[2]:.12g} / {a:.12g})')
 else:cases.append(f'oklab({vs[0]:.12g} {vs[1]*.4:.12g} {vs[2]*.4:.12g} / {a:.12g})')
(r/'fixtures/oracle/inputs.json').write_text(json.dumps(list(dict.fromkeys(cases)),indent=2)+'\n')
# Additional independent numeric, parser, gamut and transfer threshold boundaries.
extra=[]
for n in ['0','.039279999999','.03928','.039280000001','.040449999999','.04045','.040450000001','.49999999999999994','.5','.5000000000000001','1','1.000001','1.00000100001','1e20','1e21','1e100','1e308','1e-320']:
 extra += [f'color(srgb {n} {n} {n} / .5000000000000001)',f'color(display-p3 {n} 0 0)',f'oklch({n} 0 none)']
for x in ['rgb(1 2 3)garbage','color( srgb 1 0 0)','color(srgb 1 0 0))','color(srgb 1 0 0 / .5 / .5)','color(srgb 1 0 0 / none)','hsl(1 20% 30% / none)','hsl(10, 20%, 30%) ','hsl(10, .2, .3)','rgb(1%, 2, 3)','rgb(1 2 3 / 1deg)','oklch(.5 1turn 0)','lab(50% none 0)','Figma P3 #ff800000','Figma P3 #ff8000ff','Figma P3 #ff8000','Figma P3#ff8000ff','Figma P3 \ufeff#12345678']:
 extra.append(x)
# Round-trip actual original rendered outputs, including deliberately unparseable
# vec() and unusual alpha-less P3/OK outputs, without assuming format symmetry.
prior=r/'fixtures/oracle/results.json'
if prior.exists():
 original=json.loads(prior.read_text())
 for query in ['red','#ff000080','Figma P3 #FF800080','oklch(53% .5 118.34)','oklab(50% 10% -20% / .2)','rgba(255, 0, 127, 50%)','color(display-p3 1.6 1.6 1.6)','rgb(none none none)']:
  c=next(c for c in original if c['input']==query)
  extra += [d['value'] for d in c['data']]
inputs=list(dict.fromkeys(cases+extra));(r/'fixtures/oracle/inputs.json').write_text(json.dumps(inputs,indent=2)+'\n')

inputs+=['#12345678\n','#12345678\r','#abc\n','#abc\r\n','rgba(255, 0, 0, .5)\n','rgba(255, 0, 0, .5)\r\n','color(','color()','rgb(1foo 2 3)','rgb(1px 2 3)','color(srgb 1px 0 0)']
(r/'fixtures/oracle/inputs.json').write_text(json.dumps(list(dict.fromkeys(inputs)),indent=2)+'\n')
# The source's hardcoded vec neighborhoods (including its empty-string annulus)
# are reached after its *first* transfer function, not from literal vec input.
refs=[[-.10584,.96562,-.07085],[.10936,.00112,.87233],[.87235,.97693,-.05886],[1.17638,.18288,-.03661],[1.09832,.14968,.45634],[-.10584+.007,.96562,-.07085]]
for ref in refs:
 values=[v*12.92 if v<0 else 1.055*v**(1/2.4)-.055 for v in ref]
 inputs.append('color(srgb '+' '.join(format(v,'.17g') for v in values)+' / .2)')
for a in ['.9999999999999999','1','1.0000000000000002','.00049999999999999','.0005','.0005000000000000001','-.0000001']:
 inputs.append(f'rgb(31 127 213 / {a})')
(r/'fixtures/oracle/inputs.json').write_text(json.dumps(list(dict.fromkeys(inputs)),indent=2)+'\n')

for fn,values in [('oklab',['.5','.1','-.1']),('oklch',['.5','.1','90'])]:
 for mask in range(1,8):
  inputs.append(fn+'('+' '.join('none' if mask&(1<<i) else v for i,v in enumerate(values))+')')
(r/'fixtures/oracle/inputs.json').write_text(json.dumps(list(dict.fromkeys(inputs)),indent=2)+'\n')
