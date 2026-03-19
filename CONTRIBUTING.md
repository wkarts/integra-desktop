# Contribuição

## Fluxo recomendado

1. Crie uma branch a partir de `main`.
2. Faça commits seguindo Conventional Commits.
3. Abra um Pull Request com título no formato:
   - `feat: ...`
   - `fix: ...`
   - `chore: ...`
   - `docs: ...`
4. Aguarde a CI validar TypeScript, Rust, build e sincronismo de versão.

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
