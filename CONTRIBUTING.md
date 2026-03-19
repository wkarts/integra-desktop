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

O release macOS agora gera binário **sem exigir** secrets da Apple (certificado/notarização), ideal para distribuição direta fora da App Store.

Secrets de assinatura/notarização Apple permanecem opcionais e não são obrigatórios para publicar assets da release.

O workflow também publica com `includeUpdaterJson: false`, evitando dependência de chave de assinatura de updater quando a distribuição for apenas por binário.

Se aparecer erro antigo sobre `Unrecognized named-value: 'matrix'` no `if` do job, atualize a branch com a versão mais recente do `release.yml` (o publish foi movido para `if` no step, sem uso de `matrix` no nível de job).
