//! Configurações compartilhadas entre a interface e o vigia (`config\configuracoes.json`)
//! e o estado em tempo de execução (`estado\*.json`).
//!
//! Só a interface grava as configurações; o vigia apenas lê (e relê quando o arquivo muda).

use crate::{caminhos, protecao, util};
use serde::{Deserialize, Serialize};
use std::io;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Modo {
    #[default]
    Basico,
    Avancado,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum UsoPrincipal {
    Jogos,
    Edicao,
    #[default]
    Geral,
}

/// Prioridade de processo. Tempo Real NUNCA existe aqui, de propósito.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Prioridade {
    Ociosa,
    AbaixoDoNormal,
    #[default]
    Normal,
    AcimaDoNormal,
    Alta,
}

/// Plano de energia desejado por um perfil, Turbo ou IdleSaver.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PlanoAlvo {
    Maximo,
    Alto,
    Equilibrado,
    Economia,
    #[default]
    Manter,
}

/// Como separar os núcleos: CPU Sets (padrão, o Windows evita os outros núcleos sem proibir)
/// ou afinidade rígida (avançado).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ModoNucleos {
    #[default]
    Todos,
    /// Um grupo de cache L3 (CCD/CCX do Ryzen), pelo índice detectado.
    GrupoCache,
    /// Só núcleos de desempenho (processadores híbridos Intel).
    Desempenho,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Nucleos {
    pub modo: ModoNucleos,
    pub grupo: u8,
    pub rigido: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PrioridadeEs {
    MuitoBaixa,
    Baixa,
    #[default]
    Normal,
}

/// Perfil por aplicativo (estilo Process Lasso).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PerfilApp {
    pub id: String,
    pub nome: String,
    pub executaveis: Vec<String>,
    pub ativo: bool,
    pub e_jogo: bool,
    pub remover_eficiencia: bool,
    pub incluir_filhos: bool,
    pub prioridade: Prioridade,
    pub plano: PlanoAlvo,
    pub nucleos: Nucleos,
    pub prioridade_es: PrioridadeEs,
    /// 1 (muito baixa) a 5 (normal).
    pub prioridade_memoria: u8,
    /// Reduz prioridade de E/S e de memória dos apps em segundo plano enquanto este estiver aberto.
    pub reduzir_segundo_plano: bool,
}

