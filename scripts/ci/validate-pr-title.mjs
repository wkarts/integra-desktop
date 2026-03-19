import fs from 'node:fs';

const pattern = /^(build|chore|ci|docs|feat|fix|perf|refactor|revert|style|test)(\([^)]+\))?!?:\s.+$/;

function resolveTitle() {
  const explicitTitle = process.env.PR_TITLE?.trim();
  if (explicitTitle) {
    return explicitTitle;
  }

  const eventPath = process.env.GITHUB_EVENT_PATH;
  if (!eventPath || !fs.existsSync(eventPath)) {
    return '';
  }

  const payload = JSON.parse(fs.readFileSync(eventPath, 'utf8'));
  return payload.pull_request?.title?.trim() ?? '';
}

const title = resolveTitle();

if (!title) {
  console.log('Sem payload de Pull Request e sem PR_TITLE. Validação ignorada.');
  process.exit(0);
}

if (!pattern.test(title)) {
  console.error(`Título do PR inválido: "${title}"`);
  console.error('Use Conventional Commits. Exemplos válidos:');
  console.error('- feat(nfse): adiciona parser de ubaira');
  console.error('- fix(core): corrige exportação de layout prosoft');
  console.error('- docs(ci): documenta validação de título de PR');
  process.exit(1);
}

console.log(`Título do PR válido: ${title}`);
