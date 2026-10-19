#!/usr/bin/env node
// Xablau Optimizer — a versão do programa mora em UM lugar só: o "version" do package.json.
//
//   node scripts/versao.mjs                         → copia a versão para tauri.conf.json e Cargo.toml
//   node scripts/versao.mjs --conferir [--tag vX]   → confere se está tudo sincronizado (usado no CI)
//   node scripts/versao.mjs --notas                 → gera as notas da Release a partir do changelog.json
//
// Regras de versão (correção → 1.0.1, função nova → 1.1.0, mudança grande → 2.0.0): REGRAS_PARA_IA.md
import { appendFileSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const raiz = join(dirname(fileURLToPath(import.meta.url)), '..');
const caminho = (p) => join(raiz, p);
const lerTexto = (p) => readFileSync(caminho(p), 'utf8').replace(/^\uFEFF/, '');
const lerJson = (p) => JSON.parse(lerTexto(p));
const SEMVER = /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$/;
// A chave "version" precisa ser a PRIMEIRA linha de [workspace.package] no Cargo.toml.
const RE_CARGO = /(\[workspace\.package\]\s*\r?\nversion\s*=\s*")([^"]+)(")/;

function falhar(mensagem) {
  console.error(`\n✖ ${mensagem}\n`);
  if (process.env.GITHUB_ACTIONS) console.log(`::error title=Versão::${mensagem}`);
  process.exit(1);
}

const argumentos = process.argv.slice(2);
const opcao = (nome) => {
  const i = argumentos.indexOf(nome);
  return i >= 0 ? argumentos[i + 1] : undefined;
};

const versao = lerJson('package.json').version;
if (!SEMVER.test(versao ?? '')) {
  falhar(`A versão "${versao}" do package.json precisa ter três números, por exemplo 1.0.0.`);
}

function sincronizar() {
  const confArq = 'src-tauri/tauri.conf.json';
  const conf = lerJson(confArq);
  if (conf.version !== versao) {
    conf.version = versao;
    writeFileSync(caminho(confArq), `${JSON.stringify(conf, null, 2)}\n`);
    console.log(`• tauri.conf.json → ${versao}`);
  }
  const cargoArq = 'src-tauri/Cargo.toml';
  const cargo = lerTexto(cargoArq);
  if (!RE_CARGO.test(cargo)) {
    falhar('Não encontrei version = "..." logo abaixo de [workspace.package] em src-tauri/Cargo.toml.');
  }
  const novo = cargo.replace(RE_CARGO, `$1${versao}$3`);
  if (novo !== cargo) {
    writeFileSync(caminho(cargoArq), novo);
    console.log(`• Cargo.toml → ${versao}`);
  }
  console.log(`✔ Versão ${versao} sincronizada.`);
}

function conferir(tag) {
  const conf = lerJson('src-tauri/tauri.conf.json');
  if (conf.version !== versao) {
    falhar(`tauri.conf.json está com ${conf.version}, mas o package.json diz ${versao}. Rode: npm run versao`);
  }
  const m = lerTexto('src-tauri/Cargo.toml').match(RE_CARGO);
  if (!m || m[2] !== versao) {
    falhar(`src-tauri/Cargo.toml não está com a versão ${versao}. Rode: npm run versao`);
  }
  const historico = lerJson('changelog.json');
  const primeira = historico.versoes?.[0];
  if (!primeira || primeira.versao !== versao) {
    falhar(`changelog.json: a primeira entrada precisa ser a versão ${versao} (hoje é ${primeira?.versao ?? 'nenhuma'}).`);
  }
  if (!lerTexto('CHANGELOG.md').includes(`## [${versao}]`)) {
    falhar(`CHANGELOG.md precisa ter uma seção "## [${versao}]".`);
  }
  if (tag) {
    const limpa = tag.replace(/^refs\/tags\//, '');
    if (limpa !== `v${versao}`) {
      falhar(`A tag "${limpa}" não bate com a versão do programa (v${versao}). Crie a tag v${versao} ou corrija o package.json.`);
    }
  }
  console.log(`✔ Versão ${versao} conferida${tag ? ` com a tag ${tag}` : ''}.`);
}

function notas() {
  const entrada = lerJson('changelog.json').versoes?.find((v) => v.versao === versao);
  let texto = '';
  if (entrada) {
    texto += `## ${entrada.titulo}\n\n`;
    for (const d of entrada.destaques ?? []) texto += `- ${d}\n`;
    for (const grupo of entrada.novidades ?? []) {
      texto += `\n### ${grupo.area}\n`;
      for (const item of grupo.itens ?? []) texto += `- ${item}\n`;
    }
    if ((entrada.correcoes ?? []).length > 0) {
      texto += '\n### Correções\n';
      for (const c of entrada.correcoes) texto += `- ${c}\n`;
    }
  }
  texto +=
    '\n---\n' +
    '**Qual arquivo baixar?**\n' +
    '- `..._x64-setup.exe` → instalador recomendado (com atualização automática).\n' +
    '- `..._x64_pt-BR.msi` → instalador alternativo (para quem prefere MSI).\n' +
    '- `XablauOptimizer-...-portatil.zip` → versão portátil (pendrive), com WebView2 embutido.\n\n' +
    'O Windows pode mostrar o aviso do SmartScreen ("O Windows protegeu o computador") porque o ' +
    'programa ainda não tem certificado pago de assinatura de código. Clique em **Mais informações → ' +
    'Executar assim mesmo**. Confira os arquivos com o `SHA256SUMS.txt`.\n';
  const destino = process.env.GITHUB_OUTPUT;
  if (destino) {
    appendFileSync(destino, `notas<<XABLAU_FIM_NOTAS\n${texto}\nXABLAU_FIM_NOTAS\n`);
    console.log('✔ Notas da versão preparadas.');
  } else {
    console.log(texto);
  }
}

if (argumentos.includes('--conferir')) conferir(opcao('--tag'));
else if (argumentos.includes('--notas')) notas();
else sincronizar();
