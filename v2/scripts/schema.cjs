#!/usr/bin/env node
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict'),crypto=require('node:crypto');
const root=path.resolve(__dirname,'..');
const Ajv=require('../fixtures/validators/node_modules/ajv/dist/2020').default;
const schema=JSON.parse(fs.readFileSync(path.join(root,'fixtures/host/schemas/extension-v2.schema.json')));
const ajv=new Ajv({strict:true,allErrors:true});const valid=ajv.compile(schema);
const manifest=JSON.parse(fs.readFileSync(path.join(root,'extension-v2.json')));
assert(valid(manifest),JSON.stringify(valid.errors));
for(const mutate of [m=>m.schemaVersion=1,m=>m.runtime='javascript',m=>m.author='strongeron',m=>m.id='raycast.oklch-color-converter',m=>m.icon='/wrong.png',m=>m.commands=[],m=>delete m.commands[0].platforms,m=>m.commands[0].platforms=['macos'],m=>m.verification[0].state='tested',m=>delete m.replaces]){const m=structuredClone(manifest);mutate(m);assert.equal(valid(m),false,'negative schema case accepted');}
assert.equal(manifest.id,'com.tezbar.native.oklch-color-converter');assert.deepEqual(manifest.replaces,['raycast.oklch-color-converter']);assert.deepEqual(manifest.commands.map(c=>c.name),['convert-color']);assert.deepEqual(manifest.verification.map(v=>[v.platform,v.state]),[['macOS','not-run'],['Windows','not-run']]);
const icon=fs.readFileSync(path.join(root,'assets/icon.png'));assert.equal(icon.length,202561);assert.equal(crypto.createHash('sha256').update(icon).digest('hex'),'8bcde32c4dc7b94a8e119d3576153f34aee95bd416f937b2b93c8b56471af1e9');assert(icon.equals(fs.readFileSync(path.join(root,'fixtures/upstream/icon.png'))));
console.log('Full pinned beta schema (Ajv 8.17.1 / 2020-12), ten negative cases, manifest invariants and immutable icon: PASS');
