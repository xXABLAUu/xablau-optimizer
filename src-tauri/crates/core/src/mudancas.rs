//! Mudanças atômicas no sistema. Cada uma guarda o estado ANTES e DEPOIS, sabe se aplicar e
//! se desfazer. É isso que vai para o diário de transações — e é isso que o kit de emergência lê.
//!
//! Desfazer é idempotente: restaurar o "antes" é seguro mesmo se a mudança nem chegou a ser
//! aplicada (por exemplo, quando o PC desligou no meio da operação).

use crate::registro::{self, ValorReg};
use crate::servicos::{self, InicioServico};
use crate::{comando, energia, protecao, sistema, tarefas};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Uma mudança atômica no sistema.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "tipo", rename_all = "snake_case")]
pub enum Mudanca {
    /// Valor do registro. `antes`/`depois` = `None` significa "valor não existe".
    Registro {
        chave: String,
        nome: String,
        antes: Option<ValorReg>,
        depois: Option<ValorReg>,
        /// Chave mais alta criada pelo Xablau (apagada ao desfazer, se ficar vazia).
        #[serde(default)]
        chave_criada: Option<String>,
    },
    /// Tipo de início de um serviço.
    Servico { nome: String, antes: InicioServico, depois: InicioServico },
    /// Valor de uma configuração de energia (CA = na tomada; CC = bateria).
    Energia { esquema: String, subgrupo: String, configuracao: String, ca: bool, antes: u32, depois: u32 },
    /// Plano de energia ativo. `criado` = plano criado pelo Xablau (apagado ao desfazer).
    PlanoEnergia {
        antes: String,
        depois: String,
        #[serde(default)]
        criado: Option<String>,
    },
    /// Tarefa agendada ativa/inativa.
    Tarefa { caminho: String, antes: bool, depois: bool },
    /// Arquivo de texto do sistema (ex.: hosts), com cópia do antes e do depois.
    Arquivo {
        caminho: String,
        #[serde(default)]
        backup: Option<String>,
        novo: String,
        existia: bool,
    },
    /// Hibernação ligada/desligada.
    Hibernacao { antes: bool, depois: bool },
    /// Recurso opcional do Windows (Hyper-V, WSL, Sandbox...).
    Recurso { nome: String, antes: bool, depois: bool },
    /// Servidores DNS de um adaptador (vazio = automático/DHCP).
    Dns { indice: u32, guid: String, alias: String, antes_v4: String, antes_v6: String, depois: Vec<String> },
    /// App pré-instalado removido (desfazer tenta registrar de novo; senão, orienta pela Store).
    Appx { nome: String, familia: String, todos_usuarios: bool },
    /// Pasta renomeada como backup (ex.: SoftwareDistribution no reparo do Windows Update).
    PastaRenomeada { de: String, para: String },
    /// Arquivo/pasta enviado para a Lixeira (restaurar pela própria Lixeira).
    Lixeira { caminho: String },
}

fn texto_opcional(v: &Option<ValorReg>) -> String {
    match v {
        Some(v) => v.para_texto(),
        None => "(não existe)".to_string(),
    }
}

fn liga(b: bool) -> &'static str {
    if b { "ligado" } else { "desligado" }
}

fn nome_plano(guid: &str) -> String {
    energia::listar().into_iter().find(|p| p.guid.eq_ignore_ascii_case(guid)).map(|p| p.nome).unwrap_or_else(|| guid.to_string())
}

impl Mudanca {
    /// Identidade do alvo (para detectar conflitos e reversões).
    pub fn alvo(&self) -> String {
        match self {
            Mudanca::Registro { chave, nome, .. } => format!("reg:{}\\{}", registro::normalizar(chave).to_lowercase(), nome.to_lowercase()),
            Mudanca::Servico { nome, .. } => format!("svc:{}", nome.to_lowercase()),
            Mudanca::Energia { esquema, subgrupo, configuracao, ca, .. } => format!("pwr:{esquema}/{subgrupo}/{configuracao}/{ca}"),
            Mudanca::PlanoEnergia { .. } => "plano".into(),
            Mudanca::Tarefa { caminho, .. } => format!("task:{}", caminho.to_lowercase()),
            Mudanca::Arquivo { caminho, .. } => format!("file:{}", caminho.to_lowercase()),
            Mudanca::Hibernacao { .. } => "hibernacao".into(),
            Mudanca::Recurso { nome, .. } => format!("feat:{}", nome.to_lowercase()),
            Mudanca::Dns { guid, .. } => format!("dns:{}", guid.to_lowercase()),
            Mudanca::Appx { nome, .. } => format!("appx:{}", nome.to_lowercase()),
            Mudanca::PastaRenomeada { de, .. } => format!("dir:{}", de.to_lowercase()),
            Mudanca::Lixeira { caminho } => format!("lixeira:{}", caminho.to_lowercase()),
        }
    }

