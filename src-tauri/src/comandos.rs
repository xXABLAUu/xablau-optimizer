//! Lista FECHADA de comandos que a interface pode chamar. Cada comando valida seus parâmetros.
//! A interface nunca monta comandos a partir de texto livre: só chama estas funções tipadas.

use crate::atualizador::{self, InfoAtualizacao};
use crate::diagnostico::{self, Culpado, Hardware, SaudeSsd, UsoTempoReal};
use crate::estado::Estado;
use crate::catalogo::AjusteParaInterface;
use serde::Serialize;
use std::sync::Mutex;
use tauri::{AppHandle, Manager, State};
use xablau_core::aplicador::{ResultadoAplicacao, Simulacao};
use xablau_core::config::{self, Configuracoes, StatusVigia};
use xablau_core::diario::{self, Operacao, ResultadoDesfazer, Revertida};
use xablau_core::{log, restauracao, sistema, verificacoes};

/// Amostrador de CPU mantido entre chamadas (para o uso em tempo real).
pub struct AmostradorCompartilhado(pub Mutex<sistema::AmostradorCpu>);

/// Resposta de "sobre o programa" (aba Sobre e "O que há de novo").
#[derive(Serialize)]
pub struct SobrePrograma {
    pub nome: String,
    pub criador: String,
    pub versao: String,
    pub portatil: bool,
    pub admin: bool,
    pub modo_seguranca: bool,
}

// ---------- Informações e diagnóstico (só leem) ----------

#[tauri::command]
pub fn sobre() -> SobrePrograma {
    SobrePrograma {
        nome: xablau_core::NOME_PRODUTO.into(),
        criador: xablau_core::CRIADOR.into(),
        versao: xablau_core::VERSAO.into(),
        portatil: atualizador::e_portatil(),
        admin: sistema::e_administrador(),
        modo_seguranca: sistema::modo_seguranca(),
    }
}

#[tauri::command]
pub fn obter_hardware(estado: State<Estado>) -> Hardware {
    diagnostico::hardware(&estado.topologia, &sistema::versao_windows())
}

#[tauri::command]
pub fn obter_topologia(estado: State<Estado>) -> xablau_core::topologia::Topologia {
    estado.topologia.clone()
}

#[tauri::command]
pub fn uso_tempo_real(amostrador: State<AmostradorCompartilhado>) -> UsoTempoReal {
    let mut guard = amostrador.0.lock().unwrap_or_else(|e| e.into_inner());
    diagnostico::uso_tempo_real(&mut guard)
}

#[tauri::command]
pub fn monitor_culpados(limite: Option<usize>) -> Vec<Culpado> {
    diagnostico::culpados(limite.unwrap_or(15).clamp(1, 60))
}

#[tauri::command]
pub fn saude_ssds() -> Vec<SaudeSsd> {
    diagnostico::saude_ssds()
}

#[tauri::command]
pub fn verificacoes_previas() -> verificacoes::Relatorio {
    verificacoes::verificar(2.0)
}

#[tauri::command]
pub fn ficha_pc(estado: State<Estado>, markdown: bool) -> String {
    let hw = diagnostico::hardware(&estado.topologia, &sistema::versao_windows());
    let ssds = diagnostico::saude_ssds();
    diagnostico::ficha_pc(&hw, &ssds, markdown)
}

// ---------- Catálogo, simulação e aplicação ----------

#[tauri::command]
pub fn listar_ajustes(estado: State<Estado>) -> Vec<AjusteParaInterface> {
    estado.catalogo.listar()
}

/// Monta a simulação (antes → depois) de uma lista de ajustes, sem aplicar nada.
#[tauri::command]
pub fn simular(estado: State<Estado>, ids: Vec<String>) -> Result<Simulacao, String> {
    if ids.is_empty() || ids.len() > 500 {
        return Err("Lista de ajustes inválida.".into());
    }
    let (planejadas, sem_efeito, erros) = estado.catalogo.planejar(&ids);
    Ok(xablau_core::aplicador::simular(planejadas, sem_efeito, erros))
}

