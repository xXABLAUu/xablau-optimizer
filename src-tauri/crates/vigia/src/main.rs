//! Xablau Vigia — processo leve em segundo plano.
//!
//! Fica na bandeja e, a cada poucos segundos, aplica as automações (perfis por aplicativo, Modo
//! Jogo, Modo Turbo, Watchdog e IdleSaver) respeitando a ordem de prioridade. Não tem janela de
//! interface (sem WebView2), para gastar pouca memória. Uma só instância por sessão (mutex).

// Sem console: o vigia não abre janela preta.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod acoes;
mod bandeja;
mod gpu;
mod motor;

use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver};
use std::time::{Duration, Instant};
use windows_sys::core::PCWSTR;
use windows_sys::Win32::Foundation::{CloseHandle, GetLastError, ERROR_ALREADY_EXISTS, HANDLE};
use windows_sys::Win32::System::Threading::CreateMutexW;
use windows_sys::Win32::UI::Shell::ShellExecuteW;

use acoes::Acao;
use bandeja::{Bandeja, EstadoMenu, ItemMenuPerfil};
use motor::Motor;
use xablau_core::{caminhos, config, log};

const ORIGEM: &str = "vigia";
const INTERVALO: Duration = Duration::from_secs(3);

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// Garante uma só instância do vigia por sessão. Devolve a alça do mutex (mantida viva).
fn instancia_unica() -> Option<HANDLE> {
    let nome = wide("Local\\XablauOptimizerVigia");
    unsafe {
        let h = CreateMutexW(std::ptr::null(), 1, nome.as_ptr());
        if h.is_null() || GetLastError() == ERROR_ALREADY_EXISTS {
            if !h.is_null() {
                CloseHandle(h);
            }
            None
        } else {
            Some(h)
        }
    }
}

fn pasta_do_executavel() -> PathBuf {
    std::env::current_exe().ok().and_then(|p| p.parent().map(PathBuf::from)).unwrap_or_else(|| PathBuf::from("."))
}

fn abrir(caminho: &std::path::Path) {
    let arq = wide(&caminho.to_string_lossy());
    let abrir = wide("open");
    unsafe {
        ShellExecuteW(std::ptr::null_mut(), abrir.as_ptr() as PCWSTR, arq.as_ptr() as PCWSTR, std::ptr::null(), std::ptr::null(), 1);
    }
}

fn abrir_interface() {
    let exe = pasta_do_executavel().join("xablau-optimizer.exe");
    if exe.exists() {
        abrir(&exe);
    } else {
        log::aviso(ORIGEM, "Interface não encontrada ao lado do vigia.");
    }
}

fn abrir_kit() {
    let local = pasta_do_executavel().join("xablau-restaurar.exe");
    let alternativo = caminhos::kit().join("xablau-restaurar.exe");
    if local.exists() {
        abrir(&local);
    } else if alternativo.exists() {
        abrir(&alternativo);
    } else {
        log::aviso(ORIGEM, "Kit de emergência não encontrado.");
    }
}

fn montar_menu(cfg: &config::Configuracoes, st: &config::StatusVigia) -> EstadoMenu {
    EstadoMenu {
        turbo: st.turbo,
        pausado: st.pausado,
        perfis: cfg.perfis.iter().map(|p| ItemMenuPerfil { id: p.id.clone(), nome: p.nome.clone(), ativo: p.ativo }).collect(),
    }
}

fn dica(st: &config::StatusVigia) -> String {
    if st.pausado {
        "Xablau Vigia — pausado".to_string()
    } else {
        format!("Xablau Vigia — {}", st.regra_ativa)
    }
}

