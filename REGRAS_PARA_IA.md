# Regras para qualquer IA (ou pessoa) que for alterar o Xablau Optimizer

Este arquivo existe para que qualquer assistente de IA continue o projeto sem quebrar nada.
**Leia tudo antes de mexer no código.**

## 1. Versão — sempre leia antes de mexer

1. A versão oficial fica **só** no `package.json` (campo `"version"`). Leia a versão atual antes de qualquer alteração.
2. Suba a versão a **cada pedido** atendido:
   - **correção de bug** → sobe o último número: 1.0.0 → 1.0.1 → 1.0.2;
   - **função nova** → sobe o do meio e zera o último: 1.0.2 → 1.1.0;
   - **mudança grande** (quebra compatibilidade, reestrutura o programa) → sobe o primeiro: 1.4.2 → 2.0.0.
3. Use **sempre três números** (o Tauri e o instalador do Windows exigem esse formato).
4. Depois de mudar o `package.json`, rode `npm run versao`: ele copia o número para `src-tauri/tauri.conf.json` e `src-tauri/Cargo.toml`. (O GitHub Actions também roda isso sozinho, mas deixe os arquivos sincronizados no repositório.)
5. Adicione a entrada nova **no topo** dos dois arquivos de histórico:
   - `CHANGELOG.md`: seção `## [X.Y.Z] — AAAA-MM-DD` com o que mudou;
   - `changelog.json`: objeto novo em primeiro lugar dentro de `"versoes"`, com `versao`, `data`, `titulo`, `destaques`, `novidades` (por área) e `correcoes`. É ele que aparece na tela "O que há de novo".
6. O workflow confere tudo isso: se a tag (ex.: `v1.0.1`) não bater com o `package.json` ou faltar a entrada no histórico, a compilação para com uma mensagem explicando o motivo.

## 2. Regras de segurança que NUNCA podem ser quebradas

- Nunca quebrar o sistema, nunca corromper arquivos, tudo reversível.
- Toda alteração no sistema passa pelo motor de ajustes: verificações prévias → simulação aprovada → ponto de restauração confirmado → diário (journal) gravado **antes** de cada passo → backup granular → aplicação → registro no log.
- Se o ponto de restauração falhar, nada é aplicado.
- Lista de proteção (nunca remover/desativar): Microsoft Store, App Installer (winget), Visual C++, .NET, WebView2, Windows Update, áudio, rede, instalador do Windows, criptografia e o antivírus instalado. Ela fica em `src-tauri/crates/core/src/protecao.rs`.
- Serviços: preferir "Manual" a "Desativado".
- Nunca apagar WinSxS ou Windows\Installer manualmente; nunca limpar o Prefetch.
- Drivers: só domínios oficiais via HTTPS, assinatura Authenticode válida, backup com `pnputil /export-driver` e um driver por vez.
- Não mexer no Defender se outro antivírus estiver ativo; nunca driblar autoproteção de antivírus.
- Anti-cheat: só APIs padrão do Windows (prioridade, CPU Sets); nunca injetar código, ler memória ou abrir processos de jogo com permissões além do necessário.
- Scripts PowerShell ficam **embutidos** no executável (pasta `src-tauri/powershell/`). Nunca baixar scripts ou "bancos de ajustes" da internet em tempo de execução.
- A interface só chama a **lista fechada** de comandos do Rust, com parâmetros validados. Nunca montar comandos a partir de texto livre digitado pelo usuário.
- A chave privada do atualizador nunca entra no código, no repositório, em logs ou em mensagens de commit.

## 3. Funções retiradas de propósito (não adicionar de volta)

Desligar mitigações do processador, forçar modo MSI na GPU, limpador de registro, atualizador genérico
de drivers, desfragmentação de SSD, desligar a Integridade de Memória sozinho (só informar), ajustes de
timer e `bcdedit`, cofre de arquivos, gerenciador de senhas, antivírus próprio, banco de ajustes baixado
da internet, sistema de login/contas e engenharia reversa de software pago.

## 4. Onde fica cada coisa

| Pasta/arquivo | O que é |
| --- | --- |
| `src/` | Interface (React + TypeScript + Tailwind + Framer Motion). Todo texto em português do Brasil. |
| `src/lib/comandos.ts` | Lista fechada (e tipada) das chamadas da interface para o Rust. |
| `src-tauri/src/` | Backend da interface (comandos Tauri e motor de ajustes). |
| `src-tauri/catalogo/ajustes.json` | Catálogo de ajustes: textos, "Saiba mais", nível de risco e ações. |
| `src-tauri/powershell/` | Scripts PowerShell 5.1 embutidos no executável. |
| `src-tauri/crates/core/` | Núcleo compartilhado: diário, backups, desfazer, processos, energia. |
| `src-tauri/crates/vigia/` | Vigia em segundo plano (bandeja, perfis por app, IdleSaver, Watchdog). |
| `src-tauri/crates/restaurar/` | Kit de emergência "Xablau — Restaurar tudo" (funciona no Modo de Segurança). |
| `scripts/` | Scripts de versão, preparação da compilação e montagem da versão portátil. |
| `.github/workflows/build.yml` | Compilação e publicação automáticas no GitHub. |

## 5. Como adicionar uma função nova

1. **Ajuste simples (registro, serviço, energia, tarefa):** acrescente ao catálogo `src-tauri/catalogo/ajustes.json` com título, descrição curta, "Saiba mais" (o que faz, como funciona, benefício, risco), selo de risco (verde/amarelo/vermelho), modo (básico/avançado), condições (Windows 11, desktop, NVIDIA…) e palavras para a busca.
2. **Comando novo do Rust:** crie a função com validação de todos os parâmetros, registre no `generate_handler!` e acrescente a chamada tipada em `src/lib/comandos.ts`.
3. **Toda função** precisa aparecer na busca global (Ctrl+K), ter explicação nos dois modos e respeitar "reduzir animações".
4. Suba a versão e atualize os históricos (seção 1).

## 6. Antes de entregar

- Rode `npm run build` (interface) e `cargo check` dentro de `src-tauri` (Rust) sem erros.
- Mantenha `package-lock.json` e `src-tauri/Cargo.lock` atualizados no repositório.
- Para testar a compilação completa sem publicar, use **Actions → Compilar e publicar → Run workflow** (gera os arquivos como artefatos, sem criar Release).
