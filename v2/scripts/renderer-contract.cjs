#!/usr/bin/env node
// Executes actual pinned beta ListRuntime, its parsers and reconciliation hooks,
// plus the required one-line server-search patch. This is an isolated hook/tree
// harness. UI primitives, unrelated details/color-converter and browser APIs are
// explicit stubs. It is not a browser/desktop visual QA claim.
const fs=require('node:fs'),path=require('node:path'),vm=require('node:vm'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'..'),base=path.join(root,'fixtures/host/src/renderer/src/raycast-api');
if(process.argv[2]==='--compile'){
 const ts=require(path.resolve(process.argv[3],'typescript'));
 const compile=s=>ts.transpileModule(s,{compilerOptions:{target:ts.ScriptTarget.ES2022,module:ts.ModuleKind.CommonJS,jsx:ts.JsxEmit.React,esModuleInterop:true}}).outputText;
 const src=fs.readFileSync(path.join(base,'list-runtime.tsx'),'utf8');
 const before='const isColorConversionSurface = isColorConversionCommand(commandName, title, navigationTitle)';const after='const isColorConversionSurface = !hasServerSearch && isColorConversionCommand(commandName, title, navigationTitle)';
 assert(src.includes(before));assert(fs.readFileSync(path.join(root,'integration/server-search.patch'),'utf8').includes('+'+after.replace(/^/,'  ')));
 const exports='\nexport {parseListRows, parseRowIcon, SymbolIcon};\n';
 fs.writeFileSync(path.join(root,'fixtures/oracle/beta-list-original.cjs'),compile(src+exports));
 fs.writeFileSync(path.join(root,'fixtures/oracle/beta-list-runtime.cjs'),compile(src.replace(before,after)+exports));
 fs.writeFileSync(path.join(root,'fixtures/oracle/beta-action-overlay.cjs'),compile(fs.readFileSync(path.join(base,'action-runtime-overlay.tsx'),'utf8')));
}
function harness(file){
 let slots=[],deps=[],cursor=0,pending=[];const keys={};
 const React={createElement(type,props,...children){return{type,props:props||{},children:children.flat(Infinity).filter(v=>v!==false&&v!==null&&v!==undefined)}},useState(v){const i=cursor++;if(!(i in slots))slots[i]=typeof v==='function'?v():v;return[slots[i],v=>{slots[i]=typeof v==='function'?v(slots[i]):v}]},useMemo(fn){return fn()},useRef(value){const i=cursor++;if(!(i in slots))slots[i]={current:value};return slots[i]},useEffect(fn,ds){const i=cursor++;if(!deps[i]||ds.some((v,j)=>v!==deps[i][j])){deps[i]=ds;pending.push(fn)}}};
 const stub=name=>p=>React.createElement(name,p,p.children);
 const module={exports:{}};let converterCalls=0;
 const sandbox={React,exports:module.exports,module,require:s=>{if(s==='react')return React;if(s.includes('primitives'))return{Hint:stub('Hint'),HintBar:stub('HintBar'),Kbd:stub('Kbd'),ViewHeader:stub('ViewHeader'),cx:(...x)=>x.filter(Boolean).join(' ')};if(s.includes('Markdown'))return{Markdown:stub('Markdown')};if(s.includes('detail-runtime'))return{MetadataItem:stub('MetadataItem'),MetadataSidebar:stub('MetadataSidebar')};if(s.includes('colorConverter'))return{buildColorConversionResults:()=>{converterCalls++;return[{id:'generic',title:'GENERIC OVERRIDE',subtitle:'unrelated converter stub',action:{type:'copy-text',text:'wrong'}}]}};throw Error(s)},setTimeout:()=>1,clearTimeout:()=>{},console,window:{addEventListener:(k,fn)=>{keys[k]=fn},removeEventListener:()=>{}}};
 vm.runInNewContext(fs.readFileSync(path.join(root,'fixtures/oracle',file),'utf8'),sandbox);
 function render(fn,props){cursor=0;let result=fn(props);let work=pending;pending=[];work.forEach(f=>f());cursor=0;return fn(props)}
 return{exports:module.exports,render,get converterCalls(){return converterCalls},keys};
}
function walk(n,pred,out=[]){if(!n||typeof n!=='object')return out;if(pred(n))out.push(n);n.children?.forEach(v=>walk(v,pred,out));return out;}
const view=JSON.parse(fs.readFileSync(process.argv.find(x=>x.endsWith('/view.json'))||path.join(root,'validation-output/view.json'),'utf8'));
assert.equal(view.root.type,'List');let invocations=[];
const props={root:view.root,title:view.title,commandName:'convert-color',onBack(){},onRunPrimaryAction:(...args)=>invocations.push(args),actions:view.actions,onOpenActions(){},onSearchTextChanged(){},onLoadMore(){}};
const original=harness('beta-list-original.cjs');original.render(original.exports.ListRuntime,props);assert(original.converterCalls>0,'original pinned renderer must reproduce override bug');
const h=harness('beta-list-runtime.cjs');let tree=h.render(h.exports.ListRuntime,props);assert.equal(h.converterCalls,0);
const parsed=h.exports.parseListRows(view.root);assert.equal(parsed.length,8);assert.equal(parsed[0].title,view.root.children[0].children[0].props.title);assert.equal(parsed[0].icon.tintColor,view.root.children[0].children[0].props.icon.tintColor);assert.equal(parsed[0].actionIds[0],view.actions[0].id);
let input=walk(tree,n=>n.type==='input')[0];input.props.onChange({target:{value:'newer pending draft'}});tree=h.render(h.exports.ListRuntime,props);input=walk(tree,n=>n.type==='input')[0];assert.equal(input.props.value,'newer pending draft');
const staleResponse={...props,root:JSON.parse(JSON.stringify(view.root))};tree=h.render(h.exports.ListRuntime,staleResponse);assert.equal(walk(tree,n=>n.type==='input')[0].props.value,'newer pending draft','action/refresh response must not overwrite newer local draft');
const buttons=walk(tree,n=>n.type==='button'&&n.props.onClick);const rowButton=buttons.find(n=>walk(n,x=>x.type==='p').some(p=>p.children.includes(parsed[0].title)));assert(rowButton);rowButton.props.onClick();assert.equal(invocations[0][0],view.actions[0].id);
const swatch=h.exports.SymbolIcon({icon:parsed[0].icon.source,tintColor:parsed[0].icon.tintColor,title:parsed[0].title});assert.equal(swatch.props.style.backgroundColor,parsed[0].icon.tintColor);
const menu=harness('beta-action-overlay.cjs');let executed=[];let menuTree=menu.render(menu.exports.ActionPanelOverlay,{actions:view.actions.slice(0,2),onClose(){},onExecute:a=>executed.push(a.id)});assert.deepEqual(walk(menuTree,n=>n.type==='button').map(n=>n.children[0]),['Copy Color Value','Copy as Hex']);menu.keys.keydown({key:'ArrowDown',preventDefault(){}});menuTree=menu.render(menu.exports.ActionPanelOverlay,{actions:view.actions.slice(0,2),onClose(){},onExecute:a=>executed.push(a.id)});menu.keys.keydown({key:'Enter',preventDefault(){}});assert.equal(executed[0],view.actions[1].id);
console.log('Pinned beta parser, swatch, menu, committed action routing and pending-draft reconciliation: PASS (isolated harness; required server-search patch applied only in memory)');
