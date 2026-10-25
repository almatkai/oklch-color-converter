#!/usr/bin/env python3
"""Materialize the included harness manifest only for a single Cargo invocation.
Cargo excludes nested packages even when include/** matches them; keeping the
manifest as a template ensures the packaged tests really remain self-contained.
"""
from pathlib import Path
import subprocess,sys
r=Path(__file__).resolve().parents[1];manifest=r/'harness/Cargo.toml'
if manifest.exists():raise SystemExit('Refusing to overwrite an existing harness/Cargo.toml')
args=sys.argv[1:];toolchain=[]
if args and args[0].startswith('+'):toolchain=[args.pop(0)]
if not args:args=['test','--locked']
command=args.pop(0)
manifest.write_bytes((r/'harness/Cargo.toml.in').read_bytes())
try:
 result=subprocess.run(['cargo',*toolchain,command,'--manifest-path',str(manifest),*args])
finally:manifest.unlink()
raise SystemExit(result.returncode)