    /// Descrição legível em português (aparece na simulação, no histórico e no kit).
    pub fn descrever(&self) -> String {
        match self {
            Mudanca::Registro { chave, nome, antes, depois, .. } => {
                let n = if nome.is_empty() { "(Padrão)" } else { nome.as_str() };
                format!("Registro {}\\{}: {} → {}", registro::normalizar(chave), n, texto_opcional(antes), texto_opcional(depois))
            }
            Mudanca::Servico { nome, antes, depois } => format!("Serviço {nome}: {} → {}", antes.nome(), depois.nome()),
            Mudanca::Energia { configuracao, ca, antes, depois, .. } => {
                format!("Energia ({}) {configuracao}: {antes} → {depois}", if *ca { "na tomada" } else { "na bateria" })
            }
            Mudanca::PlanoEnergia { antes, depois, .. } => format!("Plano de energia: {} → {}", nome_plano(antes), nome_plano(depois)),
            Mudanca::Tarefa { caminho, antes, depois } => {
                format!("Tarefa {caminho}: {} → {}", if *antes { "ativa" } else { "desativada" }, if *depois { "ativa" } else { "desativada" })
            }
            Mudanca::Arquivo { caminho, .. } => format!("Arquivo {caminho} (com cópia de segurança)"),
            Mudanca::Hibernacao { antes, depois } => format!("Hibernação: {} → {}", liga(*antes), liga(*depois)),
            Mudanca::Recurso { nome, antes, depois } => format!("Recurso do Windows {nome}: {} → {}", liga(*antes), liga(*depois)),
            Mudanca::Dns { alias, antes_v4, depois, .. } => {
                let a = if antes_v4.is_empty() { "automático".to_string() } else { antes_v4.clone() };
                let d = if depois.is_empty() { "automático".to_string() } else { depois.join(", ") };
                format!("DNS do adaptador {alias}: {a} → {d}")
            }
            Mudanca::Appx { nome, todos_usuarios, .. } => {
                format!("Remover o app {nome}{}", if *todos_usuarios { " (todos os usuários)" } else { "" })
            }
            Mudanca::PastaRenomeada { de, para } => format!("Pasta {de} guardada como {para}"),
            Mudanca::Lixeira { caminho } => format!("{caminho} enviado para a Lixeira"),
        }
    }

    /// Dá para desfazer automaticamente?
    pub fn reversivel(&self) -> bool {
        !matches!(self, Mudanca::Lixeira { .. })
    }

    /// Desfazer de novo seria inofensivo? (usado para tentar desfazer até mudanças que falharam)
    pub fn desfazer_e_idempotente(&self) -> bool {
        !matches!(self, Mudanca::Appx { .. } | Mudanca::PastaRenomeada { .. } | Mudanca::Lixeira { .. })
    }

    /// Aplica a mudança (estado "depois").
    pub fn aplicar(&self) -> Result<(), String> {
        match self {
            Mudanca::Registro { chave, nome, depois, .. } => match depois {
                Some(v) => registro::escrever(chave, nome, v),
                None => registro::apagar_valor(chave, nome),
            },
            Mudanca::Servico { nome, depois, .. } => {
                if protecao::servico_protegido(nome) {
                    return Err(format!("O serviço {nome} está na lista de proteção."));
                }
                servicos::definir_inicio(nome, *depois)
            }
            Mudanca::Energia { esquema, subgrupo, configuracao, ca, depois, .. } => {
                energia::escrever_valor(esquema, subgrupo, configuracao, *ca, *depois)
            }
            Mudanca::PlanoEnergia { depois, .. } => energia::ativar(depois),
            Mudanca::Tarefa { caminho, depois, .. } => tarefas::definir_ativa(caminho, *depois),
            Mudanca::Arquivo { caminho, novo, .. } => copiar_arquivo(Path::new(novo), Path::new(caminho)),
            Mudanca::Hibernacao { depois, .. } => definir_hibernacao(*depois),
            Mudanca::Recurso { nome, depois, .. } => dism_recurso(nome, *depois),
            Mudanca::Dns { indice, depois, .. } => definir_dns(*indice, depois),
            Mudanca::Appx { nome, todos_usuarios, .. } => remover_appx(nome, *todos_usuarios),
            Mudanca::PastaRenomeada { de, para } => std::fs::rename(de, para).map_err(|e| format!("Não foi possível renomear {de}: {e}")),
            Mudanca::Lixeira { .. } => Err("Envio para a Lixeira é feito pela interface.".into()),
        }
    }

