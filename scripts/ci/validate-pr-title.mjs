import fs from 'node:fs';

const eventPath = process.env.GITHUB_EVENT_PATH;
if (!eventPath || !fs.existsSync(eventPath)) {
  console.log('Sem payload de Pull Request. Validação ignorada.');
  process.exit(0);
}

const payload = JSON.parse(fs.readFileSync(eventPath, 'utf8'));
const title = payload.pull_request?.title ?? '';
const pattern = /^(build|chore|ci|docs|feat|fix|perf|refactor|revert|style|test)(\([^)]+\))?!?:\s.+$/;

if (!title) {
  console.error('Título do PR ausente.');
  process.exit(1);
}

if (!pattern.test(title)) {
  console.error(`Título do PR inválido: "${title}"`);
  console.error('Use Conventional Commits, por exemplo: feat(nfse): adiciona parser de ubaira');
  process.exit(1);
}

console.log(`Título do PR válido: ${title}`);
