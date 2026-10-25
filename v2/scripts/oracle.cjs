// Validation only: execute exact pinned TS and locked React. No JS in runtime.
const fs=require('node:fs');const path=require('node:path');const assert=require('node:assert/strict');const {spawnSync}=require('node:child_process');
const root=path.resolve(__dirname,'..');const ts=require(root+'/fixtures/own/node_modules/typescript');
function load(file,custom={}) {
 const source=fs.readFileSync(file,'utf8');const output=ts.transpileModule(source,{compilerOptions:{module:ts.ModuleKind.CommonJS,jsx:ts.JsxEmit.React,esModuleInterop:true,target:ts.ScriptTarget.ES2020}}).outputText;
 const module={exports:{}};new Function('require','module','exports',output)((id)=>id in custom?custom[id]:require(id),module,module.exports);return module.exports;
}
const utils=load(root+'/fixtures/own/src/utils.ts');
const keys=['min_font_size','max_font_size','min_vw_width','max_vw_width','pixels_per_rem'];
const defaults={min_font_size:'16',max_font_size:'32',min_vw_width:'375',max_vw_width:'1400',pixels_per_rem:'16'};
const edge=['',' ','\t\r\n','\u00a0','\ufeff16\u2028','\u008516','\u180e16','NaN','Infinity','-Infinity','+Infinity','inf','infinity','0','-0','+0','0.0000','-0.0000001','0x10','0Xff','0b11','0o17','0x1_0','+0x10','-0x10','0x','0b2','1e3','1e-300','1e309','1e-999','-1e999','1e','1e+','1.2x','16px','1,2','.5','1.','+.5','0.00005','1.00005','1.23445','999999999999999999999','1e21','1e20','2.2250738585072014e-308','5e-324','9007199254740993','日本','😀','١٦','１６','1\0','1_000','0xff_ff','0x'+ 'f'.repeat(400),null,true,false,0,-1,0.1,1e21,[],[null],['16'],[1,2],{}];
const cases=[];for(const mode of ['px','rem']) {
 for(const key of keys)for(const item of edge)cases.push({mode,values:{...defaults,[key]:item}});
 for(const [lo,hi] of [['1400','375'],['375','375'],['0','0'],['375.00000000000006','375'],['1e308','1e308'],['-1','-2']])cases.push({mode,values:{...defaults,min_vw_width:lo,max_vw_width:hi}});
}
let seed=0x1987abcd;function rand(){seed^=seed<<13;seed^=seed>>>17;seed^=seed<<5;return seed>>>0;}
const buf=new ArrayBuffer(8);const dv=new DataView(buf);
function randomNumber(){dv.setUint32(0,rand());dv.setUint32(4,rand());const n=dv.getFloat64(0);return Number.isFinite(n)?n:String(n);}
for(let i=0;i<12000;i++){
 let v={};for(const key of keys){let n=i%3===0?(rand()%1000000)/((rand()%1000)+1):randomNumber();v[key]=i%2?String(n):n;}
 cases.push({mode:i%2?'rem':'px',values:v});
}
// Dense rounding ties and adjacent f64s through the actual formula.
for(let i=-2000;i<=2000;i++){cases.push({mode:'rem',values:{...defaults,min_font_size:String(i/10000+0.00005),max_font_size:String(i/9999+1),min_vw_width:'1',max_vw_width:'2'}});}
const input=cases.map(c=>JSON.stringify(c)).join('\n')+'\n';
const run=spawnSync(process.env.CSS_RUST_DRIVER||root+'/target/debug/css-oracle-driver',[],{input,encoding:'utf8',maxBuffer:100*1024*1024});if(run.status!==0)throw new Error(run.stderr);
const actual=run.stdout.trim().split('\n').map(JSON.parse);assert.equal(actual.length,cases.length);
function sortErrors(e){return e.sort((a,b)=>a.key.localeCompare(b.key));}
cases.forEach((c,i)=>{
 const errors=utils.validateAllValues(c.values);const expected={css:utils.calculateClamp(c.values,c.mode),errors:sortErrors(errors),change:keys.map(k=>utils.validateValue(k,Number(c.values[k]))??null)};
 actual[i].errors=sortErrors(actual[i].errors);assert.deepEqual(actual[i],expected,`case ${i}: ${JSON.stringify(c)}`);
});
console.log(`${cases.length} exact CSS + submit validation + Number(onChange) oracle cases passed`);
// Execute both actual TSX components and actual hook source, with locked React's
// dispatcher seam and fake Raycast effects. Nothing calls the native Raycast API.
const React=require(root+'/fixtures/libraries/react');const calls=[];let slots=[],cursor=0;
React.__CLIENT_INTERNALS_DO_NOT_USE_OR_WARN_USERS_THEY_CANNOT_UPGRADE.H={useState(initial){const i=cursor++;if(!(i in slots))slots[i]=initial;return[slots[i],v=>slots[i]=v];}};
const hooks=load(root+'/fixtures/own/src/hooks.ts',{react:React});
const api={Action:{SubmitForm:'Submit'},ActionPanel:'Actions',Form:Object.assign(function Form(){},{TextField:'TextField'}),Clipboard:{copy:async text=>{calls.push(['copy',text]);}},showHUD:async text=>{calls.push(['hud',text]);}};
(async()=>{
 for(const mode of ['px','rem']){
  slots=[];const component=load(root+`/fixtures/own/src/clamp_${mode}.tsx`,{react:React,'@raycast/api':api,'./hooks':hooks,'./utils':utils}).default;
  function render(){cursor=0;return component();}
  let tree=render();assert.equal(tree.props.children.length,5);assert.equal(tree.props.children[2].props.defaultValue,'375');assert.equal(tree.props.children[4].props.defaultValue,'16');
  tree.props.children[0].props.onChange('0');tree=render();assert.equal(tree.props.children[0].props.error,'Min font size can not be empty!');
  tree.props.children[2].props.onChange('-1');tree=render();const previous=tree.props.children[2].props.error;
  await tree.props.actions.props.children.props.onSubmit({...defaults,min_vw_width:'bad'});tree=render();assert.equal(tree.props.children[2].props.error,previous);
  const start=calls.length;await tree.props.actions.props.children.props.onSubmit(defaults);assert.deepEqual(calls.slice(start),[['copy',utils.calculateClamp(defaults,mode)],['hud','Clamp Copied 🎊']]);
  const zero={...defaults,min_font_size:'0'};await tree.props.actions.props.children.props.onSubmit(zero);assert.equal(calls.at(-2)[1],utils.calculateClamp(zero,mode));
 }
 console.log('Both pinned TSX commands and hooks executed: defaults, change/submit mismatch, hidden error keys, copy → HUD ordering');
})().catch(e=>{console.error(e);process.exitCode=1});