    /// Desfaz a mudança (volta ao estado "antes").
    pub fn desfazer(&self) -> Result<(), String> {
        match self {
            Mudanca::Registro { chave, nome, antes, chave_criada, .. } => {
                match antes {
                    Some(v) => registro::escrever(chave, nome, v)?,
                    None => registro::apagar_valor(chave, nome)?,
                }
                if let Some(topo) = chave_criada {
                    registro::apagar_chaves_vazias(chave, topo)?;
                }
                Ok(())
            }
            Mudanca::Servico { nome, antes, .. } => {
                if matches!(antes, InicioServico::Boot | InicioServico::Sistema) {
                    return Ok(());
                }
                servicos::definir_inicio(nome, *antes)
            }
            Mudanca::Energia { esquema, subgrupo, configuracao, ca, antes, .. } => {
                energia::escrever_valor(esquema, subgrupo, configuracao, *ca, *antes)
            }
            Mudanca::PlanoEnergia { antes, criado, .. } => {
                energia::ativar(antes)?;
                if let Some(c) = criado {
                    if !c.eq_ignore_ascii_case(antes) && energia::existe(c) {
                        let _ = energia::apagar(c);
                    }
                }
                Ok(())
            }
            Mudanca::Tarefa { caminho, antes, .. } => tarefas::definir_ativa(caminho, *antes),
            Mudanca::Arquivo { caminho, backup, existia, .. } => {
                if *existia {
                    let b = backup.as_ref().ok_or("A cópia de segurança do arquivo não foi encontrada.")?;
                    copiar_arquivo(Path::new(b), Path::new(caminho))
                } else if Path::new(caminho).exists() {
                    std::fs::remove_file(caminho).map_err(|e| format!("Não foi possível apagar {caminho}: {e}"))
                } else {
                    Ok(())
                }
            }
            Mudanca::Hibernacao { antes, .. } => definir_hibernacao(*antes),
            Mudanca::Recurso { nome, antes, .. } => dism_recurso(nome, *antes),
            Mudanca::Dns { indice, antes_v4, antes_v6, .. } => {
                let lista: Vec<String> = antes_v4
                    .split([',', ' ', ';'])
                    .chain(antes_v6.split([',', ' ', ';']))
                    .filter(|s| !s.trim().is_empty())
                    .map(|s| s.trim().to_string())
                    .collect();
                definir_dns(*indice, &lista)
            }
            Mudanca::Appx { nome, .. } => reinstalar_appx(nome),
            Mudanca::PastaRenomeada { de, para } => {
                if !Path::new(para).exists() {
                    return Err(format!("A pasta de backup {para} não existe mais."));
                }
                if Path::new(de).exists() {
                    let descartada = format!("{de}.xablau-descartado-{}", chrono::Local::now().format("%Y%m%d%H%M%S"));
                    std::fs::rename(de, &descartada).map_err(|e| format!("Não foi possível liberar {de}: {e}"))?;
                }
                std::fs::rename(para, de).map_err(|e| format!("Não foi possível restaurar {de}: {e}"))
            }
            Mudanca::Lixeira { caminho } => Err(format!("Restaure \"{caminho}\" pela Lixeira do Windows (botão Restaurar).")),
        }
    }

    /// O sistema ainda está no estado "depois"? `None` = não dá para saber.
    /// Serve para avisar quando uma atualização do Windows desfez um ajuste.
    pub fn esta_aplicada(&self) -> Option<bool> {
        match self {
            Mudanca::Registro { chave, nome, depois, .. } => registro::ler(chave, nome).ok().map(|atual| atual == *depois),
            Mudanca::Servico { nome, depois, .. } => servicos::ler_inicio(nome).ok().flatten().map(|a| a == *depois),
            Mudanca::Energia { esquema, subgrupo, configuracao, ca, depois, .. } => {
                energia::ler_valor(esquema, subgrupo, configuracao, *ca).ok().map(|v| v == *depois)
            }
            Mudanca::Tarefa { caminho, depois, .. } => tarefas::esta_ativa(caminho).ok().flatten().map(|a| a == *depois),
            Mudanca::Hibernacao { depois, .. } => Some(hibernacao_ativa() == *depois),
            _ => None,
        }
    }
}

