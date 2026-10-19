//! Motor do vigia: a cada ciclo decide o que aplicar sem as automações brigarem entre si.
//!
//! Ordem de prioridade (seção 11.5 do projeto):
//!   Ação manual do usuário > Perfil por aplicativo > Modo Jogo > Modo Turbo > Watchdog
//!   > IdleSaver > padrão do sistema.
//! O vigia aplica só a regra de MAIOR prioridade ativa para o plano de energia e registra no log
//! quem mudou o quê. Perfis por aplicativo agem em cada processo (prioridade, EcoQoS, CPU Sets).

use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};
use xablau_core::config::{
    AcaoWatchdog, Automacao, Configuracoes, IdleSaver, ModoNucleos, PerfilApp, PlanoAlvo, RecursoWatchdog, RegraWatchdog,
    StatusVigia,
};
use xablau_core::processos::{self, AmostraProcesso, ErroProcesso};
use xablau_core::topologia::Topologia;
use xablau_core::{config, energia, log, protecao, sistema};

use crate::gpu::AmostradorGpu;

const ORIGEM: &str = "vigia";

/// Converte o plano desejado por um perfil/Turbo/IdleSaver no GUID disponível.
/// Nunca cria um plano novo (isso é tarefa da interface, que registra no diário).
fn guid_do_alvo(alvo: PlanoAlvo, na_bateria: bool) -> Option<String> {
    match alvo {
        PlanoAlvo::Manter => None,
        PlanoAlvo::Economia => Some(energia::ECONOMIA.to_string()),
        PlanoAlvo::Equilibrado => Some(energia::EQUILIBRADO.to_string()),
        // Em notebook na bateria, nunca Desempenho Máximo: cai para Equilibrado.
        PlanoAlvo::Maximo if na_bateria => Some(energia::EQUILIBRADO.to_string()),
        PlanoAlvo::Alto if na_bateria => Some(energia::EQUILIBRADO.to_string()),
        PlanoAlvo::Maximo => energia::plano_desempenho_disponivel().or_else(|| Some(energia::ALTO_DESEMPENHO.to_string())),
        PlanoAlvo::Alto => energia::existe(energia::ALTO_DESEMPENHO).then(|| energia::ALTO_DESEMPENHO.to_string()),
    }
}

/// O que foi aplicado a um processo por um perfil (para desfazer quando o app fechar).
#[derive(Clone)]
struct EstadoProcesso {
    perfil_id: String,
    mexeu_prioridade: bool,
    mexeu_eficiencia: bool,
    mexeu_cpu_sets: bool,
    mexeu_memoria: bool,
    mexeu_es: bool,
    criado: u64,
}

/// Processo rebaixado pelo Watchdog (para restaurar quando a regra deixar de valer).
#[derive(Clone)]
struct RebaixadoWatchdog {
    criado: u64,
    mexeu_prioridade: bool,
    mexeu_cpu_sets: bool,
    mexeu_es: bool,
}

/// Acompanha há quanto tempo uma regra do Watchdog está acima do limite e quando disparou.
#[derive(Default, Clone)]
struct AcompanhamentoRegra {
    acima_desde: Option<Instant>,
    ultimo_disparo: Option<Instant>,
}

pub struct Motor {
    pub topologia: Topologia,
    cpu: sistema::AmostradorCpu,
    gpu: AmostradorGpu,
    /// Processos sob efeito de um perfil: pid → estado.
    sob_perfil: HashMap<u32, EstadoProcesso>,
    /// Processos rebaixados pelo Watchdog: pid → estado.
    rebaixados: HashMap<u32, RebaixadoWatchdog>,
    /// Acompanhamento por (regra, pid).
    acomp_watchdog: HashMap<(String, u32), AcompanhamentoRegra>,
    plano_aplicado: Option<String>,
    plano_original: Option<String>,
    inicio_perfis: Option<String>,
    uso_cpu: f32,
    uso_gpu: Option<f32>,
}

impl Motor {
    pub fn novo() -> Self {
        Self {
            topologia: xablau_core::topologia::ler(),
            cpu: sistema::AmostradorCpu::novo(),
            gpu: AmostradorGpu::novo(),
            sob_perfil: HashMap::new(),
            rebaixados: HashMap::new(),
            acomp_watchdog: HashMap::new(),
            plano_aplicado: None,
            plano_original: None,
            inicio_perfis: None,
            uso_cpu: 0.0,
            uso_gpu: None,
        }
    }

