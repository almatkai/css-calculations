#!/usr/bin/env python3
"""Cargo excludes nested package manifests. Materialize from shipped templates after extraction."""
from pathlib import Path
import shutil
root = Path(__file__).resolve().parents[1]
for name in ['host', 'tauri']:
    template = root/'validation'/name
    dest = root/'.validation'/name
    (dest/'src').mkdir(parents=True, exist_ok=True)
    shutil.copyfile(template/'Cargo.toml.in', dest/'Cargo.toml')
    (dest/'src/lib.rs').write_text((template/'lib.rs.in').read_text().replace('@ROOT@',str(root)))
(root/'.validation/host/tests').mkdir(exist_ok=True)
shutil.copyfile(root/'validation/host/contract.rs.in', root/'.validation/host/tests/contract.rs')
if (root/'validation/host/Cargo.lock.in').exists():
    shutil.copyfile(root/'validation/host/Cargo.lock.in', root/'.validation/host/Cargo.lock')
print('Materialized pinned-beta contract crate')