fn copiar_arquivo(origem: &Path, destino: &Path) -> Result<(), String> {
    let dados = std::fs::read(origem).map_err(|e| format!("Não foi possível ler {}: {e}", origem.display()))?;
    crate::util::escrever_atomico(destino, &dados).map_err(|e| format!("Não foi possível gravar {}: {e}", destino.display()))
}

/// A hibernação está ligada?
pub fn hibernacao_ativa() -> bool {
    match registro::ler_dword(r"HKLM\SYSTEM\CurrentControlSet\Control\Power", "HibernateEnabled") {
        Some(v) => v != 0,
        None => {
            let unidade = std::env::var("SystemDrive").unwrap_or_else(|_| "C:".into());
            PathBuf::from(format!("{unidade}\\hiberfil.sys")).exists()
        }
    }
}

fn definir_hibernacao(ligar: bool) -> Result<(), String> {
    let s = comando::executar_sistema("powercfg.exe", &["/hibernate", if ligar { "on" } else { "off" }], Duration::from_secs(120))?;
    if s.ok() {
        Ok(())
    } else {
        Err(format!("powercfg /hibernate falhou: {}", s.erro()))
    }
}

fn nome_recurso_valido(nome: &str) -> bool {
    !nome.is_empty() && nome.len() <= 100 && nome.chars().all(|c| c.is_ascii_alphanumeric() || "._-".contains(c))
}

/// O recurso opcional está ativo?
pub fn recurso_ativo(nome: &str) -> Result<bool, String> {
    if !nome_recurso_valido(nome) {
        return Err(format!("Nome de recurso inválido: {nome}"));
    }
    let s = comando::powershell(
        "$f=Get-WindowsOptionalFeature -Online -FeatureName $env:XB_RECURSO; if($f){[string]$f.State}else{'Ausente'}",
        &[("XB_RECURSO", nome)],
        Duration::from_secs(180),
    )?;
    match s.stdout.trim() {
        "Enabled" | "EnablePending" => Ok(true),
        "Ausente" => Err(format!("O recurso {nome} não existe neste Windows.")),
        "" => Err(format!("Não foi possível consultar o recurso {nome}: {}", s.erro())),
        _ => Ok(false),
    }
}

fn dism_recurso(nome: &str, ativar: bool) -> Result<(), String> {
    if !nome_recurso_valido(nome) {
        return Err(format!("Nome de recurso inválido: {nome}"));
    }
    let arg = format!("/FeatureName:{nome}");
    let mut args = vec!["/Online", if ativar { "/Enable-Feature" } else { "/Disable-Feature" }, arg.as_str(), "/NoRestart", "/Quiet"];
    if ativar {
        args.push("/All");
    }
    let s = comando::executar_sistema("Dism.exe", &args, Duration::from_secs(3600))?;
    match s.codigo {
        0 | 3010 => Ok(()),
        c => Err(format!("O DISM terminou com o código {c}.")),
    }
}

/// Endereço IP válido (v4 ou v6)?
pub fn ip_valido(ip: &str) -> bool {
    ip.trim().parse::<std::net::IpAddr>().is_ok()
}

fn definir_dns(indice: u32, servidores: &[String]) -> Result<(), String> {
    if servidores.iter().any(|s| !ip_valido(s)) {
        return Err("Endereço de DNS inválido.".into());
    }
    let indice_txt = indice.to_string();
    let lista = servidores.join(",");
    let script = if servidores.is_empty() {
        "Set-DnsClientServerAddress -InterfaceIndex ([int]$env:XB_IF) -ResetServerAddresses"
    } else {
        "Set-DnsClientServerAddress -InterfaceIndex ([int]$env:XB_IF) -ServerAddresses ($env:XB_DNS -split ',')"
    };
    let s = comando::powershell(script, &[("XB_IF", &indice_txt), ("XB_DNS", &lista)], Duration::from_secs(60))?;
    if s.ok() {
        Ok(())
    } else {
        Err(format!("Não foi possível alterar o DNS: {}", s.erro()))
    }
}

