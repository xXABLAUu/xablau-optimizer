//! Serviços do Windows: leitura do tipo de início (pelo registro) e alteração pela API oficial
//! do Gerenciador de Serviços. No Modo de Segurança, se a API falhar, grava direto no registro.

use crate::registro::{self, ValorReg};
use crate::{sistema, util};
use serde::{Deserialize, Serialize};
use windows_sys::Win32::System::Services::{
    ChangeServiceConfig2W, ChangeServiceConfigW, CloseServiceHandle, OpenSCManagerW, OpenServiceW, QueryServiceStatus,
    SC_MANAGER_CONNECT, SERVICE_AUTO_START, SERVICE_CHANGE_CONFIG, SERVICE_CONFIG_DELAYED_AUTO_START_INFO,
    SERVICE_DELAYED_AUTO_START_INFO, SERVICE_DEMAND_START, SERVICE_DISABLED, SERVICE_NO_CHANGE, SERVICE_QUERY_CONFIG,
    SERVICE_QUERY_STATUS, SERVICE_RUNNING, SERVICE_STATUS,
};

const CHAVE_SERVICOS: &str = r"HKLM\SYSTEM\CurrentControlSet\Services";

/// Tipo de início de um serviço.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InicioServico {
    Boot,
    Sistema,
    Automatico,
    AutomaticoAtrasado,
    Manual,
    Desativado,
}

impl InicioServico {
    pub fn nome(&self) -> &'static str {
        match self {
            InicioServico::Boot => "Inicialização (driver)",
            InicioServico::Sistema => "Sistema (driver)",
            InicioServico::Automatico => "Automático",
            InicioServico::AutomaticoAtrasado => "Automático (atraso)",
            InicioServico::Manual => "Manual",
            InicioServico::Desativado => "Desativado",
        }
    }
}

/// Nome de serviço aceitável (letras, números e `_ - . $`).
pub fn nome_valido(nome: &str) -> bool {
    !nome.is_empty() && nome.len() <= 80 && nome.chars().all(|c| c.is_ascii_alphanumeric() || "_-.$".contains(c))
}

/// O serviço existe neste PC?
pub fn existe(nome: &str) -> bool {
    nome_valido(nome) && registro::chave_existe(&format!("{CHAVE_SERVICOS}\\{nome}"))
}

/// Tipo de início atual. `Ok(None)` = serviço não existe neste PC.
pub fn ler_inicio(nome: &str) -> Result<Option<InicioServico>, String> {
    if !nome_valido(nome) {
        return Err(format!("Nome de serviço inválido: {nome}"));
    }
    let chave = format!("{CHAVE_SERVICOS}\\{nome}");
    let Some(start) = registro::ler_dword(&chave, "Start") else { return Ok(None) };
    let atrasado = registro::ler_dword(&chave, "DelayedAutostart").unwrap_or(0) == 1;
    Ok(Some(match start {
        0 => InicioServico::Boot,
        1 => InicioServico::Sistema,
        2 if atrasado => InicioServico::AutomaticoAtrasado,
        2 => InicioServico::Automatico,
        4 => InicioServico::Desativado,
        _ => InicioServico::Manual,
    }))
}

/// O serviço está rodando agora?
pub fn em_execucao(nome: &str) -> Option<bool> {
    if !nome_valido(nome) {
        return None;
    }
    unsafe {
        let scm = OpenSCManagerW(std::ptr::null(), std::ptr::null(), SC_MANAGER_CONNECT);
        if scm.is_null() {
            return None;
        }
        let n = util::wide(nome);
        let svc = OpenServiceW(scm, n.as_ptr(), SERVICE_QUERY_STATUS);
        if svc.is_null() {
            CloseServiceHandle(scm);
            return None;
        }
        let mut st: SERVICE_STATUS = std::mem::zeroed();
        let ok = QueryServiceStatus(svc, &mut st) != 0;
        CloseServiceHandle(svc);
        CloseServiceHandle(scm);
        ok.then_some(st.dwCurrentState == SERVICE_RUNNING)
    }
}

