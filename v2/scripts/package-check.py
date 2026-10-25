#!/usr/bin/env python3
"""Package + self-contained extracted Rust and optional cached Node checks, offline."""
import json, os, subprocess, sys, tarfile, tempfile
from pathlib import Path
root=Path(__file__).resolve().parents[1]
def run(args,cwd=root,env=None):
    subprocess.run(args,cwd=cwd,check=True,env=env)
run(['cargo','+stable','package','--locked','--offline','--allow-dirty'])
listing=subprocess.check_output(['cargo','+stable','package','--locked','--offline','--allow-dirty','--list'],cwd=root).decode().splitlines()
assert not any('node_modules/' in p or '/target/' in p or p.startswith('.validation/') for p in listing)
mandatory=[p['path'] for p in json.loads((root/'provenance.json').read_text())['files']]
mandatory+=['manifest.json','INTEGRATION.md','VALIDATION.md','README.md','adapter/css_calculations.rs','validation/host/Cargo.toml.in','validation/host/Cargo.lock.in','validation/host/lib.rs.in','validation/host/contract.rs.in','validation/tauri/Cargo.toml.in','validation/tauri/lib.rs.in']
assert set(mandatory)<=set(listing),set(mandatory)-set(listing)
dest=Path(tempfile.mkdtemp(prefix='extracted-',dir=root/'.validation'))
archive=root/'target/package/css-calculations-native-v2-0.1.0.crate'
with tarfile.open(archive) as tar:
    for member in tar.getmembers():
        target=(dest/member.name).resolve()
        assert dest in target.parents and not member.issym() and not member.islnk()
    tar.extractall(dest)
extracted=dest/'css-calculations-native-v2-0.1.0'
run(['python3','scripts/materialize.py'],extracted)
for manifest in ['Cargo.toml','.validation/host/Cargo.toml']:
    run(['cargo','+stable','test','--manifest-path',manifest,'--locked','--offline'],extracted)
    run(['cargo','+stable','clippy','--manifest-path',manifest,'--locked','--offline','--all-targets','--','-D','warnings'],extracted)
run([str(root/'.validation/python/bin/python'),'scripts/verify.py'],extracted)
if '--node' in sys.argv:
    run(['npm','ci','--offline','--ignore-scripts','--prefix','fixtures/own'],extracted)
    run(['cargo','+stable','build','--locked','--offline','--bin','css-oracle-driver'],extracted)
    env=os.environ.copy();env['CSS_WIRE_OUTPUT']=str(extracted/'.validation/wire.json')
    run(['cargo','+stable','test','--manifest-path','.validation/host/Cargo.toml','--locked','--offline'],extracted,env)
    run(['node','scripts/oracle.cjs'],extracted)
    run(['node','scripts/renderer.cjs'],extracted)
    run(['npm','test','--prefix','fixtures/own'],extracted)
    run(['npm','run','typecheck','--prefix','fixtures/own'],extracted)
print(f'Package {len(listing)} files; {len(mandatory)} mandatory paths present; extracted offline checks passed: {extracted}')
