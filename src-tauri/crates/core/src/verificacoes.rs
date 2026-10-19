//! Verificações prévias (feitas antes de aplicar qualquer ajuste): reinicialização pendente,
//! Windows Update instalando, espaço em disco, versão do Windows, notebook vs desktop e bateria.

use crate::registro;
use crate::sistema::{self, StatusEnergia, VersaoWindows};
use serde::Serialize;
use windows_sys::Win32::Storage::FileSystem::GetDiskFreeSpaceExW;

/// Nível de uma verificação.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Nivel {
    /// Tudo certo.
    Ok,
    /// Dá para seguir, mas convém avisar.
    Aviso,
    /// Melhor não aplicar agora (ex.: Windows Update instalando).
    Bloqueio,
}

#[derive(Debug, Clone, Serialize)]
pub struct Verificacao {
    pub chave: String,
    pub titulo: String,
    pub nivel: Nivel,
    pub detalhe: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Relatorio {
    pub itens: Vec<Verificacao>,
    pub versao_windows: VersaoWindows,
    pub energia: StatusEnergia,
    pub administrador: bool,
    pub modo_seguranca: bool,
    pub espaco_livre_gb: f64,
    /// Há algum item de bloqueio?
    pub pode_prosseguir: bool,
}

fn v(chave: &str, titulo: &str, nivel: Nivel, detalhe: impl Into<String>) -> Verificacao {
    Verificacao { chave: chave.into(), titulo: titulo.into(), nivel, detalhe: detalhe.into() }
}

/// Reinicialização pendente (de atualização, instalação ou renomeação de arquivos).
pub fn reinicializacao_pendente() -> bool {
    registro::chave_existe(r"HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\WindowsUpdate\Auto Update\RebootRequired")
        || registro::chave_existe(r"HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Component Based Servicing\RebootPending")
        || registro::ler(r"HKLM\SYSTEM\CurrentControlSet\Control\Session Manager", "PendingFileRenameOperations").ok().flatten().is_some()
}

/// Windows Update instalando agora (processos do instalador ativos).
pub fn windows_update_instalando(processos: &[crate::processos::AmostraProcesso]) -> bool {
    const INSTALADORES: &[&str] = &["TiWorker.exe", "TrustedInstaller.exe", "wuauclt.exe"];
    processos.iter().any(|p| INSTALADORES.iter().any(|n| p.nome.eq_ignore_ascii_case(n)))
}

/// Espaço livre (bytes) na unidade do sistema.
pub fn espaco_livre_sistema() -> u64 {
    let unidade = std::env::var("SystemDrive").unwrap_or_else(|_| "C:".into());
    let caminho: Vec<u16> = format!("{unidade}\\").encode_utf16().chain(std::iter::once(0)).collect();
    let mut livre: u64 = 0;
    let ok = unsafe { GetDiskFreeSpaceExW(caminho.as_ptr(), &mut livre, std::ptr::null_mut(), std::ptr::null_mut()) };
    if ok == 0 { 0 } else { livre }
}

/// Faz todas as verificações. `espaco_minimo_gb` costuma ser 2 GB (precisa de espaço para o ponto de restauração).
pub fn verificar(espaco_minimo_gb: f64) -> Relatorio {
    let processos = crate::processos::instantaneo();
    let versao = sistema::versao_windows();
    let energia = sistema::status_energia();
    let administrador = sistema::e_administrador();
    let modo_seguranca = sistema::modo_seguranca();
    let livre = espaco_livre_sistema();
    let livre_gb = livre as f64 / 1_073_741_824.0;

    let mut itens = Vec::new();

    if administrador {
        itens.push(v("admin", "Permissão de administrador", Nivel::Ok, "O programa está rodando como administrador."));
    } else {
        itens.push(v("admin", "Permissão de administrador", Nivel::Bloqueio, "Abra o programa como administrador para aplicar ajustes."));
    }

    if versao.windows11 {
        itens.push(v("windows", "Versão do Windows", Nivel::Ok, format!("Windows 11 (build {}).", versao.build)));
    } else if versao.major == 10 {
        itens.push(v("windows", "Versão do Windows", Nivel::Ok, format!("Windows 10 (build {}).", versao.build)));
    } else {
        itens.push(v("windows", "Versão do Windows", Nivel::Aviso, "Versão do Windows mais antiga que a recomendada; alguns ajustes podem não existir."));
    }

    if reinicializacao_pendente() {
        itens.push(v("reinicio", "Reinicialização pendente", Nivel::Aviso, "Há uma reinicialização pendente. Reinicie antes para evitar resultados inesperados."));
    } else {
        itens.push(v("reinicio", "Reinicialização pendente", Nivel::Ok, "Nenhuma reinicialização pendente."));
    }

    if windows_update_instalando(&processos) {
        itens.push(v("update", "Windows Update", Nivel::Bloqueio, "O Windows Update está instalando algo agora. Aguarde terminar antes de otimizar."));
    } else {
        itens.push(v("update", "Windows Update", Nivel::Ok, "O Windows Update não está instalando nada no momento."));
    }

    if livre_gb >= espaco_minimo_gb {
        itens.push(v("disco", "Espaço em disco", Nivel::Ok, format!("{livre_gb:.1} GB livres na unidade do sistema.")));
    } else {
        itens.push(v("disco", "Espaço em disco", Nivel::Bloqueio, format!("Só {livre_gb:.1} GB livres. São necessários pelo menos {espaco_minimo_gb:.0} GB (inclusive para o ponto de restauração).")));
    }

    if energia.na_bateria() {
        let pct = energia.bateria_pct.map(|p| format!(" ({p}%)")).unwrap_or_default();
        itens.push(v("energia", "Energia", Nivel::Aviso, format!("Notebook na bateria{pct}. Desempenho Máximo não será aplicado enquanto estiver na bateria.")));
    } else if energia.tem_bateria {
        itens.push(v("energia", "Energia", Nivel::Ok, "Notebook na tomada."));
    } else {
        itens.push(v("energia", "Energia", Nivel::Ok, "Computador de mesa (desktop)."));
    }

    if modo_seguranca {
        itens.push(v("seguranca", "Modo de Segurança", Nivel::Aviso, "No Modo de Segurança não é possível criar ponto de restauração. Prefira rodar o Windows normalmente."));
    }

    let pode_prosseguir = !itens.iter().any(|i| i.nivel == Nivel::Bloqueio);
    Relatorio { itens, versao_windows: versao, energia, administrador, modo_seguranca, espaco_livre_gb: livre_gb, pode_prosseguir }
}