/// Altera o tipo de início (nunca mexe em drivers de boot/sistema).
pub fn definir_inicio(nome: &str, inicio: InicioServico) -> Result<(), String> {
    if !nome_valido(nome) {
        return Err(format!("Nome de serviço inválido: {nome}"));
    }
    if matches!(inicio, InicioServico::Boot | InicioServico::Sistema) {
        return Err("O Xablau não altera serviços de driver (boot/sistema).".into());
    }
    match definir_via_scm(nome, inicio) {
        Ok(()) => Ok(()),
        Err(e) if sistema::modo_seguranca() => definir_via_registro(nome, inicio).map_err(|e2| format!("{e} / {e2}")),
        Err(e) => Err(e),
    }
}

fn erro_windows(contexto: &str) -> String {
    format!("{contexto}: {}", std::io::Error::last_os_error())
}

fn definir_via_scm(nome: &str, inicio: InicioServico) -> Result<(), String> {
    let tipo = match inicio {
        InicioServico::Automatico | InicioServico::AutomaticoAtrasado => SERVICE_AUTO_START,
        InicioServico::Manual => SERVICE_DEMAND_START,
        InicioServico::Desativado => SERVICE_DISABLED,
        InicioServico::Boot | InicioServico::Sistema => return Err("tipo não suportado".into()),
    };
    unsafe {
        let scm = OpenSCManagerW(std::ptr::null(), std::ptr::null(), SC_MANAGER_CONNECT);
        if scm.is_null() {
            return Err(erro_windows("Não foi possível abrir o Gerenciador de Serviços"));
        }
        let n = util::wide(nome);
        let svc = OpenServiceW(scm, n.as_ptr(), SERVICE_CHANGE_CONFIG | SERVICE_QUERY_CONFIG);
        if svc.is_null() {
            let e = erro_windows(&format!("Não foi possível abrir o serviço {nome}"));
            CloseServiceHandle(scm);
            return Err(e);
        }
        let mut resultado = if ChangeServiceConfigW(
            svc,
            SERVICE_NO_CHANGE,
            tipo,
            SERVICE_NO_CHANGE,
            std::ptr::null(),
            std::ptr::null(),
            std::ptr::null_mut(),
            std::ptr::null(),
            std::ptr::null(),
            std::ptr::null(),
            std::ptr::null(),
        ) == 0
        {
            Err(erro_windows(&format!("Não foi possível alterar o serviço {nome}")))
        } else {
            Ok(())
        };
        if resultado.is_ok() && tipo == SERVICE_AUTO_START {
            let info = SERVICE_DELAYED_AUTO_START_INFO { fDelayedAutostart: i32::from(inicio == InicioServico::AutomaticoAtrasado) };
            if ChangeServiceConfig2W(svc, SERVICE_CONFIG_DELAYED_AUTO_START_INFO, &info as *const _ as *const core::ffi::c_void) == 0 {
                resultado = Err(erro_windows(&format!("Não foi possível ajustar o atraso do serviço {nome}")));
            }
        }
        CloseServiceHandle(svc);
        CloseServiceHandle(scm);
        resultado
    }
}

fn definir_via_registro(nome: &str, inicio: InicioServico) -> Result<(), String> {
    let chave = format!("{CHAVE_SERVICOS}\\{nome}");
    if !registro::chave_existe(&chave) {
        return Err(format!("Serviço {nome} não encontrado."));
    }
    let (start, atrasado) = match inicio {
        InicioServico::Automatico => (2, Some(0)),
        InicioServico::AutomaticoAtrasado => (2, Some(1)),
        InicioServico::Manual => (3, None),
        InicioServico::Desativado => (4, None),
        InicioServico::Boot | InicioServico::Sistema => return Err("tipo não suportado".into()),
    };
    registro::escrever(&chave, "Start", &ValorReg::Dword(start))?;
    if let Some(a) = atrasado {
        registro::escrever(&chave, "DelayedAutostart", &ValorReg::Dword(a))?;
    }
    Ok(())
}