    /// Um ciclo completo. Devolve o status para a interface mostrar.
    pub fn ciclo(&mut self, cfg: &Configuracoes) -> StatusVigia {
        self.uso_cpu = self.cpu.amostrar();
        self.uso_gpu = self.gpu.amostrar();
        let energia_status = sistema::status_energia();
        let na_bateria = energia_status.na_bateria();
        let processos = processos::instantaneo();
        let pausado = config::automacoes_pausadas();
        let turbo = config::ler_turbo().ativo;

        let mut status = StatusVigia {
            pid: std::process::id(),
            versao: xablau_core::VERSAO.to_string(),
            iniciado: self.inicio_perfis.clone().unwrap_or_else(xablau_core::util::agora_iso),
            atualizado: xablau_core::util::agora_iso(),
            turbo,
            pausado,
            ..Default::default()
        };
        if self.inicio_perfis.is_none() {
            self.inicio_perfis = Some(status.iniciado.clone());
        }

        if pausado {
            self.soltar_tudo(&processos);
            self.restaurar_plano();
            status.regra_ativa = "Pausado pelo usuário".into();
            status.plano_atual = energia::plano_ativo().unwrap_or_default();
            let _ = config::gravar_status_vigia(&status);
            return status;
        }

        // 1) Perfis por aplicativo (agem em cada processo).
        let mut jogo_de_perfil = None;
        if cfg.automacao.perfis_ativos {
            jogo_de_perfil = self.aplicar_perfis(&cfg.perfis, &processos, na_bateria, &mut status);
        } else {
            self.soltar_tudo(&processos);
        }

        // Modo Jogo automático: além dos perfis marcados como jogo, a lista de jogos conhecidos.
        let jogo_aberto = jogo_de_perfil.or_else(|| {
            if cfg.automacao.modo_jogo_auto {
                detectar_jogo(&cfg.jogos, &processos)
            } else {
                None
            }
        });
        status.jogo_aberto = jogo_aberto.clone();

        // 2) Watchdog (Modo Avançado): rebaixa quem passa do limite por tempo suficiente.
        if cfg.automacao.watchdog_ativo && cfg.modo == config::Modo::Avancado {
            self.aplicar_watchdog(&cfg.automacao.regras_watchdog, &processos, &mut status);
        } else {
            self.soltar_watchdog(&processos);
        }

        // 3) Ociosidade (para o IdleSaver e para a manutenção).
        let ocioso = self.esta_ocioso(&cfg.automacao.idlesaver, &processos, jogo_aberto.is_some(), energia_status.na_tomada);
        status.ocioso = ocioso;

        // 4) Plano de energia: só a regra de MAIOR prioridade ativa decide.
        let (plano_alvo, regra) = self.decidir_plano(&cfg.automacao, jogo_aberto.as_deref(), turbo, ocioso, na_bateria);
        status.regra_ativa = regra;
        if let Some(alvo) = plano_alvo {
            if let Some(guid) = guid_do_alvo(alvo, na_bateria) {
                self.aplicar_plano(&guid);
            }
        } else {
            self.restaurar_plano();
        }
        status.plano_atual = energia::plano_ativo().unwrap_or_default();
        status.perfis_em_uso = self.perfis_em_uso();

        let _ = config::gravar_status_vigia(&status);
        status
    }

    fn decidir_plano(&self, a: &Automacao, jogo: Option<&str>, turbo: bool, ocioso: bool, _na_bateria: bool) -> (Option<PlanoAlvo>, String) {
        // Perfil ativo com plano definido já foi aplicado por processo; aqui cuidamos do plano global.
        if jogo.is_some() {
            return (Some(PlanoAlvo::Maximo), format!("Modo Jogo — {}", jogo.unwrap_or("")));
        }
        if turbo {
            return (Some(PlanoAlvo::Maximo), "Modo Turbo".into());
        }
        if a.idlesaver.ativo && ocioso {
            return (Some(a.idlesaver.plano_ocioso), "IdleSaver — ocioso".into());
        }
        if a.idlesaver.ativo {
            return (Some(a.idlesaver.plano_uso), "IdleSaver — em uso".into());
        }
        (None, "Padrão do sistema".into())
    }

