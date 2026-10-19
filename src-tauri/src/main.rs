//! Xablau Optimizer — processo da interface (Tauri).
//!
//! Abre a janela (React/WebView2), expõe a lista fechada de comandos validados e inicia o vigia
//! em segundo plano quando a configuração pede. Pede administrador pelo manifesto.
//! Criado por Xablau Studios — Luiz.

// Sem console em release.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod atualizador;
mod catalogo;
mod comandos;
mod diagnostico;
mod estado;

use std::sync::Mutex;
use tauri::{Emitter, Manager};
use xablau_core::{log, sistema};

const ORIGEM: &str = "interface";

fn main() {
    if let Err(e) = xablau_core::caminhos::garantir() {
        eprintln!("Aviso: não foi possível preparar as pastas de dados: {e}");
    }
    log::info(ORIGEM, &format!("Interface iniciada (versão {}).", xablau_core::VERSAO));

    let estado = match estado::Estado::novo() {
        Ok(e) => e,
        Err(e) => {
            // Catálogo inválido é um erro de desenvolvimento; registra e segue com erro visível.
            log::erro(ORIGEM, &format!("Falha ao iniciar: {e}"));
            eprintln!("Falha ao iniciar o Xablau Optimizer: {e}");
            std::process::exit(1);
        }
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // Já há uma instância: traz a janela existente para frente.
            if let Some(j) = app.get_webview_window("principal") {
                let _ = j.show();
                let _ = j.set_focus();
            }
        }))
        .manage(estado)
        .manage(comandos::AmostradorCompartilhado(Mutex::new(sistema::AmostradorCpu::novo())))
        .setup(|app| {
            let handle = app.handle().clone();
            // Avisa a interface sobre operações interrompidas (recuperação) ao abrir.
            let interrompidas = xablau_core::diario::operacoes_incompletas();
            if !interrompidas.is_empty() {
                let _ = handle.emit("operacoes-interrompidas", interrompidas.len());
                log::aviso(ORIGEM, &format!("{} operação(ões) interrompida(s) detectada(s) ao abrir.", interrompidas.len()));
            }
            // Inicia o vigia, se a configuração pedir e ele ainda não estiver rodando.
            iniciar_vigia_se_preciso();
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            comandos::sobre,
            comandos::obter_hardware,
            comandos::obter_topologia,
            comandos::uso_tempo_real,
            comandos::monitor_culpados,
            comandos::saude_ssds,
            comandos::verificacoes_previas,
            comandos::ficha_pc,
            comandos::listar_ajustes,
            comandos::simular,
            comandos::aplicar_ajustes,
            comandos::criar_ponto_restauracao,
            comandos::listar_operacoes,
            comandos::operacoes_interrompidas,
            comandos::desfazer_tudo,
            comandos::desfazer_operacao,
            comandos::desfazer_item,
            comandos::conflitos_do_item,
            comandos::encerrar_interrompida,
            comandos::alteracoes_revertidas,
            comandos::log_recente,
            comandos::exportar_log,
            comandos::carregar_config,
            comandos::salvar_config,
            comandos::status_vigia,
            comandos::definir_turbo,
            comandos::turbo_ativo,
            comandos::pausar_automacoes,
            comandos::checar_atualizacao,
            comandos::instalar_atualizacao,
            comandos::mostrar_janela,
        ])
        .run(tauri::generate_context!())
        .expect("erro ao iniciar o Xablau Optimizer");
}

/// Inicia o vigia ao lado do executável, se a configuração pedir.
fn iniciar_vigia_se_preciso() {
    let cfg = xablau_core::config::carregar();
    if !cfg.automacao.vigia_ativo {
        return;
    }
    let exe = match std::env::current_exe().ok().and_then(|p| p.parent().map(|d| d.join("xablau-vigia.exe"))) {
        Some(p) if p.exists() => p,
        _ => return,
    };
    // O vigia tem instância única (mutex), então chamar mais de uma vez é seguro.
    use std::os::windows::process::CommandExt;
    let _ = std::process::Command::new(exe).creation_flags(0x0800_0000).spawn();
}
