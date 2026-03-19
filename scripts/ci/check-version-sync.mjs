import fs from 'node:fs';
import path from 'node:path';

const root = process.cwd();
const versionFile = fs.readFileSync(path.join(root, 'VERSION'), 'utf8').trim();
const packageJson = JSON.parse(fs.readFileSync(path.join(root, 'package.json'), 'utf8'));
const tauriConf = JSON.parse(fs.readFileSync(path.join(root, 'src-tauri', 'tauri.conf.json'), 'utf8'));
const cargoToml = fs.readFileSync(path.join(root, 'src-tauri', 'Cargo.toml'), 'utf8');
const cargoMatch = cargoToml.match(/\[package\][\s\S]*?^version\s*=\s*"([^"]+)"/m);

if (!cargoMatch) {
  console.error('Não foi possível localizar a versão em src-tauri/Cargo.toml');
  process.exit(1);
}

const versions = {
  VERSION: versionFile,
  'package.json': packageJson.version,
  'src-tauri/Cargo.toml': cargoMatch[1],
  'src-tauri/tauri.conf.json': tauriConf.version
};

const unique = new Set(Object.values(versions));

if (unique.size !== 1) {
  console.error('Versões fora de sincronia:');
  for (const [file, version] of Object.entries(versions)) {
    console.error(`- ${file}: ${version}`);
  }
  process.exit(1);
}

console.log(`Versionamento sincronizado em ${versionFile}`);
