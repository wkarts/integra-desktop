## O que foi alterado

-

## Formato obrigatório do título do PR

Use **Conventional Commits** no título:

```text
<tipo>(<escopo opcional>)?: <descrição>
```

Exemplos válidos:

- `feat(nfse): adiciona parser de ubaira`
- `fix(core): corrige exportação de layout prosoft`
- `docs(ci): documenta validação de título de PR`

Compatibilidade temporária (legado):

- `Core: adiciona parsers e mappers`

> Para evitar erros de CI, prefira sempre o formato Conventional Commits.

## Checklist

- [ ] Mantive compatibilidade com o fallback legado
- [ ] Validei build web
- [ ] Validei regras de exportação Prosoft impactadas
- [ ] Atualizei documentação quando necessário
- [ ] O título do PR segue Conventional Commits
