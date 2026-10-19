//! Tarefas agendadas do Windows (ativar/desativar), via schtasks.exe.
//! O estado é lido do XML da tarefa, que não depende do idioma do Windows.

use crate::comando;
use std::time::Duration;

/// Caminho de tarefa aceitável, ex.: `\Microsoft\Windows\Application Experience\ProgramDataUpdater`.
pub fn caminho_valido(caminho: &str) -> bool {
    caminho.starts_with('\\')
        && caminho.len() < 260
        && !caminho.contains("..")
        && caminho.chars().all(|c| c.is_alphanumeric() || " \\_-.(){}".contains(c))
}

/// A tarefa está ativa? `Ok(None)` = não existe neste PC.
pub fn esta_ativa(caminho: &str) -> Result<Option<bool>, String> {
    if !caminho_valido(caminho) {
        return Err(format!("Caminho de tarefa inválido: {caminho}"));
    }
    let s = comando::executar_sistema("schtasks.exe", &["/Query", "/TN", caminho, "/XML"], Duration::from_secs(30))?;
    if !s.ok() {
        return Ok(None);
    }
    let xml = s.stdout.as_str();
    let trecho = match (xml.find("<Settings>"), xml.find("</Settings>")) {
        (Some(a), Some(b)) if b > a => &xml[a..b],
        _ => xml,
    };
    Ok(Some(!trecho.contains("<Enabled>false</Enabled>")))
}

/// Ativa ou desativa uma tarefa.
pub fn definir_ativa(caminho: &str, ativa: bool) -> Result<(), String> {
    if !caminho_valido(caminho) {
        return Err(format!("Caminho de tarefa inválido: {caminho}"));
    }
    let acao = if ativa { "/ENABLE" } else { "/DISABLE" };
    let s = comando::executar_sistema("schtasks.exe", &["/Change", "/TN", caminho, acao], Duration::from_secs(30))?;
    if s.ok() {
        Ok(())
    } else {
        Err(format!("Não foi possível alterar a tarefa {caminho}: {}", s.erro()))
    }
}