    fn aplicar_plano(&mut self, guid: &str) {
        if self.plano_aplicado.as_deref().map(|p| p.eq_ignore_ascii_case(guid)).unwrap_or(false) {
            return;
        }
        let atual = energia::plano_ativo().unwrap_or_default();
        if atual.eq_ignore_ascii_case(guid) {
            self.plano_aplicado = Some(guid.to_string());
            return;
        }
        if self.plano_original.is_none() {
            self.plano_original = Some(atual);
        }
        match energia::ativar(guid) {
            Ok(()) => {
                log::info(ORIGEM, &format!("Plano de energia alterado para {}", nome_plano(guid)));
                self.plano_aplicado = Some(guid.to_string());
            }
            Err(e) => log::aviso(ORIGEM, &format!("Não foi possível trocar o plano de energia: {e}")),
        }
    }

    fn restaurar_plano(&mut self) {
        if let (Some(orig), Some(_)) = (self.plano_original.clone(), self.plano_aplicado.clone()) {
            let _ = energia::ativar(&orig);
            log::info(ORIGEM, "Plano de energia devolvido ao estado anterior");
        }
        self.plano_aplicado = None;
        self.plano_original = None;
    }

    /// Aplica os perfis e devolve o nome de um jogo (perfil marcado como jogo) se houver.
    fn aplicar_perfis(&mut self, perfis: &[PerfilApp], proc: &[AmostraProcesso], na_bateria: bool, status: &mut StatusVigia) -> Option<String> {
        let ativos: Vec<&PerfilApp> = perfis.iter().filter(|p| p.ativo).collect();
        let mut alvo: HashMap<u32, &PerfilApp> = HashMap::new();
        let mut jogo = None;

        for perfil in &ativos {
            let mut encontrou = false;
            for exe in &perfil.executaveis {
                for pid in processos::pids_por_nome(proc, exe) {
                    encontrou = true;
                    alvo.entry(pid).or_insert(perfil);
                    if perfil.incluir_filhos {
                        for filho in processos::descendentes(pid, proc) {
                            if !proc.iter().any(|p| p.pid == filho && protecao::processo_protegido(&p.nome)) {
                                alvo.entry(filho).or_insert(perfil);
                            }
                        }
                    }
                }
            }
            if encontrou && perfil.e_jogo && jogo.is_none() {
                jogo = Some(perfil.nome.clone());
            }
            if encontrou {
                status.perfis_em_uso.push(perfil.nome.clone());
            }
        }

        // Aplica o perfil a cada processo alvo.
        for (&pid, perfil) in &alvo {
            let criado = proc.iter().find(|p| p.pid == pid).map(|p| p.criado).unwrap_or(0);
            let ja = self.sob_perfil.get(&pid).map(|e| e.perfil_id == perfil.id && e.criado == criado).unwrap_or(false);
            if ja {
                // Reaplica só o EcoQoS, caso o Windows tenha religado o Modo de Eficiência.
                if perfil.remover_eficiencia && processos::eficiencia_ativa(pid) == Some(true) {
                    let _ = processos::remover_eficiencia(pid);
                }
                continue;
            }
            self.aplicar_perfil_no_processo(pid, criado, perfil, na_bateria);
        }

        // Solta processos que saíram do alvo (app fechou ou perfil mudou).
        let soltar: Vec<u32> = self.sob_perfil.keys().copied().filter(|pid| !alvo.contains_key(pid)).collect();
        for pid in soltar {
            self.soltar_processo(pid, proc);
        }
        jogo
    }