/// Aplica os ajustes: verificações → ponto de restauração → diário → aplicar (tudo reversível).
#[tauri::command]
pub fn aplicar_ajustes(estado: State<Estado>, titulo: String, ids: Vec<String>) -> Result<ResultadoAplicacao, String> {
    if ids.is_empty() || ids.len() > 500 {
        return Err("Lista de ajustes inválida.".into());
    }
    if !sistema::e_administrador() {
        return Err("Abra o programa como administrador para aplicar ajustes.".into());
    }
    let relatorio = verificacoes::verificar(2.0);
    if !relatorio.pode_prosseguir {
        let bloqueios: Vec<String> = relatorio.itens.iter().filter(|i| i.nivel == verificacoes::Nivel::Bloqueio).map(|i| i.detalhe.clone()).collect();
        return Err(format!("Não é seguro aplicar agora: {}", bloqueios.join(" ")));
    }
    let titulo = titulo.chars().take(80).collect::<String>();
    let (planejadas, _sem_efeito, _erros) = estado.catalogo.planejar(&ids);
    if planejadas.is_empty() {
        return Err("Nada a aplicar: os ajustes escolhidos já estão como desejado.".into());
    }
    xablau_core::aplicador::aplicar(&titulo, &planejadas, true)
}

#[tauri::command]
pub fn criar_ponto_restauracao(descricao: String) -> restauracao::ResultadoPonto {
    restauracao::criar(&descricao.chars().take(64).collect::<String>())
}

// ---------- Histórico e desfazer (aba Segurança) ----------

#[tauri::command]
pub fn listar_operacoes() -> Vec<Operacao> {
    diario::listar_operacoes()
}

#[tauri::command]
pub fn operacoes_interrompidas() -> Vec<Operacao> {
    diario::operacoes_incompletas()
}

#[tauri::command]
pub fn desfazer_tudo() -> Result<Vec<(String, ResultadoDesfazer)>, String> {
    diario::desfazer_tudo("interface")
}

#[tauri::command]
pub fn desfazer_operacao(op_id: String) -> Result<ResultadoDesfazer, String> {
    diario::desfazer_operacao(&op_id, "interface")
}

#[tauri::command]
pub fn desfazer_item(op_id: String, item_id: u32) -> Result<ResultadoDesfazer, String> {
    diario::desfazer_item(&op_id, item_id, "interface")
}

#[tauri::command]
pub fn conflitos_do_item(op_id: String, item_id: u32) -> Vec<String> {
    diario::conflitos_posteriores(&op_id, item_id)
}

#[tauri::command]
pub fn encerrar_interrompida(op_id: String) -> Result<(), String> {
    diario::encerrar_incompleta(&op_id, "interface")
}

#[tauri::command]
pub fn alteracoes_revertidas() -> Vec<Revertida> {
    diario::alteracoes_revertidas()
}

// ---------- Log ----------

#[tauri::command]
pub fn log_recente(linhas: Option<usize>) -> Vec<String> {
    log::ler_recentes(linhas.unwrap_or(300).clamp(10, 5000))
}

#[tauri::command]
pub fn exportar_log(pasta: String) -> Result<String, String> {
    let destino = std::path::PathBuf::from(&pasta);
    if !destino.is_dir() {
        return Err("Pasta de destino inválida.".into());
    }
    log::exportar(&destino).map(|p| p.to_string_lossy().into_owned()).map_err(|e| e.to_string())
}

// ---------- Configurações, perfis e automações ----------

#[tauri::command]
pub fn carregar_config() -> Configuracoes {
    config::carregar()
}

#[tauri::command]
pub fn salvar_config(cfg: Configuracoes) -> Result<(), String> {
    config::salvar(&cfg).map_err(|e| format!("Não foi possível salvar as configurações: {e}"))
}

#[tauri::command]
pub fn status_vigia() -> Option<StatusVigia> {
    config::ler_status_vigia()
}

#[tauri::command]
pub fn definir_turbo(ativo: bool) -> Result<(), String> {
    config::gravar_turbo(ativo, "interface").map_err(|e| e.to_string())
}

#[tauri::command]
pub fn turbo_ativo() -> bool {
    config::ler_turbo().ativo
}

#[tauri::command]
pub fn pausar_automacoes(pausar: bool) -> Result<(), String> {
    config::pausar_automacoes(pausar).map_err(|e| e.to_string())
}

// ---------- Atualização ----------

#[tauri::command]
pub async fn checar_atualizacao(app: AppHandle, estado: State<'_, Estado>) -> Result<InfoAtualizacao, String> {
    let info = atualizador::checar(&app).await?;
    *estado.atualizacao.lock().unwrap_or_else(|e| e.into_inner()) = Some(info.clone());
    Ok(info)
}

#[tauri::command]
pub async fn instalar_atualizacao(app: AppHandle) -> Result<(), String> {
    atualizador::instalar(&app).await
}

// ---------- Janela ----------

#[tauri::command]
pub fn mostrar_janela(app: AppHandle) {
    if let Some(j) = app.get_webview_window("principal") {
        let _ = j.show();
        let _ = j.set_focus();
    }
}
