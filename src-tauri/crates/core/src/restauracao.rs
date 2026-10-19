//! Ponto de restauração do sistema (System Restore).
//!
//! Antes de qualquer alteração: liga a Proteção do Sistema se estiver desligada, contorna o limite
//! de "um ponto a cada 24 h" (chave SystemRestorePointCreationFrequency) e confirma que o ponto
//! foi mesmo criado. Se não der para garantir o ponto, NADA é aplicado (quem chama trata o erro).
//!
//! No Modo de Segurança não é possível criar pontos — isso é detectado e informado.

use crate::registro::{self, ValorReg};
use crate::{comando, log, sistema};
use serde::Serialize;
use std::time::Duration;

const ORIGEM: &str = "restauracao";
const CHAVE_SR: &str = r"HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\SystemRestore";

/// Resultado da tentativa de criar o ponto de restauração.
#[derive(Debug, Clone, Serialize)]
pub struct ResultadoPonto {
    pub criado: bool,
    pub numero_sequencia: Option<i64>,
    pub descricao: String,
    pub mensagem: String,
    /// Proteção do Sistema foi ligada agora por nós (para avisar o usuário).
    pub protecao_ligada_agora: bool,
}

/// A unidade do sistema (ex.: "C:").
fn unidade_sistema() -> String {
    std::env::var("SystemDrive").unwrap_or_else(|_| "C:".into())
}

/// A Proteção do Sistema está ligada na unidade do sistema?
pub fn protecao_ativa() -> bool {
    // RPSessionInterval > 0 costuma indicar proteção ativa; confirmamos via WMI quando possível.
    let s = comando::powershell(
        "try{ $d=(Get-CimInstance -Namespace root/default -ClassName SystemRestore -ErrorAction Stop); if($d){'ativa'}else{'inativa'} }catch{ 'inativa' }",
        &[],
        Duration::from_secs(30),
    );
    matches!(s, Ok(ref r) if r.stdout.trim() == "ativa")
}

/// Liga a Proteção do Sistema na unidade do sistema.
fn ligar_protecao() -> Result<(), String> {
    let unidade = format!("{}\\", unidade_sistema());
    let s = comando::powershell("Enable-ComputerRestore -Drive $env:XB_UNIDADE", &[("XB_UNIDADE", &unidade)], Duration::from_secs(120))?;
    if s.ok() {
        log::info(ORIGEM, "Proteção do Sistema ligada.");
        Ok(())
    } else {
        Err(format!("Não foi possível ligar a Proteção do Sistema: {}", s.erro()))
    }
}

/// Lê a frequência mínima entre pontos (minutos). `None` = padrão do Windows (1440 = 24 h).
fn frequencia_atual() -> Option<u32> {
    registro::ler_dword(CHAVE_SR, "SystemRestorePointCreationFrequency")
}

/// Define a frequência como 0 (permite criar ponto mesmo tendo criado um nas últimas 24 h).
fn zerar_frequencia() -> Result<Option<u32>, String> {
    let antes = frequencia_atual();
    registro::escrever(CHAVE_SR, "SystemRestorePointCreationFrequency", &ValorReg::Dword(0))?;
    Ok(antes)
}

/// Restaura a frequência ao valor anterior (ou remove a chave se não existia).
fn restaurar_frequencia(antes: Option<u32>) {
    let _ = match antes {
        Some(v) => registro::escrever(CHAVE_SR, "SystemRestorePointCreationFrequency", &ValorReg::Dword(v)),
        None => registro::apagar_valor(CHAVE_SR, "SystemRestorePointCreationFrequency"),
    };
}

/// Último número de sequência de ponto de restauração (para confirmar que um novo foi criado).
fn ultima_sequencia() -> Option<i64> {
    let s = comando::powershell(
        "try{ $p=Get-ComputerRestorePoint -ErrorAction Stop | Sort-Object SequenceNumber | Select-Object -Last 1; if($p){[string]$p.SequenceNumber}else{'0'} }catch{ '' }",
        &[],
        Duration::from_secs(60),
    )
    .ok()?;
    let t = s.stdout.trim();
    if t.is_empty() { None } else { t.parse::<i64>().ok() }
}

/// Cria um ponto de restauração com todas as garantias. Se não conseguir confirmar, `criado=false`
/// e o chamador NÃO deve aplicar nada.
pub fn criar(descricao: &str) -> ResultadoPonto {
    let desc = descricao.chars().filter(|c| *c != '\r' && *c != '\n' && *c != '"').take(64).collect::<String>();
    let mut resultado = ResultadoPonto {
        criado: false,
        numero_sequencia: None,
        descricao: desc.clone(),
        mensagem: String::new(),
        protecao_ligada_agora: false,
    };

    if sistema::modo_seguranca() {
        resultado.mensagem =
            "No Modo de Segurança não é possível criar ponto de restauração. Prefira rodar fora do Modo de Segurança, ou use o kit de emergência para desfazer.".into();
        return resultado;
    }

    // 1) Garante a Proteção do Sistema ligada.
    if !protecao_ativa() {
        match ligar_protecao() {
            Ok(()) => resultado.protecao_ligada_agora = true,
            Err(e) => {
                resultado.mensagem = e;
                return resultado;
            }
        }
    }

    // 2) Contorna o limite de 24 h e guarda a sequência anterior.
    let freq_antes = match zerar_frequencia() {
        Ok(v) => v,
        Err(e) => {
            resultado.mensagem = format!("Não foi possível ajustar a frequência de pontos: {e}");
            return resultado;
        }
    };
    let seq_antes = ultima_sequencia();

    // 3) Cria o ponto (MODIFY_SETTINGS = 12).
    let criacao = comando::powershell(
        "Checkpoint-Computer -Description $env:XB_DESC -RestorePointType MODIFY_SETTINGS",
        &[("XB_DESC", &desc)],
        Duration::from_secs(300),
    );
    restaurar_frequencia(freq_antes);

    match criacao {
        Ok(s) if s.ok() => {}
        Ok(s) => {
            // Pode ter falhado por limite de tempo mesmo com frequência 0 — confirmamos pela sequência.
            log::aviso(ORIGEM, &format!("Checkpoint-Computer retornou aviso: {}", s.erro()));
        }
        Err(e) => {
            resultado.mensagem = format!("Falha ao criar o ponto de restauração: {e}");
            return resultado;
        }
    }

    // 4) Confirma que um ponto NOVO foi criado comparando a sequência.
    let seq_depois = ultima_sequencia();
    let confirmado = match (seq_antes, seq_depois) {
        (Some(a), Some(d)) => d > a,
        (None, Some(_)) => true,
        _ => false,
    };

    if confirmado {
        resultado.criado = true;
        resultado.numero_sequencia = seq_depois;
        resultado.mensagem = "Ponto de restauração criado e confirmado.".into();
        log::info(ORIGEM, &format!("Ponto de restauração criado: \"{desc}\" (sequência {:?})", seq_depois));
    } else {
        resultado.mensagem =
            "Não foi possível confirmar a criação do ponto de restauração. Por segurança, nenhuma alteração será aplicada.".into();
        log::erro(ORIGEM, &resultado.mensagem);
    }
    resultado
}