impl Default for PerfilApp {
    fn default() -> Self {
        Self {
            id: String::new(),
            nome: String::new(),
            executaveis: Vec::new(),
            ativo: true,
            e_jogo: false,
            remover_eficiencia: true,
            incluir_filhos: true,
            prioridade: Prioridade::Alta,
            plano: PlanoAlvo::Maximo,
            nucleos: Nucleos::default(),
            prioridade_es: PrioridadeEs::Normal,
            prioridade_memoria: 5,
            reduzir_segundo_plano: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct IdleSaver {
    pub ativo: bool,
    /// Minutos sem teclado/mouse para considerar ocioso.
    pub minutos: u32,
    /// CPU abaixo disso (%) para considerar ocioso.
    pub cpu_max: u8,
    /// GPU abaixo disso (%) para considerar ocioso.
    pub gpu_max: u8,
    pub plano_ocioso: PlanoAlvo,
    pub plano_uso: PlanoAlvo,
}

impl Default for IdleSaver {
    fn default() -> Self {
        Self { ativo: false, minutos: 10, cpu_max: 12, gpu_max: 10, plano_ocioso: PlanoAlvo::Equilibrado, plano_uso: PlanoAlvo::Maximo }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RecursoWatchdog {
    #[default]
    Cpu,
    Ram,
    Disco,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AcaoWatchdog {
    #[default]
    BaixarPrioridade,
    CpuSet,
    PrioridadeEsBaixa,
    Notificar,
    /// Só disponível marcando manualmente, com confirmação. Nunca é padrão.
    Encerrar,
}

/// Regra "se… então" do Watchdog (somente Modo Avançado).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct RegraWatchdog {
    pub id: String,
    pub nome: String,
    pub ativo: bool,
    /// Executável alvo ("chrome.exe") ou "*" para qualquer processo (exceto os protegidos).
    pub processo: String,
    pub recurso: RecursoWatchdog,
    /// CPU/disco: porcentagem. RAM: megabytes.
    pub limite: u32,
    pub duracao_s: u32,
    pub acao: AcaoWatchdog,
    pub grupo_cache: u8,
    /// Tempo mínimo entre dois disparos da mesma regra no mesmo processo.
    pub espera_s: u32,
    /// Confirmação explícita do usuário para a ação "Encerrar".
    pub encerrar_confirmado: bool,
}

impl Default for RegraWatchdog {
    fn default() -> Self {
        Self {
            id: String::new(),
            nome: String::new(),
            ativo: true,
            processo: String::new(),
            recurso: RecursoWatchdog::Cpu,
            limite: 80,
            duracao_s: 30,
            acao: AcaoWatchdog::BaixarPrioridade,
            grupo_cache: 0,
            espera_s: 120,
            encerrar_confirmado: false,
        }
    }
}

/// Programa da inicialização com atraso (aberto um por vez pelo vigia).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ItemInicioAtrasado {
    pub id: String,
    pub nome: String,
    pub comando: String,
    pub origem: String,
    pub ordem: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Automacao {
    pub iniciar_com_windows: bool,
    pub vigia_ativo: bool,
    pub perfis_ativos: bool,
    pub modo_jogo_auto: bool,
    pub idlesaver: IdleSaver,
    pub watchdog_ativo: bool,
    pub regras_watchdog: Vec<RegraWatchdog>,
    pub manutencao_ociosa: bool,
    pub inicio_atrasado: Vec<ItemInicioAtrasado>,
    pub intervalo_inicio_atrasado_s: u32,
    pub avisar_reversoes: bool,
    pub notificacoes: bool,
}

impl Default for Automacao {
    fn default() -> Self {
        Self {
            iniciar_com_windows: false,
            vigia_ativo: true,
            perfis_ativos: true,
            modo_jogo_auto: true,
            idlesaver: IdleSaver::default(),
            watchdog_ativo: false,
            regras_watchdog: Vec::new(),
            manutencao_ociosa: false,
            inicio_atrasado: Vec::new(),
            intervalo_inicio_atrasado_s: 8,
            avisar_reversoes: true,
            notificacoes: true,
        }
    }
}

/// Placa de vídeo: Manual (padrão, só avisa), Automático ou Ignorar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ModoDriverVideo {
    #[default]
    Manual,
    Automatico,
    Ignorar,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Drivers {
    pub video: ModoDriverVideo,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AlertasSsd {
    pub desgaste_pct: u8,
    pub temperatura_c: u8,
}

impl Default for AlertasSsd {
    fn default() -> Self {
        Self { desgaste_pct: 80, temperatura_c: 70 }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Atualizacao {
    /// No Modo Básico, instala sozinho ao fechar o programa.
    pub automatica_no_basico: bool,
    pub lembrar_depois_ate: Option<String>,
}

impl Default for Atualizacao {
    fn default() -> Self {
        Self { automatica_no_basico: true, lembrar_depois_ate: None }
    }
}

/// Todas as configurações do programa.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Configuracoes {
    pub versao_esquema: u32,
    pub modo: Modo,
    pub assistente_concluido: bool,
    pub uso_principal: UsoPrincipal,
    /// Última versão em que o usuário viu o "O que há de novo".
    pub ultima_versao_vista: String,
    /// Preferências visuais da interface (tema, fonte, densidade...). O Rust só guarda.
    pub interface: serde_json::Map<String, serde_json::Value>,
    pub automacao: Automacao,
    pub perfis: Vec<PerfilApp>,
    /// Executáveis marcados como jogo (além dos detectados).
    pub jogos: Vec<String>,
    pub drivers: Drivers,
    pub alertas_ssd: AlertasSsd,
    pub atualizacao: Atualizacao,
}

impl Default for Configuracoes {
    fn default() -> Self {
        Self {
            versao_esquema: 1,
            modo: Modo::Basico,
            assistente_concluido: false,
            uso_principal: UsoPrincipal::Geral,
            ultima_versao_vista: String::new(),
            interface: serde_json::Map::new(),
            automacao: Automacao::default(),
            perfis: Vec::new(),
            jogos: Vec::new(),
            drivers: Drivers::default(),
            alertas_ssd: AlertasSsd::default(),
            atualizacao: Atualizacao::default(),
        }
    }
}

/// Nome de executável aceitável num perfil (ex.: "CapCut.exe").
pub fn exe_valido(nome: &str) -> bool {
    let n = nome.trim();
    n.len() >= 5
        && n.len() <= 80
        && n.to_ascii_lowercase().ends_with(".exe")
        && !n.contains("..")
        && n.chars().all(|c| c.is_alphanumeric() || " ._-()+[]&'".contains(c))
}

fn limpar_exes(lista: &mut Vec<String>) {
    let mut vistos = std::collections::HashSet::new();
    lista.retain(|e| {
        let e = e.trim().to_string();
        exe_valido(&e) && !protecao::processo_protegido(&e) && vistos.insert(e.to_ascii_lowercase())
    });
    for e in lista.iter_mut() {
        *e = e.trim().to_string();
    }
}

impl Configuracoes {
    /// Corrige valores fora da faixa e remove o que for inválido ou protegido.
    pub fn validar(&mut self) {
        self.versao_esquema = 1;
        self.ultima_versao_vista.truncate(32);
        if serde_json::to_vec(&self.interface).map(|v| v.len()).unwrap_or(0) > 128 * 1024 {
            self.interface.clear();
        }
        self.perfis.truncate(64);
        for p in &mut self.perfis {
            if !util::id_valido(&p.id) {
                p.id = util::gerar_id("perfil");
            }
            p.nome = p.nome.trim().chars().take(60).collect();
            limpar_exes(&mut p.executaveis);
            p.executaveis.truncate(16);
            p.prioridade_memoria = p.prioridade_memoria.clamp(1, 5);
            p.nucleos.grupo = p.nucleos.grupo.min(63);
        }
        self.perfis.retain(|p| !p.executaveis.is_empty());
        limpar_exes(&mut self.jogos);
        self.jogos.truncate(500);
        let a = &mut self.automacao;
        a.idlesaver.minutos = a.idlesaver.minutos.clamp(2, 240);
        a.idlesaver.cpu_max = a.idlesaver.cpu_max.clamp(1, 60);
        a.idlesaver.gpu_max = a.idlesaver.gpu_max.clamp(1, 60);
        a.intervalo_inicio_atrasado_s = a.intervalo_inicio_atrasado_s.clamp(2, 120);
        a.inicio_atrasado.truncate(40);
        a.regras_watchdog.truncate(64);
        for r in &mut a.regras_watchdog {
            if !util::id_valido(&r.id) {
                r.id = util::gerar_id("regra");
            }
            r.nome = r.nome.trim().chars().take(60).collect();
            let alvo = r.processo.trim().to_string();
            r.processo = if alvo == "*" || (exe_valido(&alvo) && !protecao::processo_protegido(&alvo)) { alvo } else { String::new() };
            r.limite = match r.recurso {
                RecursoWatchdog::Ram => r.limite.clamp(64, 262_144),
                _ => r.limite.clamp(5, 100),
            };
            r.duracao_s = r.duracao_s.clamp(5, 3600);
            r.espera_s = r.espera_s.clamp(30, 86_400);
            // "Encerrar" só vale com confirmação explícita; sem ela, vira só um aviso.
            if r.acao == AcaoWatchdog::Encerrar && !r.encerrar_confirmado {
                r.acao = AcaoWatchdog::Notificar;
            }
        }
        a.regras_watchdog.retain(|r| !r.processo.is_empty());
        self.alertas_ssd.desgaste_pct = self.alertas_ssd.desgaste_pct.clamp(10, 100);
        self.alertas_ssd.temperatura_c = self.alertas_ssd.temperatura_c.clamp(40, 95);
    }
}

/// Carrega as configurações (ou os padrões, se o arquivo não existir/estiver inválido).
pub fn carregar() -> Configuracoes {
    let mut c: Configuracoes = util::ler_json(&caminhos::arquivo_configuracoes()).unwrap_or_default();
    c.validar();
    c
}

/// Salva as configurações (validadas) de forma atômica.
pub fn salvar(c: &Configuracoes) -> io::Result<()> {
    let mut c = c.clone();
    c.validar();
    util::escrever_json(&caminhos::arquivo_configuracoes(), &c)
}

/// Pedido de Modo Turbo (gravado pela interface ou pelo menu da bandeja).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct EstadoTurbo {
    pub ativo: bool,
    pub desde: Option<String>,
    pub origem: String,
}

/// Situação do vigia (gravada por ele a cada poucos segundos, lida pela interface).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct StatusVigia {
    pub pid: u32,
    pub versao: String,
    pub iniciado: String,
    pub atualizado: String,
    pub turbo: bool,
    pub pausado: bool,
    /// Regra que está mandando agora (ordem: manual > perfil > Modo Jogo > Turbo > Watchdog > IdleSaver > padrão).
    pub regra_ativa: String,
    pub plano_atual: String,
    pub perfis_em_uso: Vec<String>,
    pub jogo_aberto: Option<String>,
    pub ocioso: bool,
    pub avisos: Vec<String>,
}

fn arquivo_estado(nome: &str) -> std::path::PathBuf {
    caminhos::estado().join(nome)
}

pub fn ler_turbo() -> EstadoTurbo {
    util::ler_json(&arquivo_estado("turbo.json")).unwrap_or_default()
}

pub fn gravar_turbo(ativo: bool, origem: &str) -> io::Result<()> {
    let e = EstadoTurbo { ativo, desde: ativo.then(util::agora_iso), origem: origem.chars().take(20).collect() };
    util::escrever_json(&arquivo_estado("turbo.json"), &e)
}

pub fn ler_status_vigia() -> Option<StatusVigia> {
    util::ler_json(&arquivo_estado("vigia.json"))
}

pub fn gravar_status_vigia(s: &StatusVigia) -> io::Result<()> {
    util::escrever_json(&arquivo_estado("vigia.json"), s)
}

/// Pausa das automações pedida pelo usuário (menu da bandeja).
pub fn automacoes_pausadas() -> bool {
    util::ler_json::<bool>(&arquivo_estado("pausa.json")).unwrap_or(false)
}

pub fn pausar_automacoes(pausar: bool) -> io::Result<()> {
    util::escrever_json(&arquivo_estado("pausa.json"), &pausar)
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn validacao_remove_perigos() {
        let mut c = Configuracoes::default();
        c.perfis.push(PerfilApp { executaveis: vec!["CapCut.exe".into(), "svchost.exe".into(), "..\\x.exe".into()], ..Default::default() });
        c.automacao.regras_watchdog.push(RegraWatchdog { processo: "chrome.exe".into(), acao: AcaoWatchdog::Encerrar, ..Default::default() });
        c.validar();
        assert_eq!(c.perfis[0].executaveis, vec!["CapCut.exe".to_string()]);
        assert_eq!(c.automacao.regras_watchdog[0].acao, AcaoWatchdog::Notificar);
        let json = serde_json::to_string(&c).expect("json");
        let volta: Configuracoes = serde_json::from_str(&json).expect("ler");
        assert_eq!(volta, c);
    }
}
