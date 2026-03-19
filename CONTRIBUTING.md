# Contribuição

## Fluxo recomendado

1. Crie uma branch a partir de `main`.
2. Faça commits seguindo Conventional Commits.
3. Abra um Pull Request com título **obrigatoriamente** no formato Conventional Commits.
4. Aguarde a CI validar TypeScript, Rust, build e sincronismo de versão.

## Título de PR (obrigatório na CI)

A workflow executa `node ./scripts/ci/validate-pr-title.mjs` e aceita apenas:

```text
<tipo>(<escopo opcional>)?: <descrição>
```

Exemplos válidos:

- `feat(nfse): adiciona parser de ubaira`
- `fix(core): corrige normalização de documento`
- `chore(ci): ajusta validação de versão`

Compatibilidade temporária (para evitar falha recorrente em PR legada):

- `Core: adiciona parsers e mappers`
- `Adicionar core Rust (parsers/mappers/exporters), validação de título de PR e ajustes de build/CI`

> Esse formato legado passa na CI, mas o padrão recomendado continua sendo Conventional Commits.

Exemplo inválido (vai falhar):

- `Add Rust core processing (parsers, mappers, exporters), UI profile layouts and lockfiles`

Exemplos válidos para PRs grandes de core:

- `feat(core): adiciona processamento Rust, layouts de perfil da UI e lockfiles`
- `feat(rust): adiciona parsers, mappers, exporters e layouts de perfil da UI`

## Padrão de commit

Formatos aceitos:

- `feat: adiciona parser de município`
- `fix(nfse): corrige cálculo de iss retido`
- `chore(ci): ajusta pipeline de release`

Tipos aceitos:

- `build`
- `chore`
- `ci`
- `docs`
- `feat`
- `fix`
- `perf`
- `refactor`
- `revert`
- `style`
- `test`

## Como validar localmente o título do PR

```bash
cat > /tmp/pr-event.json <<'JSON'
{"pull_request":{"title":"feat(nfse): adiciona parser de ubaira"}}
JSON
GITHUB_EVENT_PATH=/tmp/pr-event.json node ./scripts/ci/validate-pr-title.mjs
```

## Scripts úteis

```bash
npm install
npm run typecheck
npm run build:web
npm run ci:version
cargo fmt --manifest-path src-tauri/Cargo.toml --all --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --all-targets --all-features
```

## Release macOS na CI

O job de release para macOS só executa quando os secrets de assinatura estão preenchidos:

- `TAURI_SIGNING_PRIVATE_KEY`
- `APPLE_CERTIFICATE`
- `APPLE_CERTIFICATE_PASSWORD`
- `KEYCHAIN_PASSWORD`

Sem esses secrets, a pipeline publica Linux/Windows e pula macOS para evitar falha de `codesign` por certificado vazio.