fn remover_appx(nome: &str, todos: bool) -> Result<(), String> {
    if !protecao::nome_appx_valido(nome) || protecao::appx_protegido(nome) {
        return Err(format!("O app {nome} está protegido ou tem nome inválido."));
    }
    let script = if todos {
        "Get-AppxPackage -AllUsers -Name $env:XB_APPX | Remove-AppxPackage -AllUsers; Get-AppxProvisionedPackage -Online | Where-Object DisplayName -eq $env:XB_APPX | Remove-AppxProvisionedPackage -Online | Out-Null"
    } else {
        "Get-AppxPackage -Name $env:XB_APPX | Remove-AppxPackage"
    };
    let s = comando::powershell(script, &[("XB_APPX", nome)], Duration::from_secs(300))?;
    if s.ok() {
        Ok(())
    } else {
        Err(format!("Não foi possível remover {nome}: {}", s.erro()))
    }
}

fn reinstalar_appx(nome: &str) -> Result<(), String> {
    if !protecao::nome_appx_valido(nome) {
        return Err(format!("Nome de app inválido: {nome}"));
    }
    let script = "$p=Get-AppxPackage -AllUsers -Name $env:XB_APPX | Select-Object -First 1; if(-not $p){ exit 3 }; Add-AppxPackage -DisableDevelopmentMode -Register (Join-Path $p.InstallLocation 'AppXManifest.xml')";
    let s = comando::powershell(script, &[("XB_APPX", nome)], Duration::from_secs(300))?;
    match s.codigo {
        0 => Ok(()),
        3 => Err(format!("O app {nome} não está mais no PC. Reinstale pela Microsoft Store (procure pelo nome).")),
        _ => Err(format!("Não foi possível reinstalar {nome}: {}. Tente pela Microsoft Store.", s.erro())),
    }
}

/// Captura uma mudança de registro. `Ok(None)` = já está como desejado.
pub fn capturar_registro(chave: &str, nome: &str, depois: Option<ValorReg>) -> Result<Option<Mudanca>, String> {
    let antes = registro::ler(chave, nome)?;
    if antes == depois {
        return Ok(None);
    }
    let chave_criada = if depois.is_some() { registro::primeira_chave_ausente(chave)? } else { None };
    Ok(Some(Mudanca::Registro { chave: registro::normalizar(chave), nome: nome.to_string(), antes, depois, chave_criada }))
}

/// Captura uma mudança de serviço. `Ok(None)` = não existe neste PC ou já está como desejado.
pub fn capturar_servico(nome: &str, depois: InicioServico) -> Result<Option<Mudanca>, String> {
    if protecao::servico_protegido(nome) {
        return Err(format!("O serviço {nome} está na lista de proteção e não é alterado."));
    }
    let Some(antes) = servicos::ler_inicio(nome)? else { return Ok(None) };
    if antes == depois || matches!(antes, InicioServico::Boot | InicioServico::Sistema) {
        return Ok(None);
    }
    Ok(Some(Mudanca::Servico { nome: nome.to_string(), antes, depois }))
}

/// Captura uma mudança de configuração de energia no plano ativo.
pub fn capturar_energia(subgrupo: &str, configuracao: &str, ca: bool, depois: u32) -> Result<Option<Mudanca>, String> {
    let esquema = energia::plano_ativo()?;
    let antes = energia::ler_valor(&esquema, subgrupo, configuracao, ca)?;
    if antes == depois {
        return Ok(None);
    }
    Ok(Some(Mudanca::Energia { esquema, subgrupo: subgrupo.to_string(), configuracao: configuracao.to_string(), ca, antes, depois }))
}

/// Captura a troca de plano de energia.
pub fn capturar_plano(depois: &str, criado: Option<String>) -> Result<Option<Mudanca>, String> {
    let antes = energia::plano_ativo()?;
    if antes.eq_ignore_ascii_case(depois) {
        return Ok(None);
    }
    Ok(Some(Mudanca::PlanoEnergia { antes, depois: depois.to_string(), criado }))
}

/// Captura a mudança de uma tarefa agendada. `Ok(None)` = não existe ou já está como desejado.
pub fn capturar_tarefa(caminho: &str, depois: bool) -> Result<Option<Mudanca>, String> {
    let Some(antes) = tarefas::esta_ativa(caminho)? else { return Ok(None) };
    if antes == depois {
        return Ok(None);
    }
    Ok(Some(Mudanca::Tarefa { caminho: caminho.to_string(), antes, depois }))
}

/// Captura a mudança da hibernação.
pub fn capturar_hibernacao(depois: bool) -> Result<Option<Mudanca>, String> {
    let antes = hibernacao_ativa();
    if antes == depois {
        return Ok(None);
    }
    Ok(Some(Mudanca::Hibernacao { antes, depois }))
}

