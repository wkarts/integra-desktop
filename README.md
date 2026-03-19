# Integra Desktop

Aplicação desktop em **Tauri + Rust + React/TypeScript** para conversão local de **XML NFS-e / NFe / SPED** em layouts Prosoft, mantendo o **HTML legado** como fallback operacional isolado.

## O que o repositório entrega

- frontend modular em `src/` com módulos Dashboard, NFS-e, NFe/Faturas, Configurações, Logs e Legado.
- backend Tauri/Rust em `src-tauri/` com commands para processamento, exportação, logs e perfis.
- fallback legado isolado em `src/assets/legacy/` (sem acoplamento no motor novo).
- exportação TXT/CSV com layout selecionável (`ba_prestados`, `ba_tomados`, `prosoft_faturas`).
- regras parametrizáveis por campo (source, zero, empty, ignore, constant), persistidas por perfil.
- parsers utilitários para NFe, SPED, pipe legado e descoberta de XML/ZIP.
- normalizadores e mapeadores com testes unitários em Rust.
- pipeline GitHub para CI/CD, release semântica e publicação Tauri.

## Estrutura principal

```text
integra-desktop/
  .github/workflows/
    ci.yml
    release.yml
    pages.yml
  src/
    app/
    modules/
      dashboard/
      nfe-faturas/
      nfse-servicos/
      legado/
      settings/
      logs/
    shared/
  src-tauri/src/
    commands/
    core/
      domain/
      parsers/
      normalizers/
      mappers/
      exporters/
      validation/
    storage/
```

## Scripts úteis

```bash
npm install --legacy-peer-deps
npm run typecheck
npm run build:web
npm run ci:version
cargo fmt --manifest-path src-tauri/Cargo.toml --all --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --all-targets --all-features
npm run tauri dev
npm run tauri build
```

## Dependências de build Linux (Tauri)

```bash
sudo apt-get update
sudo apt-get install -y libglib2.0-dev libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf
```

## Fluxo CI/CD

### Pull Request

A workflow `CI` valida:

- título do PR com Conventional Commits.
- sincronismo de versão (`VERSION`, `package.json`, `Cargo.toml`, `tauri.conf.json`).
- typecheck/build web.
- `cargo fmt`, `cargo clippy`, `cargo test`.

### Merge na `main`

A workflow `Release`:

1. revalida frontend e Rust.
2. executa `semantic-release`.
3. gera tag `vX.Y.Z` e atualiza changelog.
4. publica binários Tauri por plataforma.

## Secrets esperados no GitHub

Obrigatório:
- `GITHUB_TOKEN`.

Recomendados para assinatura/updater:
- `TAURI_SIGNING_PRIVATE_KEY`
- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`

Opcional Windows:
- `WINDOWS_CERTIFICATE`
- `WINDOWS_CERTIFICATE_PASSWORD`

Opcional macOS:
- `APPLE_CERTIFICATE`
- `APPLE_CERTIFICATE_PASSWORD`
- `APPLE_ID`
- `APPLE_PASSWORD`
- `APPLE_TEAM_ID`
- `APPLE_API_KEY`
- `APPLE_API_ISSUER`
- `KEYCHAIN_PASSWORD`

## Documentação complementar

- `docs/RELEASES.md`
- `docs/SECRETS.md`
- `CONTRIBUTING.md`
- `SECURITY.md`
