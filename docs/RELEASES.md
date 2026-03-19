# Fluxo de release

## 1. Desenvolvimento

Use Conventional Commits:

- `feat(nfse): adiciona regra para zerar aliquota`
- `fix(export): corrige valor do iss retido`
- `chore(ci): ajusta workflow de release`

## 2. Pull Request

A CI valida:

- título do PR
- sincronismo de versão
- TypeScript
- build web
- `cargo fmt`
- `cargo clippy`
- `cargo test`

## 3. Merge em `main`

O workflow de release executa:

1. validações
2. `semantic-release`
3. sincronismo de `VERSION`, `package.json`, `Cargo.toml` e `tauri.conf.json`
4. criação da tag SemVer
5. build dos binários Tauri
6. publicação dos assets em GitHub Releases
