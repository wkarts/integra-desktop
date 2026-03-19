export default {
  branches: ['main'],
  tagFormat: 'v${version}',
  plugins: [
    '@semantic-release/commit-analyzer',
    '@semantic-release/release-notes-generator',
    ['@semantic-release/changelog', { changelogFile: 'CHANGELOG.md' }],
    ['@semantic-release/exec', {
      prepareCmd: 'node ./scripts/release/prepare-release.mjs ${nextRelease.version}'
    }],
    ['@semantic-release/git', {
      assets: ['VERSION', 'package.json', 'src-tauri/Cargo.toml', 'src-tauri/tauri.conf.json', 'CHANGELOG.md'],
      message: 'chore(release): ${nextRelease.version} [skip ci]\n\n${nextRelease.notes}'
    }]
  ]
};