fn tratar_acoes(rx: &Receiver<Acao>, motor: &mut Motor, bandeja: &Bandeja) -> bool {
    while let Ok(acao) = rx.try_recv() {
        match acao {
            Acao::AbrirInterface => abrir_interface(),
            Acao::AbrirKit => abrir_kit(),
            Acao::AlternarTurbo => {
                let novo = !config::ler_turbo().ativo;
                let _ = config::gravar_turbo(novo, "bandeja");
                bandeja.notificar("Xablau Optimizer", if novo { "Modo Turbo ligado." } else { "Modo Turbo desligado." });
            }
            Acao::AlternarPausa => {
                let novo = !config::automacoes_pausadas();
                let _ = config::pausar_automacoes(novo);
                bandeja.notificar("Xablau Optimizer", if novo { "Automações pausadas." } else { "Automações retomadas." });
            }
            Acao::AlternarPerfil(id) => {
                let mut cfg = config::carregar();
                if let Some(p) = cfg.perfis.iter_mut().find(|p| p.id == id) {
                    p.ativo = !p.ativo;
                    let _ = config::salvar(&cfg);
                }
            }
            Acao::Encerrar => {
                log::info(ORIGEM, "Encerramento pedido pelo menu da bandeja.");
                motor.encerrar();
                return false;
            }
        }
    }
    true
}

fn main() {
    let Some(mutex) = instancia_unica() else {
        return; // já há um vigia rodando nesta sessão
    };
    if let Err(e) = caminhos::garantir() {
        log::erro(ORIGEM, &format!("Não foi possível preparar as pastas de dados: {e}"));
    }
    log::info(ORIGEM, &format!("Vigia iniciado (versão {}).", xablau_core::VERSAO));

    let (tx, rx) = channel::<Acao>();
    let icone = pasta_do_executavel().join("vigia.ico");
    let icone_txt = if icone.exists() { icone.to_string_lossy().into_owned() } else { String::new() };
    let mut bandeja = match Bandeja::criar(tx, &icone_txt) {
        Ok(b) => b,
        Err(e) => {
            log::erro(ORIGEM, &format!("Não foi possível criar a bandeja: {e}. O vigia continuará sem ícone."));
            // Mesmo sem bandeja, segue aplicando as automações.
            rodar_sem_bandeja(&rx);
            unsafe { CloseHandle(mutex) };
            return;
        }
    };

    let mut motor = Motor::novo();
    let mut proximo = Instant::now();
    loop {
        if !bandeja::bombear_mensagens() {
            motor.encerrar();
            break;
        }
        if !tratar_acoes(&rx, &mut motor, &bandeja) {
            break;
        }
        if Instant::now() >= proximo {
            let cfg = config::carregar();
            let status = motor.ciclo(&cfg);
            bandeja.atualizar(montar_menu(&cfg, &status), &dica(&status));
            // Avisos de reversão por atualização do Windows: notifica uma vez.
            if cfg.automacao.avisar_reversoes {
                avisar_reversoes(&bandeja);
            }
            proximo = Instant::now() + INTERVALO;
        }
        std::thread::sleep(Duration::from_millis(150));
    }

    drop(bandeja);
    unsafe { CloseHandle(mutex) };
    log::info(ORIGEM, "Vigia finalizado.");
}

fn rodar_sem_bandeja(rx: &Receiver<Acao>) {
    let mut motor = Motor::novo();
    loop {
        if let Ok(Acao::Encerrar) = rx.try_recv() {
            motor.encerrar();
            return;
        }
        let cfg = config::carregar();
        motor.ciclo(&cfg);
        std::thread::sleep(INTERVALO);
    }
}

use std::sync::atomic::{AtomicU64, Ordering};
static ULTIMO_AVISO_REVERSAO: AtomicU64 = AtomicU64::new(0);

fn avisar_reversoes(bandeja: &Bandeja) {
    let revertidas = xablau_core::diario::alteracoes_revertidas();
    if revertidas.is_empty() {
        return;
    }
    let agora = chrono::Local::now().timestamp() as u64;
    let ultimo = ULTIMO_AVISO_REVERSAO.load(Ordering::Relaxed);
    // Avisa no máximo uma vez a cada 6 horas.
    if agora.saturating_sub(ultimo) < 6 * 3600 {
        return;
    }
    ULTIMO_AVISO_REVERSAO.store(agora, Ordering::Relaxed);
    bandeja.notificar(
        "Xablau Optimizer",
        &format!("O Windows desfez {} ajuste(s). Abra o programa para revê-los.", revertidas.len()),
    );
}
