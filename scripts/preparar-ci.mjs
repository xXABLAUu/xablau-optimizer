#!/usr/bin/env node
// Configura o atualizador automático durante a compilação (GitHub Actions).
//
// • Endereço das atualizações: https://github.com/<usuario>/<repositorio>/releases/latest/download/latest.json
//   (preenchido sozinho com o nome do seu repositório).
// • Chave PÚBLICA: vem da variável TAURI_UPDATER_PUBKEY e é gravada no tauri.conf.json,
//   ficando embutida no programa. (Você também pode colar a chave direto no tauri.conf.json.)
// • Chave PRIVADA: só existe como segredo do GitHub. Se ela (ou a pública) faltar, o programa é
//   compilado SEM os arquivos de atualização e o log mostra um aviso amarelo.
import { readFileSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const raiz = join(dirname(fileURLToPath(import.meta.url)), '..');
const arquivo = join(raiz, 'src-tauri', 'tauri.conf.json');
const conf = JSON.parse(readFileSync(arquivo, 'utf8').replace(/^\uFEFF/, ''));

const repositorio = (process.env.GITHUB_REPOSITORY ?? '').trim();
const chavePublica = (process.env.TAURI_UPDATER_PUBKEY ?? '').trim();
const temChavePrivada = (process.env.TAURI_SIGNING_PRIVATE_KEY ?? '').trim().length > 0;

conf.plugins ??= {};
conf.plugins.updater ??= {};
const atualizador = conf.plugins.updater;

if (/^[\w.-]+\/[\w.-]+$/.test(repositorio)) {
  atualizador.endpoints = [`https://github.com/${repositorio}/releases/latest/download/latest.json`];
  console.log(`• Atualizações em: ${atualizador.endpoints[0]}`);
}
if (chavePublica) atualizador.pubkey = chavePublica;

const publicaValida = typeof atualizador.pubkey === 'string' && atualizador.pubkey.length > 40 && !/COLE_|SUA_CHAVE/.test(atualizador.pubkey);

if (temChavePrivada && publicaValida) {
  conf.bundle.createUpdaterArtifacts = true;
  console.log('✔ Atualização automática LIGADA (instaladores assinados + latest.json).');
} else {
  conf.bundle.createUpdaterArtifacts = false;
  const faltando = [
    !temChavePrivada && 'segredo TAURI_SIGNING_PRIVATE_KEY',
    !publicaValida && 'variável TAURI_UPDATER_PUBKEY',
  ].filter(Boolean).join(' e ');
  const aviso = `Faltando ${faltando}. O programa será compilado SEM atualização automática. Veja o README, seção "Chaves do atualizador".`;
  console.log(`::warning title=Atualização automática desligada::${aviso}`);
  console.log(`⚠ ${aviso}`);
}

writeFileSync(arquivo, `${JSON.stringify(conf, null, 2)}\n`);