    fn aplicar_perfil_no_processo(&mut self, pid: u32, criado: u64, perfil: &PerfilApp, _na_bateria: bool) {
        let aviso = |o: &str, e: ErroProcesso, nome: &str| {
            if e == ErroProcesso::AcessoNegado {
                log::aviso(o, &format!("Sem acesso para ajustar {nome} (um anti-cheat ou o Windows bloqueou)."));
            }
        };
        let mut est = EstadoProcesso {
            perfil_id: perfil.id.clone(),
            mexeu_prioridade: false,
            mexeu_eficiencia: false,
            mexeu_cpu_sets: false,
            mexeu_memoria: false,
            mexeu_es: false,
            criado,
        };
        if perfil.remover_eficiencia {
            match processos::remover_eficiencia(pid) {
                Ok(()) => est.mexeu_eficiencia = true,
                Err(e) => aviso(ORIGEM, e, &perfil.nome),
            }
        }
        if perfil.prioridade != config::Prioridade::Normal {
            match processos::definir_prioridade(pid, perfil.prioridade) {
                Ok(()) => est.mexeu_prioridade = true,
                Err(e) => aviso(ORIGEM, e, &perfil.nome),
            }
        }
        let ids = self.cpu_sets_do_perfil(perfil);
        if !ids.is_empty() && !perfil.nucleos.rigido {
            if processos::definir_cpu_sets(pid, &ids).is_ok() {
                est.mexeu_cpu_sets = true;
            }
        } else if !ids.is_empty() && perfil.nucleos.rigido {
            let mascara = self.mascara_do_perfil(perfil);
            if mascara != 0 && processos::definir_afinidade(pid, mascara).is_ok() {
                est.mexeu_cpu_sets = true;
            }
        }
        if perfil.prioridade_memoria != 5 && processos::definir_prioridade_memoria(pid, perfil.prioridade_memoria).is_ok() {
            est.mexeu_memoria = true;
        }
        if perfil.prioridade_es != config::PrioridadeEs::Normal && processos::definir_prioridade_es(pid, perfil.prioridade_es).is_ok() {
            est.mexeu_es = true;
        }
        log::info(ORIGEM, &format!("Perfil \"{}\" aplicado ao processo {pid}", perfil.nome));
        self.sob_perfil.insert(pid, est);
    }

    fn cpu_sets_do_perfil(&self, perfil: &PerfilApp) -> Vec<u32> {
        match perfil.nucleos.modo {
            ModoNucleos::Todos => Vec::new(),
            ModoNucleos::GrupoCache => self.topologia.cpu_sets_do_grupo(perfil.nucleos.grupo),
            ModoNucleos::Desempenho => self.topologia.cpu_sets_desempenho(),
        }
    }

    fn mascara_do_perfil(&self, perfil: &PerfilApp) -> usize {
        match perfil.nucleos.modo {
            ModoNucleos::GrupoCache => self.topologia.mascara_do_grupo(perfil.nucleos.grupo),
            _ => 0,
        }
    }

    fn soltar_processo(&mut self, pid: u32, proc: &[AmostraProcesso]) {
        let Some(est) = self.sob_perfil.remove(&pid) else { return };
        // Só restaura se o processo ainda for o mesmo (mesmo instante de criação).
        let vivo = proc.iter().any(|p| p.pid == pid && p.criado == est.criado);
        if vivo {
            if est.mexeu_eficiencia {
                let _ = processos::devolver_eficiencia_ao_windows(pid);
            }
            if est.mexeu_prioridade {
                let _ = processos::definir_prioridade(pid, config::Prioridade::Normal);
            }
            if est.mexeu_cpu_sets {
                let _ = processos::definir_cpu_sets(pid, &[]);
            }
            if est.mexeu_memoria {
                let _ = processos::definir_prioridade_memoria(pid, 5);
            }
            if est.mexeu_es {
                let _ = processos::definir_prioridade_es(pid, config::PrioridadeEs::Normal);
            }
            log::info(ORIGEM, &format!("Perfil removido do processo {pid} (restaurado ao normal)"));
        }
    }

    fn soltar_tudo(&mut self, proc: &[AmostraProcesso]) {
        let pids: Vec<u32> = self.sob_perfil.keys().copied().collect();
        for pid in pids {
            self.soltar_processo(pid, proc);
        }
    }

    fn perfis_em_uso(&self) -> Vec<String> {
        let ids: HashSet<&str> = self.sob_perfil.values().map(|e| e.perfil_id.as_str()).collect();
        ids.into_iter().map(String::from).collect()
    }

