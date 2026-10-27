// Execute pinned renderer parser/reconciliation/routing, using real locked React.
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'..');const ts=require(root+'/fixtures/own/node_modules/typescript');const React=require(root+'/fixtures/libraries/react');const jsx=require(root+'/fixtures/libraries/react/jsx-runtime');
let slots=[],cursor=0;
function reset(){slots=[];cursor=0;}
React.__CLIENT_INTERNALS_DO_NOT_USE_OR_WARN_USERS_THEY_CANNOT_UPGRADE.H={
 useState(initial){const i=cursor++;if(!(i in slots))slots[i]=typeof initial==='function'?initial():initial;return[slots[i],v=>slots[i]=typeof v==='function'?v(slots[i]):v];},
 useRef(initial){const i=cursor++;return slots[i]??(slots[i]={current:initial});},
 useMemo(fn){return fn();},useCallback(fn){return fn;},useEffect(){},useImperativeHandle(){},useContext(ctx){return ctx._currentValue;}
};
const empty=()=>null;const provider=React.createContext({});
function load(file,custom={},append=''){
 const source=fs.readFileSync(file,'utf8')+append;
 const output=ts.transpileModule(source,{compilerOptions:{module:ts.ModuleKind.CommonJS,jsx:ts.JsxEmit.ReactJSX,esModuleInterop:true,target:ts.ScriptTarget.ES2020}}).outputText;
 const module={exports:{}};new Function('require','module','exports',output)(id=>id==='react'?React:id==='react/jsx-runtime'?jsx:id in custom?custom[id]:(()=>{throw Error('Unexpected dependency '+id)})(),module,module.exports);return module.exports;
}
const base=root+'/fixtures/beta/src/renderer/src/raycast-api/';
const form=load(base+'form-runtime.tsx',{'../../ui/primitives':new Proxy({},{get:()=>empty}),'./detail-runtime':{NativeColorWheel:empty,parseNativeColorWheel:()=>null}},'\nexport { collectFormItems };');
const surface=load(base+'index.tsx',{'./action-runtime':{Action:empty,ActionPanel:empty,ActionPanelOverlay:empty},'./action-runtime-registry':{ActionRegistryContext:provider},'./detail-runtime':{DetailRuntime:empty},'./form-runtime':form,'./grid-runtime':{GridRuntime:empty},'./list-runtime':{ListRuntime:empty},'./hooks':{NavigationContext:provider}});
const trace=JSON.parse(fs.readFileSync(root+'/.validation/wire.json','utf8'));
function fields(view){return form.collectFormItems(view.root).filter(i=>i.type==='field').map(i=>i.field);}
const first=fields(trace[0]),changed=fields(trace[1]),second=fields(trace[2]);
assert.equal(first.length,5);assert.equal(first[2].value,'375');
let draft={...form.reconcileFormValues(first),min_font_size:'1e2',max_font_size:'newer queued draft'};
draft=form.reconcileFormValues(changed,draft,first);assert.equal(draft.max_font_size,'newer queued draft');assert.equal(draft.min_font_size,'1e2');
draft=form.reconcileFormValues(second,{...draft,min_font_size:'newer A'},changed);assert.equal(draft.min_font_size,'newer A');assert.equal(draft.max_font_size,'200');
assert.deepEqual(form.reconcileFormValues(fields(trace[3]),draft,second),draft);
function walk(node,fn){if(!node||typeof node!=='object')return;fn(node);React.Children.forEach(node.props?.children,child=>walk(child,fn));}
const calls=[];let back=0;reset();
const tree=surface.ExtensionRuntimeSurface({...trace[0],onInvokeAction:(...args)=>calls.push(args),onBack:()=>back++,onSearchTextChanged:empty,onLoadMore:empty});let element;
walk(tree,node=>{if(node.type===form.FormRuntime)element=node;});assert.ok(element);
element.props.onChangeField(first[0].actionId,'42');assert.deepEqual(calls.pop(),[first[0].actionId,{value:'42'}]);
element.props.onSubmitForm(draft);assert.deepEqual(calls.pop(),[trace[0].actions[0].id,draft]);element.props.onBack();assert.equal(back,1);
assert.equal(trace[0].actions[0].kind,'submit-form');
// Execute the real FormRuntime component event handlers and full-map submission.
reset();let submitted,updates=[];
const props={root:trace[0].root,title:trace[0].title,onBack:()=>back++,onSubmitForm:v=>submitted=v,onChangeField:(...v)=>updates.push(v),onInvokeAction:empty,onOpenActions:empty};
let rendered=form.FormRuntime.render(props,null);let inputs=[];walk(rendered,node=>{if(node.type==='input')inputs.push(node);});assert.equal(inputs.length,5);
inputs[0].props.onChange({target:{value:'99'}});assert.deepEqual(updates.pop(),[first[0].actionId,'99']);
cursor=0;rendered=form.FormRuntime.render(props,null);let prevent=0;rendered.props.onSubmit({preventDefault(){prevent++;}});assert.equal(prevent,1);assert.deepEqual(submitted,{min_font_size:'99',max_font_size:'',min_vw_width:'375',max_vw_width:'1400',pixels_per_rem:'16'});
console.log('Pinned renderer parser, queued-draft reconciliation, Surface action routing and FormRuntime events passed');
