//! Pastas de dados do Xablau Optimizer em `%ProgramData%\XablauOptimizer`.
//!
//! O diário e os backups ficam SEMPRE aqui — nunca na pasta do programa nem no pendrive
//! (modo portátil). A pasta é protegida: só SYSTEM e Administradores podem gravar, para que
//! um programa sem privilégio não consiga plantar um "desfazer" falso.

use std::io;
use std::path::PathBuf;
use std::sync::Once;
use std::time::Duration;

/// `%ProgramData%\XablauOptimizer`
pub fn base() -> PathBuf {
    let raiz = std::env::var_os("ProgramData").map(PathBuf::from).unwrap_or_else(|| PathBuf::from(r"C:\ProgramData"));
    raiz.join("XablauOptimizer")
}
/// Diário de transações (um arquivo `.jsonl` por operação).
pub fn diario() -> PathBuf {
    base().join("diario")
}
/// Backups granulares (uma pasta por operação, com as chaves `.reg` exportadas).
pub fn backups() -> PathBuf {
    base().join("backups")
}
/// Logs mensais com data e hora de cada alteração.
pub fn logs() -> PathBuf {
    base().join("logs")
}
/// Configurações (modo, perfis, automações).
pub fn config() -> PathBuf {
    base().join("config")
}
/// Estado em tempo de execução trocado entre interface e vigia (turbo, status do vigia).
pub fn estado() -> PathBuf {
    base().join("estado")
}
/// Histórico de desempenho, saúde dos SSDs e medições.
pub fn historico() -> PathBuf {
    base().join("historico")
}
/// Cópia do kit de emergência (usada no modo portátil).
pub fn kit() -> PathBuf {
    base().join("kit")
}
/// Downloads verificados (drivers oficiais).
pub fn downloads() -> PathBuf {
    base().join("downloads")
}
/// Arquivos temporários do próprio programa.
pub fn temporario() -> PathBuf {
    base().join("temp")
}
/// Arquivo principal de configurações.
pub fn arquivo_configuracoes() -> PathBuf {
    config().join("configuracoes.json")
}

static PROTEGER: Once = Once::new();

/// Cria as pastas (se faltarem) e aplica a proteção de permissões uma vez por execução.
pub fn garantir() -> io::Result<()> {
    for p in [base(), diario(), backups(), logs(), config(), estado(), historico(), kit(), downloads(), temporario()] {
        std::fs::create_dir_all(&p)?;
    }
    PROTEGER.call_once(proteger_pasta_base);
    Ok(())
}

fn proteger_pasta_base() {
    if !crate::sistema::e_administrador() {
        return;
    }
    let alvo = base().to_string_lossy().into_owned();
    let icacls = crate::sistema::dir_sistema32().join("icacls.exe");
    let limite = Duration::from_secs(60);
    // Dono: Administradores (S-1-5-32-544).
    let _ = crate::comando::executar(&icacls, &[&alvo, "/setowner", "*S-1-5-32-544", "/C", "/Q"], limite);
    // SYSTEM e Administradores: controle total. Usuários (S-1-5-32-545): só leitura.
    let resultado = crate::comando::executar(
        &icacls,
        &[&alvo, "/inheritance:r", "/grant:r", "*S-1-5-18:(OI)(CI)F", "*S-1-5-32-544:(OI)(CI)F", "*S-1-5-32-545:(OI)(CI)RX", "/C", "/Q"],
        limite,
    );
    match resultado {
        Ok(s) if s.ok() => {}
        Ok(s) => crate::log::aviso("nucleo", &format!("Não foi possível proteger a pasta de dados: {}", s.stderr.trim())),
        Err(e) => crate::log::aviso("nucleo", &format!("Não foi possível proteger a pasta de dados: {e}")),
    }
}