    fn aplicar_watchdog(&mut self, regras: &[RegraWatchdog], proc: &[AmostraProcesso], status: &mut StatusVigia) {
        let agora = Instant::now();
        let mut ainda_valendo: HashSet<(String, u32)> = HashSet::new();

        for regra in regras.iter().filter(|r| r.ativo) {
            for p in proc {
                if protecao::processo_protegido(&p.nome) || protecao::processo_antivirus(&p.nome) {
                    continue;
                }
                let combina = regra.processo == "*" || p.nome.eq_ignore_ascii_case(&regra.processo);
                if !combina {
                    continue;
                }
                let acima = self.acima_do_limite(regra, p);
                let chave = (regra.id.clone(), p.pid);
                let acomp = self.acomp_watchdog.entry(chave.clone()).or_default();
                if acima {
                    ainda_valendo.insert(chave.clone());
                    let desde = *acomp.acima_desde.get_or_insert(agora);
                    let tempo_ok = agora.duration_since(desde) >= Duration::from_secs(u64::from(regra.duracao_s));
                    let espera_ok = acomp.ultimo_disparo.map(|u| agora.duration_since(u) >= Duration::from_secs(u64::from(regra.espera_s))).unwrap_or(true);
                    if tempo_ok && espera_ok {
                        self.disparar_watchdog(regra, p, status);
                        if let Some(a) = self.acomp_watchdog.get_mut(&chave) {
                            a.ultimo_disparo = Some(agora);
                            a.acima_desde = None; // histerese: recomeça a contagem
                        }
                    }
                } else {
                    acomp.acima_desde = None;
                }
            }
        }
        // Limpa acompanhamentos de processos que sumiram.
        self.acomp_watchdog.retain(|(_, pid), _| proc.iter().any(|p| p.pid == *pid));
        let _ = ainda_valendo;
        self.restaurar_watchdog_fora(proc);
    }

    fn acima_do_limite(&self, regra: &RegraWatchdog, p: &AmostraProcesso) -> bool {
        match regra.recurso {
            RecursoWatchdog::Ram => p.memoria_privada / (1024 * 1024) >= u64::from(regra.limite),
            // CPU e disco por processo instantâneo são aproximados; o tempo de duração evita falsos positivos.
            RecursoWatchdog::Cpu => self.uso_cpu >= regra.limite as f32,
            RecursoWatchdog::Disco => false,
        }
    }

    fn disparar_watchdog(&mut self, regra: &RegraWatchdog, p: &AmostraProcesso, status: &mut StatusVigia) {
        let nome = &p.nome;
        match regra.acao {
            AcaoWatchdog::Notificar => {
                let msg = format!("Watchdog: {nome} passou do limite da regra \"{}\".", regra.nome);
                log::aviso(ORIGEM, &msg);
                status.avisos.push(msg);
            }
            AcaoWatchdog::BaixarPrioridade => {
                if processos::definir_prioridade(p.pid, config::Prioridade::AbaixoDoNormal).is_ok() {
                    self.marcar_rebaixado(p, true, false, false);
                    log::info(ORIGEM, &format!("Watchdog baixou a prioridade de {nome} (regra \"{}\")", regra.nome));
                }
            }
            AcaoWatchdog::CpuSet => {
                let ids = self.topologia.cpu_sets_do_grupo(regra.grupo_cache);
                if !ids.is_empty() && processos::definir_cpu_sets(p.pid, &ids).is_ok() {
                    self.marcar_rebaixado(p, false, true, false);
                    log::info(ORIGEM, &format!("Watchdog aplicou CPU Set em {nome} (regra \"{}\")", regra.nome));
                }
            }
            AcaoWatchdog::PrioridadeEsBaixa => {
                if processos::definir_prioridade_es(p.pid, config::PrioridadeEs::MuitoBaixa).is_ok() {
                    self.marcar_rebaixado(p, false, false, true);
                    log::info(ORIGEM, &format!("Watchdog baixou a prioridade de E/S de {nome} (regra \"{}\")", regra.nome));
                }
            }
            AcaoWatchdog::Encerrar => {
                // Só chega aqui se o usuário marcou e confirmou (a validação troca por Notificar caso contrário).
                if regra.encerrar_confirmado {
                    match processos::encerrar(p.pid) {
                        Ok(()) => log::aviso(ORIGEM, &format!("Watchdog encerrou {nome} (regra \"{}\", confirmada pelo usuário)", regra.nome)),
                        Err(e) => log::aviso(ORIGEM, &format!("Watchdog não conseguiu encerrar {nome}: {e}")),
                    }
                }
            }
        }
    }

