#!/usr/bin/env node
// Prepara os executáveis auxiliares que vão junto com a interface ("externalBin" do Tauri):
//   • xablau-vigia     → vigia em segundo plano (bandeja, perfis por app, IdleSaver, Watchdog)
//   • xablau-restaurar → kit de emergência "Xablau — Restaurar tudo"
//   • presentmon       → PresentMon da Intel (MIT), versão fixa com SHA-256 conferido
// O Tauri exige que eles fiquem em src-tauri/binaries/ com o sufixo do alvo (ex.: -x86_64-pc-windows-msvc.exe).
//
//   node scripts/preparar-binarios.mjs            → compila em modo de desenvolvimento
//   node scripts/preparar-binarios.mjs --release  → compila em modo final (usado no CI)
import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { copyFileSync, existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const raiz = join(dirname(fileURLToPath(import.meta.url)), '..');
const pastaTauri = join(raiz, 'src-tauri');
const pastaBinarios = join(pastaTauri, 'binaries');
const release = process.argv.includes('--release');
const perfil = release ? 'release' : 'debug';
const PRESENTMON = {
  versao: process.env.PRESENTMON_VERSAO || '2.6.0',
  sha256: (process.env.PRESENTMON_SHA256 || 'b2a706bc6ad475749e3b7e3409263aa1e6906d45bdcf993f6dbc0f660188f1af').toLowerCase(),
};

function falhar(mensagem) {
  console.error(`\n✖ ${mensagem}\n`);
  if (process.env.GITHUB_ACTIONS) console.log(`::error title=Binários auxiliares::${mensagem}`);
  process.exit(1);
}
const sha256 = (dados) => createHash('sha256').update(dados).digest('hex');

const alvo = (() => {
  try {
    const saida = execFileSync('rustc', ['-vV'], { encoding: 'utf8' });
    const m = saida.match(/^host:\s*(\S+)/m);
    if (m) return m[1];
  } catch {
    /* tratado abaixo */
  }
  return falhar('Rust não encontrado. Instale pelo https://rustup.rs (ou deixe o GitHub Actions compilar).');
})();

mkdirSync(pastaBinarios, { recursive: true });

console.log(`• Compilando o vigia e o kit de emergência (${perfil}, ${alvo})...`);
execFileSync('cargo', ['build', ...(release ? ['--release'] : []), '-p', 'xablau-vigia', '-p', 'xablau-restaurar'], {
  cwd: pastaTauri,
  stdio: 'inherit',
});
for (const nome of ['xablau-vigia', 'xablau-restaurar']) {
  const origem = join(pastaTauri, 'target', perfil, `${nome}.exe`);
  if (!existsSync(origem)) falhar(`Não encontrei ${origem} depois da compilação.`);
  copyFileSync(origem, join(pastaBinarios, `${nome}-${alvo}.exe`));
  console.log(`  ✔ ${nome}-${alvo}.exe`);
}

const destinoPm = join(pastaBinarios, `presentmon-${alvo}.exe`);
if (existsSync(destinoPm) && sha256(readFileSync(destinoPm)) === PRESENTMON.sha256) {
  console.log('  ✔ PresentMon já estava baixado e conferido.');
} else {
  const url = `https://github.com/GameTechDev/PresentMon/releases/download/v${PRESENTMON.versao}/PresentMon-${PRESENTMON.versao}-x64.exe`;
  console.log(`• Baixando o PresentMon ${PRESENTMON.versao} (fonte oficial: GitHub da Intel)...`);
  let dados = null;
  for (let tentativa = 1; tentativa <= 3 && !dados; tentativa++) {
    try {
      const resposta = await fetch(url, { redirect: 'follow' });
      if (!resposta.ok) throw new Error(`HTTP ${resposta.status}`);
      dados = Buffer.from(await resposta.arrayBuffer());
    } catch (erro) {
      console.log(`  tentativa ${tentativa} falhou: ${erro.message}`);
      await new Promise((r) => setTimeout(r, 3000 * tentativa));
    }
  }
  if (!dados) falhar('Não foi possível baixar o PresentMon.');
  const recebido = sha256(dados);
  if (recebido !== PRESENTMON.sha256) {
    falhar(`O PresentMon baixado não confere (SHA-256 ${recebido}). Por segurança, a compilação foi interrompida.`);
  }
  writeFileSync(destinoPm, dados);
  console.log(`  ✔ presentmon-${alvo}.exe (SHA-256 conferido)`);
}
console.log('✔ Binários auxiliares prontos.');
