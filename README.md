# Integra Desktop

Aplicação desktop em **Tauri + Rust + React/TypeScript** para conversão local de **XML NFS-e / NFe → TXT Prosoft**, mantendo o **HTML legado** como fallback operacional.

Esta versão já está preparada para **GitHub**, com **CI**, **CD**, **versionamento automático**, **release semântica**, **validação de código**, **publicação de binários** e **documentação em GitHub Pages**.

## O que o repositório entrega

- frontend modular em `src/`
- backend Tauri/Rust em `src-tauri/`
- fallback legado isolado em `src/assets/legacy/`
- exportação TXT/CSV
- regras parametrizáveis por campo na conversão NFS-e
- pipeline GitHub pronta para:
  - validar PR e push
  - validar título de PR com Conventional Commits
  - validar TypeScript e build web
  - validar `cargo fmt`, `clippy` e `cargo test`
  - manter versionamento sincronizado
  - gerar release SemVer com `semantic-release`
  - publicar binários Tauri em GitHub Releases
  - publicar documentação no GitHub Pages

## Estrutura principal

```text
integra-desktop/
  .github/
    workflows/
      ci.yml
      release.yml
      pages.yml
  docs/
  scripts/
    ci/
    release/
  src/
    app/
    modules/
    shared/
  src-tauri/
    src/
      commands/
      core/
      storage/
```

## Scripts úteis

```bash
npm install
npm run typecheck
npm run build:web
npm run ci:version
npm run lint:rust
npm run test:rust
npm run release:dry
npm run tauri dev
npm run tauri build
```

## Fluxo de versionamento e release

### PR

A pipeline `CI` valida:

- título do PR em Conventional Commits
- sincronismo de versão entre arquivos de manifesto
- TypeScript
- build web
- `cargo fmt`
- `cargo clippy`
- `cargo test`

### Merge em `main`

A pipeline `Release`:

1. revalida frontend e Rust
2. executa `semantic-release`
3. gera tag `vX.Y.Z`
4. sincroniza versões em:
   - `VERSION`
   - `package.json`
   - `src-tauri/Cargo.toml`
   - `src-tauri/tauri.conf.json`
5. cria commit `chore(release): X.Y.Z [skip ci]`
6. publica os binários Tauri no GitHub Releases

## Secrets esperados no GitHub

### Obrigatórios

- `GITHUB_TOKEN` (automático no Actions)

### Recomendados para assinatura/updater Tauri

- `TAURI_SIGNING_PRIVATE_KEY`
- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`

### Opcionais para Windows

- `WINDOWS_CERTIFICATE`
- `WINDOWS_CERTIFICATE_PASSWORD`

### Opcionais para macOS

- `APPLE_CERTIFICATE`
- `APPLE_CERTIFICATE_PASSWORD`
- `APPLE_ID`
- `APPLE_PASSWORD`
- `APPLE_TEAM_ID`
- `APPLE_API_KEY`
- `APPLE_API_ISSUER`
- `KEYCHAIN_PASSWORD`

## Bootstrap rápido do repositório

1. Suba este projeto para um repositório GitHub.
2. Configure as branch protections da `main` exigindo a workflow `CI`.
3. Adicione os secrets necessários.
4. Ative GitHub Pages para a workflow `Pages`.
5. Faça merge usando Conventional Commits.
6. A primeira release será publicada automaticamente quando houver commit compatível.

## Documentação operacional

- `docs/RELEASES.md`
- `docs/SECRETS.md`
- `CONTRIBUTING.md`
- `SECURITY.md`
