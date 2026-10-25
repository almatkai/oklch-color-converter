#!/usr/bin/env node
/* Execute the unchanged pinned own TSX + locked Culori. A hook/element harness
 * replaces only Raycast UI and React scheduling. No color math is mocked.
 * Usage: node scripts/oracle.cjs [disposable-own-node_modules]
 * With no argument, use included transpiled TSX + original Culori fixture offline.
 */
const fs=require('node:fs'),path=require('node:path'),vm=require('node:vm');
const root=path.resolve(__dirname,'..');
const source=fs.readFileSync(path.join(root,'fixtures/own/src/convert-color.tsx'),'utf8');
const extra='\nexport {formatColor, processFigmaP3, isP3HexFormat};\n';
const compiled=path.join(root,'fixtures/oracle/convert-color.cjs');
if(process.argv[2] && process.argv[2]!=='--input'){
 const ts=require(path.resolve(process.argv[2],'typescript'));
 fs.writeFileSync(compiled,ts.transpileModule(source+extra,{compilerOptions:{target:ts.ScriptTarget.ES2022,module:ts.ModuleKind.CommonJS,jsx:ts.JsxEmit.React,esModuleInterop:true}}).outputText);
}
(async()=>{
 const culori=await import(require('node:url').pathToFileURL(path.join(root,'fixtures/culori/src/index.js')).href);
 let slots=[],deps=[],cursor=0,pending=[],toasts=[];
 const React={createElement(type,props,...children){return{type,props:props||{},children:children.flat(Infinity).filter(v=>v!==false&&v!==null&&v!==undefined)};},useState(value){const i=cursor++;if(!(i in slots))slots[i]=value;return[slots[i],v=>{slots[i]=v}];},useEffect(fn,ds){const i=cursor++;if(!deps[i]||ds.some((v,j)=>v!==deps[i][j])){deps[i]=ds;pending.push(fn);}}};
 const component=name=>Object.assign(function(){},{displayName:name});
 const L=Object.assign(component('List'),{Item:component('List.Item'),Section:component('List.Section')});
 const A=Object.assign(component('Action'),{CopyToClipboard:component('Action.CopyToClipboard')});
 const AP=Object.assign(component('ActionPanel'),{Section:component('ActionPanel.Section')});
 const api={List:L,Action:A,ActionPanel:AP,Icon:{CircleFilled:'Icon.CircleFilled'},showToast:t=>{toasts.push(t)},Toast:{Style:{Failure:'failure'}}};
 const module={exports:{}};
 const sandbox={exports:module.exports,module,require:s=>{if(s==='react')return React;if(s==='@raycast/api')return api;if(s==='culori')return culori;throw Error(s);},console};
 vm.runInNewContext(fs.readFileSync(compiled,'utf8'),sandbox,{filename:'pinned-own-convert-color.cjs',timeout:2000});
 const render=()=>{cursor=0;module.exports.default();let work=pending;pending=[];work.forEach(fn=>fn());cursor=0;return module.exports.default();};
 const normalize=n=>typeof n==='object'?{type:n.type?.displayName||n.type,props:Object.fromEntries(Object.entries(n.props).filter(([k,v])=>typeof v!=='function'&&k!=='actions')),actions:n.props.actions?normalize(n.props.actions):undefined,children:n.children.map(normalize)}:n;
 const collect=(n,type,out=[])=>{if(n.type?.displayName===type)out.push(n);n.children?.forEach(v=>{if(v&&typeof v==='object')collect(v,type,out)});return out};
 let cases=process.argv[2]==='--input'?[process.argv[3]]:JSON.parse(fs.readFileSync(path.join(root,'fixtures/oracle/inputs.json'),'utf8'));
 const results=cases.map(input=>{slots=[];deps=[];cursor=0;pending=[];toasts=[];let tree=render();tree.props.onSearchTextChange(input);tree=render();const items=collect(tree,'List.Item');let color;try{color=input.startsWith('Figma P3')?module.exports.processFigmaP3(input):module.exports.isP3HexFormat(input)?module.exports.processFigmaP3('Figma P3 '+input):culori.parse(input);}catch{}let data=color?['figmaP3','oklch','p3','oklab','vec','hex','rgb','hsl'].map(f=>module.exports.formatColor(color,f)):[];return{input,data,rendered:normalize(tree),toasts};});
 if(process.argv[2]==='--input'){console.log(JSON.stringify(results));return;}
 fs.writeFileSync(path.join(root,'fixtures/oracle/results.json'),JSON.stringify(results,null,2)+'\n');
 console.log(`Executed unchanged TSX + Culori 3.3.0 for ${results.length} cases`);
})();
