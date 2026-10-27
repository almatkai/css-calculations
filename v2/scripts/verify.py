#!/usr/bin/env python3
"""Full pinned draft-2020-12 schema, negatives, exact provenance and Git visibility."""
import copy, hashlib, json, subprocess
from pathlib import Path
from jsonschema import Draft202012Validator
root = Path(__file__).resolve().parents[1]
def load(p): return json.loads((root/p).read_text())
schema = load('fixtures/beta/schemas/extension-v2.schema.json')
Draft202012Validator.check_schema(schema)
validator = Draft202012Validator(schema)
manifest = load('manifest.json')
validator.validate(manifest)
negatives = []
for key,value in [('schemaVersion',1),('id','raycast.css-calculations'),('runtime','javascript'),('icon','assets/css.png'),('author','teziovsky'),('platforms',['macos']),('commands',[]),('replaces',[]),('verification',[{'platform':'macOS','state':'not-run'}])]:
    item=copy.deepcopy(manifest);item[key]=value;negatives.append(item)
for key in schema['required']:
    item=copy.deepcopy(manifest);del item[key];negatives.append(item)
for path,value in [('runtime','go'),('platforms',['Darwin'])]:
    item=copy.deepcopy(manifest);item['commands'][0][path]=value;negatives.append(item)
for item in negatives: assert list(validator.iter_errors(item)), item
assert manifest['id']=='com.tezbar.native.css-calculations'
assert manifest['slug']=='css-calculations'
assert manifest['replaces']==['raycast.css-calculations']
assert {(v['platform'],v['state']) for v in manifest['verification']}=={('macOS','not-run'),('Windows','not-run')}
own=load('fixtures/own/package.json')
assert manifest['preferences']==[]
assert len(manifest['commands'])==len(own['commands'])==2
for actual,source in zip(manifest['commands'],own['commands']):
    for key in ['name','title','description','mode','platforms']: assert actual[key]==source[key]
    assert actual['runtime']=='rust' and actual['preferences']==[] and actual['arguments']==[]
provenance=load('provenance.json')
paths=[]
for entry in provenance['files']:
    path=root/entry['path'];data=path.read_bytes()
    assert len(data)==entry['bytes'],path
    assert hashlib.sha256(data).hexdigest()==entry['sha256'],path
    paths.append('v2/'+entry['path'])
icon=(root/'assets/css.png').read_bytes()
assert len(icon)==223702 and hashlib.sha256(icon).hexdigest()=='d05434a703c9ac222778979ec8d451ebf439456520d4d2aa2c7ef04be770978f'
assert icon==(root/'fixtures/original/assets/css.png').read_bytes()
assert 'author' not in manifest
if (root.parent/'.git').is_dir():
    # Every provenance file must be tracked or visible to git add, without force.
    listed=subprocess.check_output(['git','ls-files','--cached','--others','--exclude-standard','-z'],cwd=root.parent).decode().split('\0')
    assert set(paths)<=set(listed),set(paths)-set(listed)
    # Existing V1 tracked files remain byte-identical to the recorded own commit.
    base=provenance['ownCommit']
    old_paths=subprocess.check_output(['git','ls-tree','-r','--name-only',base],cwd=root.parent).decode().splitlines()
    for path in old_paths:
        data=subprocess.check_output(['git','show',f'{base}:{path}'],cwd=root.parent)
        if path!='README.md': assert (root.parent/path).read_bytes()==data,path
        assert (root/'fixtures/own'/path).read_bytes()==data,path
    assert not subprocess.check_output(['git','diff','--cached','--name-only'],cwd=root.parent).strip(),'Index must remain empty'
print(f'Schema passed, {len(negatives)} negatives rejected; {len(paths)} provenance/icon/license hashes and Git visibility passed')
