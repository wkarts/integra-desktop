import fs from 'node:fs';
import path from 'node:path';

const version = process.argv[2];
if (!version) {
  console.error('Uso: node scripts/release/prepare-release.mjs <version>');
  process.exit(1);
}

const root = process.cwd();

function writeJson(filePath, data) {
  fs.writeFileSync(filePath, `${JSON.stringify(data, null, 2)}\n`);
}

function updatePackageJson() {
  const filePath = path.join(root, 'package.json');
  const data = JSON.parse(fs.readFileSync(filePath, 'utf8'));
  data.version = version;
  writeJson(filePath, data);
}

function updateTauriConf() {
  const filePath = path.join(root, 'src-tauri', 'tauri.conf.json');
  const data = JSON.parse(fs.readFileSync(filePath, 'utf8'));
  data.version = version;
  writeJson(filePath, data);
}

function updateCargoToml() {
  const filePath = path.join(root, 'src-tauri', 'Cargo.toml');
  const content = fs.readFileSync(filePath, 'utf8');
  const updated = content.replace(/(\[package\][\s\S]*?^version\s*=\s*")([^"]+)(")/m, `$1${version}$3`);
  fs.writeFileSync(filePath, updated);
}

fs.writeFileSync(path.join(root, 'VERSION'), `${version}\n`);
updatePackageJson();
updateTauriConf();
updateCargoToml();
console.log(`Versão atualizada para ${version}`);
