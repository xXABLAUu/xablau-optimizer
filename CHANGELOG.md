# Histórico de versões — Xablau Optimizer

Todas as mudanças importantes do programa ficam registradas aqui. O formato segue o
[versionamento semântico](https://semver.org/lang/pt-BR/), sempre com **três números**:

- **Correção de bug** → sobe o último número (1.0.0 → 1.0.1 → 1.0.2)
- **Função nova** → sobe o número do meio (1.0.x → 1.1.0)
- **Mudança grande** → sobe o primeiro número (1.x.x → 2.0.0)

Cada versão também precisa de uma entrada no `changelog.json`, que alimenta a tela
**"O que há de novo"** dentro do programa. As regras completas estão em `REGRAS_PARA_IA.md`.

## [1.0.0] — 2026-10-18

### Lançamento inicial

Primeira versão do **Xablau Optimizer**, criado por **Xablau Studios — Luiz**, para Windows 10 e 11
(foco no 11), pensado para quem trabalha (edição de vídeo no CapCut, Chrome) e joga.

#### Segurança em primeiro lugar
- Ponto de restauração antes de qualquer alteração (liga a Proteção do Sistema se estiver desligada, contorna o limite de 24 horas e confere se o ponto foi criado; se falhar, nada é aplicado).
- Backup granular do registro: só as chaves alteradas são exportadas e restauradas.
- Diário de transações gravado antes de cada passo; se o PC desligar no meio, o programa detecta a operação interrompida e oferece desfazer.
- Verificações prévias: reinicialização pendente, Windows Update instalando, espaço em disco, versão do Windows, notebook ou desktop e bateria.
- Modo simulação: mostra tudo o que vai mudar (antes → depois) e só aplica com a sua aprovação.
- Lista de proteção: Microsoft Store, App Installer (winget), Visual C++, .NET, WebView2, Windows Update, áudio, rede, instalador do Windows, criptografia e antivírus nunca são removidos ou desativados.
- Serviços vão para "Manual" em vez de "Desativado" sempre que possível, com o estado original salvo.
- Limpeza segura: nada de apagar WinSxS ou Windows\Installer na mão (usa DISM), e o Prefetch nunca é limpo.
- Drivers só de sites oficiais via HTTPS, com assinatura digital conferida, backup dos drivers atuais e um driver por vez.
- A interface só chama uma lista fechada de comandos com parâmetros validados; scripts PowerShell embutidos no programa, com hash conferido.
- Respeito ao antivírus instalado (ex.: Kaspersky): nada de mexer no Defender quando outro antivírus está ativo e nada de driblar autoproteção.
- Aviso quando uma atualização do Windows desfaz ajustes (sem reaplicar nada pesado sozinho).
- Log completo com data e hora de cada alteração, exportável.

#### Início
- Medidor animado de saúde do sistema, que sobe conforme as otimizações são aplicadas.
- Otimizar com um clique (perfil Seguro) e Modo Turbo.
- Temperaturas e uso em tempo real; acesso rápido ao "O que há de novo".

#### Diagnóstico (só analisa, não altera nada)
- Verificações de BIOS: DOCP/XMP (velocidade real da RAM), Resizable BAR, Integridade de Memória (só informa), Inicialização Segura e virtualização.
- Latência DPC estimada e TRIM.
- Monitor de culpados por processo (CPU, disco, RAM e rede).
- Relatório de travamentos em português (telas azuis e apps que fecharam).
- Histórico de desempenho ao longo dos dias, com gráfico.
- Detector de vazamento de memória.
- Comparativo antes/depois (CPU, disco e memória) e FPS/frametime com o PresentMon (FPS médio, 1% low e gráfico).
- Saúde dos SSDs (desgaste, temperatura, horas ligadas e erros), com alerta de "Dados não confiáveis" para SSDs com SMART incompleto e histórico ao longo do tempo.
- Ficha do PC exportável (texto para WhatsApp e Markdown para fóruns), sem dados pessoais.

#### Desempenho
- Plano Desempenho Máximo, estacionamento de núcleos, Modo de Jogo e agendamento de GPU por hardware.
- Desligamento de economias ocultas (suspensão seletiva USB, PCIe Link State, economia da placa de rede) e aceleração do mouse.
- Liberação de memória em espera só por botão manual, mostrando antes e depois.
- Perfis por aplicativo: remove o Modo de Eficiência (EcoQoS) do app e dos processos filhos, reaplica se o Windows religar, prioridade Alta (nunca Tempo Real), troca o plano de energia e restaura tudo ao fechar.
- Modo Jogo automático ao detectar jogo aberto.
- CPU Sets para separar CCDs (padrão) e afinidade rígida (avançado), com a topologia real detectada.
- Prioridade de E/S e de memória por processo.
- IdleSaver: desempenho máximo no uso e plano econômico só quando o PC está realmente ocioso.
- Regras "se… então" (Watchdog) no Modo Avançado.
- Ajustes recomendados da NVIDIA guiados passo a passo.

#### Limpeza
- Temporários, cache do Windows Update, limpeza de componentes (DISM), drivers antigos não usados e hibernação opcional.
- Limpeza de launchers e apps (Steam, GOG, Battle.net, Epic, Discord, CapCut e apps da Microsoft Store) só em pastas de cache conhecidas; caches de shaders como opção avançada.
- Analisador de espaço visual e arquivos duplicados (enviados para a Lixeira).
- SFC e DISM.
- Privacidade em tela única com semáforo (verde, amarelo, vermelho): telemetria, anúncios, busca Bing, Copilot, Recall, Widgets, Edge em segundo plano, OneDrive e Otimização de Entrega.
- Remoção de apps pré-instalados com lista marcável e editor do arquivo hosts com backup.

#### Sistema
- Serviços com perfis Seguro e Agressivo e serviços sob demanda.
- Gerenciador de inicialização com inicialização com atraso.
- Recursos opcionais do Windows (Hyper-V, WSL, Sandbox).
- Drivers: comparação de versões, download híbrido e placa de vídeo com três opções (Manual, Automático, Ignorar).
- Apps e atualizações: ajustes do CapCut e do Chrome, orientação de exclusões no antivírus, atualizador via winget, instalação em lote e desinstalador completo.
- Controle do Windows Update, DNS opcional, uso de rede por programa, firewall por app e reparos (rede, Windows Update, ícones e Microsoft Store).
- Ajustes do Explorer e da barra de tarefas.

#### Segurança
- Pontos de restauração, desfazer tudo, desfazer item a item, histórico com data e recuperação de operação interrompida.
- Kit de emergência "Xablau — Restaurar tudo": programa separado que funciona até no Modo de Segurança.

#### Configurações
- 10 temas (Escuro, Claro, AMOLED, Neon, Oceano, Floresta, Pôr do Sol, Grafite, Roxo Xablau e Alto Contraste), editor de tema, fonte, tamanho do texto, densidade e "reduzir animações".
- Iniciar com o Windows (tarefa agendada com privilégio máximo e 30 s de atraso), ícone na bandeja, manutenção no ocioso, exportar/importar perfis e modo portátil.
- Atualização automática assinada (recusa qualquer atualização sem assinatura válida).
- Sobre: créditos, versão e licenças.

#### Interface
- Modo Básico (poucas abas, automático e com linguagem simples) e Modo Avançado (tudo liberado), com chave de troca sempre visível.
- Assistente de primeiro uso animado, busca global (Ctrl+K) e explicação de cada função com selo de risco e "Saiba mais".
- Visual premium com efeito vidro, transições animadas, skeleton loading e progresso em tempo real.