/// Captura a mudança de um recurso opcional.
pub fn capturar_recurso(nome: &str, depois: bool) -> Result<Option<Mudanca>, String> {
    let antes = recurso_ativo(nome)?;
    if antes == depois {
        return Ok(None);
    }
    Ok(Some(Mudanca::Recurso { nome: nome.to_string(), antes, depois }))
}

/// Captura a mudança de DNS de um adaptador (lê o estado anterior direto do registro).
pub fn capturar_dns(indice: u32, guid: &str, alias: &str, depois: Vec<String>) -> Result<Option<Mudanca>, String> {
    if depois.iter().any(|s| !ip_valido(s)) {
        return Err("Endereço de DNS inválido.".into());
    }
    let g = guid.trim().trim_start_matches('{').trim_end_matches('}');
    if energia::guid(g).is_none() {
        return Err("Identificador de adaptador inválido.".into());
    }
    let v4 = registro::ler_texto(&format!(r"HKLM\SYSTEM\CurrentControlSet\Services\Tcpip\Parameters\Interfaces\{{{g}}}"), "NameServer").unwrap_or_default();
    let v6 = registro::ler_texto(&format!(r"HKLM\SYSTEM\CurrentControlSet\Services\Tcpip6\Parameters\Interfaces\{{{g}}}"), "NameServer").unwrap_or_default();
    Ok(Some(Mudanca::Dns { indice, guid: g.to_string(), alias: alias.chars().take(80).collect(), antes_v4: v4, antes_v6: v6, depois }))
}

/// Captura a remoção de um app. `Ok(None)` = não está instalado.
pub fn capturar_appx(nome: &str, todos_usuarios: bool) -> Result<Option<Mudanca>, String> {
    if !protecao::nome_appx_valido(nome) {
        return Err(format!("Nome de app inválido: {nome}"));
    }
    if protecao::appx_protegido(nome) {
        return Err(format!("O app {nome} está na lista de proteção."));
    }
    let script = if todos_usuarios {
        "(Get-AppxPackage -AllUsers -Name $env:XB_APPX | Select-Object -First 1).PackageFamilyName"
    } else {
        "(Get-AppxPackage -Name $env:XB_APPX | Select-Object -First 1).PackageFamilyName"
    };
    let s = comando::powershell(script, &[("XB_APPX", nome)], Duration::from_secs(120))?;
    let familia = s.stdout.trim().to_string();
    if familia.is_empty() {
        return Ok(None);
    }
    Ok(Some(Mudanca::Appx { nome: nome.to_string(), familia, todos_usuarios }))
}

/// Prepara a troca de um arquivo de texto do sistema: guarda cópia do antes e do depois na
/// pasta de backup da operação.
pub fn capturar_arquivo(caminho: &Path, pasta_backup: &Path, conteudo_novo: &[u8]) -> Result<Option<Mudanca>, String> {
    let existia = caminho.exists();
    if existia && std::fs::read(caminho).map(|a| a == conteudo_novo).unwrap_or(false) {
        return Ok(None);
    }
    std::fs::create_dir_all(pasta_backup).map_err(|e| e.to_string())?;
    let base = crate::util::nome_seguro(&caminho.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| "arquivo".into()));
    let backup = if existia {
        let b = pasta_backup.join(format!("{base}.antes"));
        std::fs::copy(caminho, &b).map_err(|e| format!("Não foi possível copiar {}: {e}", caminho.display()))?;
        Some(b.to_string_lossy().into_owned())
    } else {
        None
    };
    let novo = pasta_backup.join(format!("{base}.depois"));
    crate::util::escrever_atomico(&novo, conteudo_novo).map_err(|e| e.to_string())?;
    let _ = sistema::modo_seguranca();
    Ok(Some(Mudanca::Arquivo { caminho: caminho.to_string_lossy().into_owned(), backup, novo: novo.to_string_lossy().into_owned(), existia }))
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn json_do_diario_e_estavel() {
        let m = Mudanca::Registro {
            chave: r"HKLM\SOFTWARE\Policies\Microsoft\Windows\DataCollection".into(),
            nome: "AllowTelemetry".into(),
            antes: None,
            depois: Some(ValorReg::Dword(0)),
            chave_criada: Some(r"HKLM\SOFTWARE\Policies\Microsoft\Windows\DataCollection".into()),
        };
        let j = serde_json::to_string(&m).expect("json");
        assert!(j.contains("\"tipo\":\"registro\""));
        let volta: Mudanca = serde_json::from_str(&j).expect("ler");
        assert_eq!(volta, m);
    }
}