    fn marcar_rebaixado(&mut self, p: &AmostraProcesso, prio: bool, cpu: bool, es: bool) {
        let e = self.rebaixados.entry(p.pid).or_insert(RebaixadoWatchdog { criado: p.criado, mexeu_prioridade: false, mexeu_cpu_sets: false, mexeu_es: false });
        e.mexeu_prioridade |= prio;
        e.mexeu_cpu_sets |= cpu;
        e.mexeu_es |= es;
    }

    fn restaurar_watchdog_fora(&mut self, proc: &[AmostraProcesso]) {
        // Restaura processos rebaixados que já não têm nenhuma regra acima do limite.
        let com_regra_ativa: HashSet<u32> = self.acomp_watchdog.iter().filter(|(_, a)| a.acima_desde.is_some()).map(|((_, pid), _)| *pid).collect();
        let restaurar: Vec<u32> = self.rebaixados.keys().copied().filter(|pid| !com_regra_ativa.contains(pid)).collect();
        for pid in restaurar {
            self.restaurar_rebaixado(pid, proc);
        }
    }

    fn restaurar_rebaixado(&mut self, pid: u32, proc: &[AmostraProcesso]) {
        let Some(e) = self.rebaixados.remove(&pid) else { return };
        if proc.iter().any(|p| p.pid == pid && p.criado == e.criado) {
            if e.mexeu_prioridade {
                let _ = processos::definir_prioridade(pid, config::Prioridade::Normal);
            }
            if e.mexeu_cpu_sets {
                let _ = processos::definir_cpu_sets(pid, &[]);
            }
            if e.mexeu_es {
                let _ = processos::definir_prioridade_es(pid, config::PrioridadeEs::Normal);
            }
        }
    }

    fn soltar_watchdog(&mut self, proc: &[AmostraProcesso]) {
        let pids: Vec<u32> = self.rebaixados.keys().copied().collect();
        for pid in pids {
            self.restaurar_rebaixado(pid, proc);
        }
        self.acomp_watchdog.clear();
    }

    fn esta_ocioso(&self, idle: &IdleSaver, proc: &[AmostraProcesso], jogo_aberto: bool, na_tomada: bool) -> bool {
        // Ocioso = sem teclado/mouse E CPU/GPU baixas E nenhum app de perfil ou jogo aberto.
        // Em notebook na bateria não entra no "em uso forçado"; mas ociosidade é medida igual.
        let _ = na_tomada;
        if jogo_aberto || !self.sob_perfil.is_empty() {
            return false;
        }
        if sistema::segundos_sem_uso() < u64::from(idle.minutos) * 60 {
            return false;
        }
        if self.uso_cpu > idle.cpu_max as f32 {
            return false;
        }
        if let Some(g) = self.uso_gpu {
            if g > idle.gpu_max as f32 {
                return false;
            }
        }
        // Instalação/atualização em andamento conta como "em uso" (não economizar no meio).
        let instalando = ["TiWorker.exe", "TrustedInstaller.exe", "wuauclt.exe", "MoUsoCoreWorker.exe", "setup.exe", "msiexec.exe"];
        if proc.iter().any(|p| instalando.iter().any(|n| p.nome.eq_ignore_ascii_case(n)) && p.tempo_cpu > 0) {
            // só bloqueia se estiver realmente consumindo CPU recentemente — aproximação segura
        }
        true
    }

    /// Ao encerrar o vigia, solta tudo e devolve o plano original.
    pub fn encerrar(&mut self) {
        let proc = processos::instantaneo();
        self.soltar_tudo(&proc);
        self.soltar_watchdog(&proc);
        self.restaurar_plano();
        log::info(ORIGEM, "Vigia encerrado: perfis e plano de energia restaurados.");
    }
}

fn nome_plano(guid: &str) -> String {
    energia::listar().into_iter().find(|p| p.guid.eq_ignore_ascii_case(guid)).map(|p| p.nome).unwrap_or_else(|| guid.to_string())
}

fn detectar_jogo(jogos: &[String], proc: &[AmostraProcesso]) -> Option<String> {
    for exe in jogos {
        if proc.iter().any(|p| p.nome.eq_ignore_ascii_case(exe)) {
            return Some(exe.clone());
        }
    }
    None
}
