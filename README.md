# Xablau Optimizer

Otimizador **seguro e reversível** para Windows 10 e 11 (foco no 11), feito para quem trabalha
(edição de vídeo no CapCut, Chrome) e joga. Criado por **Xablau Studios — Luiz**.

> **Prioridade absoluta:** nunca quebrar o sistema, nunca corromper arquivos, tudo reversível.
> Antes de qualquer alteração o programa cria um ponto de restauração, faz backup só das chaves que
> vai mudar e grava cada passo num diário. Se o PC desligar no meio, ele oferece desfazer na próxima vez.

---

## Índice

1. [O que o programa faz](#1-o-que-o-programa-faz)
2. [Para quem vai usar: baixar e instalar](#2-para-quem-vai-usar-baixar-e-instalar)
3. [Para o criador: publicar o programa no GitHub (passo a passo)](#3-para-o-criador-publicar-o-programa-no-github-passo-a-passo)
4. [Publicar uma versão nova](#4-publicar-uma-versão-nova)
5. [Chaves do atualizador: como funcionam e cuidados](#5-chaves-do-atualizador-como-funcionam-e-cuidados)
6. [O que o programa faz e o que ele nunca faz](#6-o-que-o-programa-faz-e-o-que-ele-nunca-faz)
7. [Estrutura do projeto](#7-estrutura-do-projeto)
8. [Compilar no próprio PC (opcional)](#8-compilar-no-próprio-pc-opcional)
9. [Solução de problemas](#9-solução-de-problemas)
10. [Créditos e licença](#10-créditos-e-licença)

---

## 1. O que o programa faz

O Xablau Optimizer tem **7 abas** e uma **busca global** (atalho `Ctrl+K`): digite "CapCut",
"Copilot", "drivers"… e vá direto para a função.

| Aba | Para que serve |
| --- | --- |
| **Início** | Medidor de saúde do sistema, **Otimizar com um clique**, **Modo Turbo**, temperaturas e "O que há de novo". |
| **Diagnóstico** | Só analisa, não muda nada: BIOS (XMP/DOCP, Resizable BAR), saúde dos SSDs, travamentos explicados em português, culpados por lentidão, histórico, vazamento de memória, FPS antes/depois e ficha do PC. |
| **Desempenho** | Energia, núcleos (CCDs do Ryzen), perfis por aplicativo (tira o Modo de Eficiência do CapCut sozinho), Modo Jogo, IdleSaver, liberação de memória (botão manual) e ajustes NVIDIA guiados. |
| **Limpeza** | Limpeza segura, analisador de espaço, duplicados, privacidade com semáforo, apps pré-instalados e arquivo hosts. |
| **Sistema** | Serviços, inicialização (com atraso), recursos do Windows, drivers, winget, desinstalador completo, Windows Update, rede, firewall por app e reparos. |
| **Segurança** | Pontos de restauração, desfazer tudo ou item a item, histórico e recuperação de operação interrompida. |
| **Configurações** | 10 temas, fontes, iniciar com o Windows, bandeja, manutenção no ocioso, perfis, modo portátil e Sobre. |

Há dois modos, com uma chave de troca sempre visível:

- **Modo Básico** — para leigos: poucas abas, otimização automática só com ajustes seguros e explicação simples em cartões grandes.
- **Modo Avançado** — todas as abas e funções, incluindo o perfil Agressivo e as regras "se… então".

Cada função tem um **selo de risco** (🟢 seguro, 🟡 atenção, 🔴 avançado) e um botão **"Saiba mais"**
com o que faz, como funciona, o benefício e o risco.

---

## 2. Para quem vai usar: baixar e instalar

### 2.1 Qual arquivo baixar

Na página do projeto no GitHub, abra a aba **Releases** (à direita) e, na versão mais recente, baixe:

| Arquivo | Quando usar |
| --- | --- |
| `Xablau Optimizer_X.Y.Z_x64-setup.exe` | **Recomendado.** Instalador normal, com atualização automática. |
| `Xablau Optimizer_X.Y.Z_x64_pt-BR.msi` | Alternativa para quem prefere instalador MSI. |
| `XablauOptimizer-X.Y.Z-portatil.zip` | Versão portátil (pendrive), com o WebView2 embutido. Não se atualiza sozinha. |
| `SHA256SUMS.txt` | Códigos para conferir se o arquivo baixado é o original (opcional). |

### 2.2 Instalar

1. Dê dois cliques no `..._x64-setup.exe`.
2. Se aparecer a tela azul **"O Windows protegeu o computador"**, clique em **Mais informações** e depois em **Executar assim mesmo** (explicação abaixo).
3. Aceite o pedido de administrador e siga o instalador.
4. Ao abrir o programa pela primeira vez, o **assistente** pergunta seu nível (Básico ou Avançado) e o uso principal (jogos, edição/trabalho ou uso geral) e sugere um perfil pronto.

### 2.3 Por que aparece o aviso do SmartScreen?

O SmartScreen avisa sobre programas que ainda não têm um **certificado pago de assinatura de código**
(custa caro por ano e está fora do escopo por enquanto). O aviso **não** significa que o programa tem vírus:
significa apenas que a Microsoft ainda não conhece aquele arquivo. Cada versão nova pode mostrar o aviso
de novo.

Quer conferir se o arquivo é o original? Abra o PowerShell na pasta do download e rode:

```powershell
Get-FileHash ".\Xablau Optimizer_1.0.0_x64-setup.exe" -Algorithm SHA256
```

O código mostrado precisa ser igual ao que está no `SHA256SUMS.txt` da mesma Release.

### 2.4 Antivírus (ex.: Kaspersky)

O Xablau Optimizer mexe em configurações do sistema e usa PowerShell, então alguns antivírus podem
pedir confirmação. O programa **nunca** desativa nem dribla o seu antivírus. Se o antivírus bloquear:

1. Confirme que você baixou o arquivo da página oficial do projeto (e confira o SHA-256).
2. No **Kaspersky**: *Configurações → Ameaças e exclusões → Especificar aplicativos confiáveis → Adicionar* e escolha `C:\Program Files\Xablau Optimizer\xablau-optimizer.exe` e `xablau-vigia.exe`.
3. Nunca desligue o antivírus inteiro.

### 2.5 Se algo der errado

- **Pelo programa:** aba **Segurança → Desfazer tudo** (ou item a item).
- **Kit de emergência:** no Menu Iniciar, abra **"Xablau - Restaurar tudo"**. Ele funciona sem a interface e até no Modo de Segurança.
- **Restauração do Sistema do Windows:** o kit também abre o `rstrui` com os pontos criados pelo Xablau.
- **Modo de Segurança:** *Configurações → Sistema → Recuperação → Inicialização avançada → Reiniciar agora → Solução de problemas → Opções avançadas → Configurações de Inicialização → Reiniciar →* tecla **4**. Depois abra o kit em `C:\Program Files\Xablau Optimizer\xablau-restaurar.exe` (na versão portátil: `C:\ProgramData\XablauOptimizer\kit\xablau-restaurar.exe`).

Os backups e o diário ficam em `C:\ProgramData\XablauOptimizer` e **continuam lá mesmo se você desinstalar
o programa**, para ainda ser possível desfazer.

---

## 3. Para o criador: publicar o programa no GitHub (passo a passo)

Você **não precisa programar nada**. O GitHub compila o programa sozinho (de graça em repositórios
públicos) e publica o instalador na aba Releases.

### 3.1 Criar a conta no GitHub

1. Acesse **https://github.com** e clique em **Sign up**.
2. Informe e-mail, senha e nome de usuário, confirme o código enviado por e-mail.

### 3.2 Criar o repositório

1. No canto superior direito, clique em **+ → New repository**.
2. Em **Repository name**, escreva `xablau-optimizer`.
3. Marque **Public** (público).
   > ⚠️ Precisa ser **público**: o atualizador automático baixa as versões novas direto da aba
   > Releases, e em repositório privado os downloads exigem login (as atualizações falhariam).
4. **Não** marque "Add a README" (o projeto já tem um).
5. Clique em **Create repository**.

### 3.3 Gerar as chaves do atualizador (faça antes da primeira versão)

As chaves garantem que o programa só aceite atualizações feitas por você
(detalhes na [seção 5](#5-chaves-do-atualizador-como-funcionam-e-cuidados)). Isso é feito **uma vez só**.

1. Instale o **Node.js LTS** em **https://nodejs.org** (botão "LTS", instalação padrão: Avançar → Avançar → Concluir).
2. Abra o **PowerShell** (Menu Iniciar → digite *PowerShell* → Enter) e rode:

   ```powershell
   npx --yes @tauri-apps/cli@2.12.3 signer generate -w "$env:USERPROFILE\.tauri\xablau-optimizer.key"
   ```

3. Ele pede uma **senha** duas vezes. Escolha uma senha forte e **anote** (gerenciador de senhas).
4. Foram criados dois arquivos na pasta `C:\Users\SEU_USUARIO\.tauri\`:
   - `xablau-optimizer.key` → **chave PRIVADA** (o "carimbo"). **Segredo absoluto.**
   - `xablau-optimizer.key.pub` → **chave PÚBLICA** (a "foto do carimbo"). Pode ser pública.
5. **Guarde uma cópia** dos dois arquivos e da senha em local seguro **fora do repositório**
   (gerenciador de senhas ou pendrive guardado). Se perder a chave privada, quem já instalou o
   programa não recebe mais atualizações automáticas.

### 3.4 Cadastrar as chaves no GitHub

No repositório: **Settings → Secrets and variables → Actions**.

Na aba **Secrets**, clique em **New repository secret** duas vezes:

| Name | Secret (valor) |
| --- | --- |
| `TAURI_SIGNING_PRIVATE_KEY` | Todo o conteúdo do arquivo `xablau-optimizer.key` |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | A senha que você escolheu |

Na aba **Variables**, clique em **New repository variable**:

| Name | Value |
| --- | --- |
| `TAURI_UPDATER_PUBKEY` | Todo o conteúdo do arquivo `xablau-optimizer.key.pub` |

Para ver o conteúdo de um arquivo e copiar: no PowerShell, rode
`notepad "$env:USERPROFILE\.tauri\xablau-optimizer.key"` (troque por `.key.pub` para a pública),
aperte `Ctrl+A`, `Ctrl+C` e cole no GitHub.

> A chave pública é colocada no `tauri.conf.json` e embutida no programa automaticamente durante a
> compilação. Se preferir, você também pode colá-la direto no campo `plugins.updater.pubkey` do
> arquivo `src-tauri/tauri.conf.json`.

### 3.5 Subir os arquivos do projeto

**Jeito recomendado — GitHub Desktop** (mais fácil e sobe tudo de uma vez):

1. Baixe e instale em **https://desktop.github.com** e entre com sua conta.
2. **File → Clone repository**, escolha `xablau-optimizer` e uma pasta no PC → **Clone**.
3. Copie **todo o conteúdo** do projeto (inclusive a pasta oculta `.github`) para dentro dessa pasta.
4. No GitHub Desktop, escreva um resumo (ex.: *Primeira versão*) e clique em **Commit to main**.
5. Clique em **Push origin**.

**Alternativa — pelo site:** na página do repositório, clique em **Add file → Upload files** e arraste
as pastas. O site aceita no máximo **100 arquivos por envio**: suba em partes (por exemplo `src`,
depois `src-tauri`, depois o resto) e confira se a pasta `.github/workflows` foi enviada.

### 3.6 Criar a tag `v1.0.0` (é ela que dispara a compilação)

O número da tag precisa ser **igual** à versão do `package.json` (hoje, `1.0.0`).

**Pelo site (mais simples):**

1. Na página do repositório, clique em **Releases → Draft a new release** (ou *Create a new release*).
2. Em **Choose a tag**, digite `v1.0.0` e clique em **Create new tag: v1.0.0 on publish**.
3. Em **Release title**, escreva `Xablau Optimizer v1.0.0`.
4. Clique em **Publish release**. Os arquivos aparecem nessa Release quando a compilação terminar.

**Pelo GitHub Desktop:** aba **History** → clique com o botão direito no último commit →
**Create Tag…** → `v1.0.0` → **Create Tag** → **Push origin**.

### 3.7 Acompanhar a compilação

1. Abra a aba **Actions** do repositório. Vai aparecer **"Compilar e publicar"** rodando (bolinha amarela).
2. A primeira compilação leva de **25 a 45 minutos** (as próximas são mais rápidas, graças ao cache).
3. Ficou **verde**: pronto. Ficou **vermelho**: clique nela, abra o passo com ❌, copie a mensagem de
   erro e peça ajuda à IA (ela segue as regras do arquivo `REGRAS_PARA_IA.md`).
4. Viu um aviso **amarelo "Atualização automática desligada"**? Faltou cadastrar as chaves do passo 3.4.
   O programa funciona, mas sem atualização automática.

### 3.8 Baixar o instalador

Abra **Releases**: estarão lá o `..._x64-setup.exe`, o `.msi`, o `...-portatil.zip`, o `latest.json`
(usado pelo atualizador) e o `SHA256SUMS.txt`.

### 3.9 Testar sem publicar

Na aba **Actions**, escolha **Compilar e publicar → Run workflow → Run workflow**. Ele compila tudo e
guarda os arquivos em **Artifacts** (no fim da página da execução), sem criar Release.

---

## 4. Publicar uma versão nova

1. Peça a mudança à IA. Ela segue o `REGRAS_PARA_IA.md`: lê a versão atual, sobe o número
   (correção → `1.0.1`; função nova → `1.1.0`; mudança grande → `2.0.0`) e registra as novidades no
   `CHANGELOG.md` e no `changelog.json` (que aparece na tela "O que há de novo").
2. Suba os arquivos alterados (GitHub Desktop: **Commit to main → Push origin**).
3. Crie a tag com o **mesmo número** (ex.: `v1.0.1`), como no passo 3.6.
4. Quem tem o programa instalado recebe o aviso animado de nova versão (no Modo Básico pode atualizar
   sozinho ao fechar). A versão portátil só avisa e abre a página da Release.

> O programa **nunca** se atualiza no meio de uma operação nem com jogo aberto.

---

## 5. Chaves do atualizador: como funcionam e cuidados

O programa roda como **administrador**. Sem assinatura, uma conta do GitHub invadida ou um download
interceptado poderia instalar um vírus com poder total em todos os PCs. Com a assinatura, isso é
impossível sem a sua chave privada.

1. **Geração única:** `tauri signer generate` cria a chave **privada** (o carimbo, secreta) e a **pública** (a foto do carimbo).
2. **Chave pública:** vai para o `tauri.conf.json` (`plugins.updater.pubkey`) e fica embutida no programa.
3. **Chave privada:** existe só como segredo do GitHub Actions (`TAURI_SIGNING_PRIVATE_KEY` e `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`). Nunca no código, no repositório, em logs ou em mensagens de commit.
4. **A cada versão:** o workflow compila o instalador, gera a assinatura (`.sig`) daquele instalador exato e publica o instalador, o `.sig` e o `latest.json` (versão, notas, endereço e assinatura).
5. **No PC do usuário:** o programa lê o `latest.json`, baixa o instalador e confere a assinatura com a chave pública. Se bater, instala. Se faltar a assinatura ou **um único byte** tiver sido alterado, **recusa** a atualização e registra no log.

**Riscos que você precisa conhecer:**

- **Perder a chave privada:** as versões já instaladas só aceitam atualizações assinadas com ela. Sem a chave, nenhuma atualização chega a esses usuários, e cada um teria que reinstalar manualmente. **Guarde uma cópia** em gerenciador de senhas ou pendrive, fora do repositório.
- **Vazar a chave privada:** qualquer pessoa poderia assinar uma atualização falsa que o programa aceitaria. Se acontecer: gere um par novo (passo 3.3), troque os segredos (passo 3.4), publique uma versão nova e avise os usuários para **reinstalarem manualmente** essa versão.
- **Não confunda com o certificado de assinatura de código (Authenticode):** a assinatura do atualizador é gratuita e serve só para o atualizador. Ela **não remove** o aviso do SmartScreen na primeira instalação — isso exige um certificado pago (opcional, fora do escopo por enquanto).

---

## 6. O que o programa faz e o que ele nunca faz

**Sempre:**

- Cria um **ponto de restauração** antes de qualquer alteração (liga a Proteção do Sistema se estiver desligada, contorna o limite de 24 h e confere se o ponto foi criado). Se falhar, **não aplica nada**.
- Faz **backup só das chaves do registro que vai alterar** e restaura só essas chaves.
- Grava cada passo no **diário antes de executar**; detecta operação interrompida e oferece desfazer.
- Confere antes: reinicialização pendente, Windows Update instalando, espaço em disco, versão do Windows, notebook ou desktop e bateria.
- Mostra a **simulação** (antes → depois) e só aplica com a sua aprovação.
- Prefere **"Manual" a "Desativado"** nos serviços, guardando o estado original.
- Mantém um **log** com data e hora de cada alteração, que você pode exportar.
- **Avisa** quando uma atualização do Windows desfaz algum ajuste (sem reaplicar nada pesado sozinho).

**Nunca:**

- Remove ou desativa a Microsoft Store, o App Installer (winget), Visual C++, .NET, **WebView2**, o Windows Update, áudio, rede, o instalador do Windows, criptografia ou o seu antivírus.
- Apaga WinSxS ou Windows\Installer na mão (usa o DISM) nem limpa o Prefetch.
- Mexe no Defender quando outro antivírus está ativo, ou tenta driblar a autoproteção de um antivírus.
- Baixa scripts ou "bancos de ajustes" da internet: tudo vem embutido no programa e só muda por atualização assinada.
- Desliga mitigações do processador, força modo MSI na GPU, "limpa" o registro, desfragmenta SSD, mexe em timer/`bcdedit` ou desliga a Integridade de Memória sozinho (só informa).
- Injeta código em jogos, lê memória de jogos ou altera arquivos de configuração de jogos (respeita anti-cheats como Vanguard, EasyAntiCheat e BattlEye).
- Inclui nome de usuário, nome do PC, números de série, chave do Windows, IP ou MAC na "Ficha do PC".

---

## 7. Estrutura do projeto

```
xablau-optimizer/
├── .github/workflows/build.yml   ← compila e publica no GitHub (tag v*)
├── scripts/                      ← versão, preparação da compilação e versão portátil
├── src/                          ← interface (React + TypeScript + Tailwind + Framer Motion)
├── src-tauri/
│   ├── src/                      ← backend da interface (comandos Tauri)
│   ├── catalogo/ajustes.json     ← catálogo de ajustes (textos, riscos, ações)
│   ├── powershell/               ← scripts PowerShell 5.1 embutidos
│   ├── crates/core/              ← núcleo: diário, backups, desfazer, processos, energia
│   ├── crates/vigia/             ← vigia em segundo plano (bandeja)
│   ├── crates/restaurar/         ← kit de emergência "Xablau — Restaurar tudo"
│   ├── icons/                    ← ícones do programa
│   ├── licencas/                 ← licenças distribuídas junto
│   ├── windows/hooks.nsh         ← ganchos do instalador
│   └── tauri.conf.json           ← configuração do Tauri
├── CHANGELOG.md / changelog.json ← histórico de versões ("O que há de novo")
├── CREDITOS.md                   ← créditos e licenças
└── REGRAS_PARA_IA.md             ← regras para qualquer IA que for alterar o código
```

A versão do programa mora **só no `package.json`**; o comando `npm run versao` (e o próprio GitHub
Actions) copia o número para o `tauri.conf.json` e o `Cargo.toml`.

---

## 8. Compilar no próprio PC (opcional)

Só para quem quiser compilar localmente (o GitHub já faz isso sozinho):

1. Instale o **Node.js 22 LTS**, o **Rust** (https://rustup.rs) e as **Visual Studio Build Tools** com a carga *"Desenvolvimento para desktop com C++"*.
2. Na pasta do projeto, rode:

   ```powershell
   npm ci
   npm run binarios      # compila o vigia e o kit e baixa o PresentMon (SHA-256 conferido)
   npm run tauri dev     # abre o programa em modo de desenvolvimento
   npm run compilar      # gera os instaladores em src-tauri\target\release\bundle
   ```

O programa pede administrador ao abrir (é necessário para otimizar o sistema).

---

## 9. Solução de problemas

| Problema | O que fazer |
| --- | --- |
| A compilação ficou vermelha no passo **"Conferir a versão com a tag"** | A tag não bate com o `package.json` ou falta a entrada no `changelog.json`/`CHANGELOG.md`. Apague a tag/Release, corrija e crie de novo. |
| Aviso **"Atualização automática desligada"** | Faltam os segredos/variável do passo 3.4. Cadastre e publique uma versão nova. |
| A Release ficou sem arquivos | A compilação falhou. Veja a aba Actions, corrija e rode de novo (**Re-run jobs**) ou apague a Release e a tag e crie de novo. |
| **"O Windows protegeu o computador"** | Normal para programa sem certificado pago: **Mais informações → Executar assim mesmo** (seção 2.3). |
| O antivírus bloqueou | Veja a seção 2.4. Nunca desligue o antivírus inteiro. |
| A versão portátil abre com a tela em branco | Extraia o `.zip` para uma pasta comum (ex.: `C:\XablauPortatil`) em vez de abrir de dentro do `.zip`. |
| Esqueci a senha da chave do atualizador | Não tem como recuperar: gere um par novo (3.3), cadastre de novo (3.4) e avise os usuários para reinstalarem a próxima versão. |
| Algo ficou estranho depois de otimizar | Aba **Segurança → Desfazer tudo**, ou o kit **"Xablau - Restaurar tudo"** (seção 2.5). |

---

## 10. Créditos e licença

Criado por **Xablau Studios — Luiz**. Licença **MIT** (arquivo `LICENSE`).

As funções foram recriadas com código próprio, inspiradas no comportamento público de projetos de
código aberto como **WinUtil** (Chris Titus Tech), **Win11Debloat** (Raphire), **Optimizer** (hellzerg)
e projetos da **Builtbybel**. A lista completa, com links e licenças (incluindo o **PresentMon** da
Intel), está em [`CREDITOS.md`](CREDITOS.md) e na aba **Configurações → Sobre** do programa.
