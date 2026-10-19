//! Registro do Windows: leitura, gravação, exclusão e exportação granular (.reg) de chaves.
//!
//! Caminhos no formato `HKLM\SOFTWARE\...` (também aceita `HKEY_LOCAL_MACHINE\...`).

use crate::{comando, sistema, util};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::io::ErrorKind;
use std::path::Path;
use std::time::Duration;
use winreg::enums::{RegType, HKEY_CLASSES_ROOT, HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, HKEY_USERS, KEY_READ, KEY_SET_VALUE};
use winreg::{RegKey, RegValue};

/// Valor do registro, guardado no diário de forma legível e exata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValorReg {
    Dword(u32),
    Qword(u64),
    Sz(String),
    ExpandSz(String),
    MultiSz(Vec<String>),
    Binario(Vec<u8>),
    /// Tipos raros (REG_NONE, REG_LINK...), guardados byte a byte.
    Bruto { tipo: u32, dados: Vec<u8> },
}

impl ValorReg {
    /// Texto curto para mostrar na simulação e no histórico.
    pub fn para_texto(&self) -> String {
        match self {
            ValorReg::Dword(v) => format!("{v}"),
            ValorReg::Qword(v) => format!("{v}"),
            ValorReg::Sz(s) | ValorReg::ExpandSz(s) => format!("\"{s}\""),
            ValorReg::MultiSz(v) => v.join(" | "),
            ValorReg::Binario(b) => util::hex(b, 12),
            ValorReg::Bruto { tipo, dados } => format!("tipo {tipo}: {}", util::hex(dados, 12)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Raiz {
    Hklm,
    Hkcu,
    Hkcr,
    Hku,
}

fn separar(caminho: &str) -> Result<(Raiz, String), String> {
    let (raiz, resto) = caminho.split_once('\\').unwrap_or((caminho, ""));
    let r = match raiz.to_ascii_uppercase().as_str() {
        "HKLM" | "HKEY_LOCAL_MACHINE" => Raiz::Hklm,
        "HKCU" | "HKEY_CURRENT_USER" => Raiz::Hkcu,
        "HKCR" | "HKEY_CLASSES_ROOT" => Raiz::Hkcr,
        "HKU" | "HKEY_USERS" => Raiz::Hku,
        _ => return Err(format!("Raiz de registro inválida em \"{caminho}\".")),
    };
    let resto = resto.trim_end_matches('\\');
    if resto.is_empty() || resto.starts_with('\\') || resto.contains("\\\\") {
        return Err(format!("Caminho de registro inválido: \"{caminho}\"."));
    }
    Ok((r, resto.to_string()))
}

fn prefixo(r: Raiz) -> &'static str {
    match r {
        Raiz::Hklm => "HKLM",
        Raiz::Hkcu => "HKCU",
        Raiz::Hkcr => "HKCR",
        Raiz::Hku => "HKU",
    }
}

fn raiz(r: Raiz) -> RegKey {
    RegKey::predef(match r {
        Raiz::Hklm => HKEY_LOCAL_MACHINE,
        Raiz::Hkcu => HKEY_CURRENT_USER,
        Raiz::Hkcr => HKEY_CLASSES_ROOT,
        Raiz::Hku => HKEY_USERS,
    })
}

/// Normaliza o caminho para comparação (prefixo curto, sem barra no fim).
pub fn normalizar(caminho: &str) -> String {
    match separar(caminho) {
        Ok((r, sub)) => format!("{}\\{}", prefixo(r), sub),
        Err(_) => caminho.to_string(),
    }
}

fn abrir_leitura(chave: &str) -> Result<Option<RegKey>, String> {
    let (r, sub) = separar(chave)?;
    match raiz(r).open_subkey_with_flags(&sub, KEY_READ) {
        Ok(k) => Ok(Some(k)),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(None),
        Err(e) => Err(format!("Não foi possível abrir {chave}: {e}")),
    }
}

/// A chave existe?
pub fn chave_existe(chave: &str) -> bool {
    matches!(abrir_leitura(chave), Ok(Some(_)))
}

/// Lê um valor. `Ok(None)` = valor (ou chave) não existe.
pub fn ler(chave: &str, nome: &str) -> Result<Option<ValorReg>, String> {
    let Some(k) = abrir_leitura(chave)? else { return Ok(None) };
    match k.get_raw_value(nome) {
        Ok(v) => Ok(Some(de_regvalue(&v))),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(None),
        Err(e) => Err(format!("Não foi possível ler {chave}\\{nome}: {e}")),
    }
}

/// Lê um DWORD (atalho).
pub fn ler_dword(chave: &str, nome: &str) -> Option<u32> {
    match ler(chave, nome) {
        Ok(Some(ValorReg::Dword(v))) => Some(v),
        _ => None,
    }
}

/// Lê um texto REG_SZ/REG_EXPAND_SZ (atalho).
pub fn ler_texto(chave: &str, nome: &str) -> Option<String> {
    match ler(chave, nome) {
        Ok(Some(ValorReg::Sz(s) | ValorReg::ExpandSz(s))) => Some(s),
        _ => None,
    }
}

/// Grava um valor (cria a chave se faltar).
pub fn escrever(chave: &str, nome: &str, valor: &ValorReg) -> Result<(), String> {
    let (r, sub) = separar(chave)?;
    let (k, _) = raiz(r).create_subkey(&sub).map_err(|e| format!("Não foi possível criar/abrir {chave}: {e}"))?;
    let v = para_regvalue(valor)?;
    k.set_raw_value(nome, &v).map_err(|e| format!("Não foi possível gravar {chave}\\{nome}: {e}"))
}

/// Apaga um valor (se não existir, não é erro).
pub fn apagar_valor(chave: &str, nome: &str) -> Result<(), String> {
    let (r, sub) = separar(chave)?;
    match raiz(r).open_subkey_with_flags(&sub, KEY_SET_VALUE) {
        Ok(k) => match k.delete_value(nome) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == ErrorKind::NotFound => Ok(()),
            Err(e) => Err(format!("Não foi possível apagar {chave}\\{nome}: {e}")),
        },
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(()),
        Err(e) => Err(format!("Não foi possível abrir {chave}: {e}")),
    }
}

/// Primeira chave do caminho que ainda NÃO existe (será criada ao gravar). `None` = já existe.
pub fn primeira_chave_ausente(chave: &str) -> Result<Option<String>, String> {
    let (r, sub) = separar(chave)?;
    let rk = raiz(r);
    let mut atual = String::new();
    for parte in sub.split('\\') {
        if !atual.is_empty() {
            atual.push('\\');
        }
        atual.push_str(parte);
        match rk.open_subkey_with_flags(&atual, KEY_READ) {
            Ok(_) => {}
            Err(e) if e.kind() == ErrorKind::NotFound => return Ok(Some(format!("{}\\{}", prefixo(r), atual))),
            Err(_) => {} // sem permissão de leitura: considera existente
        }
    }
    Ok(None)
}

fn esta_vazia(chave: &str) -> Result<Option<bool>, String> {
    let Some(k) = abrir_leitura(chave)? else { return Ok(None) };
    let info = k.query_info().map_err(|e| format!("Não foi possível consultar {chave}: {e}"))?;
    Ok(Some(info.sub_keys == 0 && info.values == 0))
}

/// Apaga `chave` e as chaves-mãe até `topo` (inclusive), mas só enquanto estiverem vazias.
/// Usado para desfazer chaves que o Xablau criou.
pub fn apagar_chaves_vazias(chave: &str, topo: &str) -> Result<(), String> {
    let topo = normalizar(topo).to_ascii_lowercase();
    let mut atual = normalizar(chave);
    loop {
        let baixa = atual.to_ascii_lowercase();
        if baixa != topo && !baixa.starts_with(&format!("{topo}\\")) {
            break;
        }
        match esta_vazia(&atual)? {
            Some(true) => {
                let (r, sub) = separar(&atual)?;
                raiz(r).delete_subkey(&sub).map_err(|e| format!("Não foi possível apagar a chave {atual}: {e}"))?;
            }
            Some(false) => break,
            None => {}
        }
        if baixa == topo {
            break;
        }
        match atual.rfind('\\') {
            Some(p) if p > 0 => atual.truncate(p),
            _ => break,
        }
    }
    Ok(())
}

/// Nomes das subchaves.
pub fn subchaves(chave: &str) -> Vec<String> {
    match abrir_leitura(chave) {
        Ok(Some(k)) => k.enum_keys().filter_map(Result::ok).collect(),
        _ => Vec::new(),
    }
}

/// Valores de uma chave.
pub fn valores(chave: &str) -> Vec<(String, ValorReg)> {
    match abrir_leitura(chave) {
        Ok(Some(k)) => k.enum_values().filter_map(Result::ok).map(|(n, v)| (n, de_regvalue(&v))).collect(),
        _ => Vec::new(),
    }
}

/// Exporta SÓ esta chave (com as subchaves dela) para um arquivo .reg — backup granular.
/// `Ok(false)` = a chave não existe (nada a exportar).
pub fn exportar(chave: &str, destino: &Path) -> Result<bool, String> {
    if !chave_existe(chave) {
        return Ok(false);
    }
    if let Some(pai) = destino.parent() {
        std::fs::create_dir_all(pai).map_err(|e| e.to_string())?;
    }
    let alvo = destino.to_string_lossy().into_owned();
    let normal = normalizar(chave);
    let s = comando::executar(&sistema::dir_sistema32().join("reg.exe"), &["export", &normal, &alvo, "/y"], Duration::from_secs(120))?;
    if s.ok() {
        Ok(true)
    } else {
        Err(format!("Falha ao exportar {normal}: {}", s.erro()))
    }
}

/// Importa um .reg exportado pelo Xablau (último recurso do kit de emergência).
pub fn importar(arquivo: &Path) -> Result<(), String> {
    let alvo = arquivo.to_string_lossy().into_owned();
    let s = comando::executar(&sistema::dir_sistema32().join("reg.exe"), &["import", &alvo], Duration::from_secs(120))?;
    if s.ok() {
        Ok(())
    } else {
        Err(format!("Falha ao importar {alvo}: {}", s.erro()))
    }
}

fn u16_de_bytes(b: &[u8]) -> Vec<u16> {
    b.chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect()
}

fn texto_de_bytes(b: &[u8]) -> String {
    util::de_wide(&u16_de_bytes(b))
}

fn multi_de_bytes(b: &[u8]) -> Vec<String> {
    u16_de_bytes(b).split(|&c| c == 0).filter(|s| !s.is_empty()).map(String::from_utf16_lossy).collect()
}

fn bytes_de_texto(s: &str) -> Vec<u8> {
    util::wide(s).iter().flat_map(|c| c.to_le_bytes()).collect()
}

fn bytes_de_multi(lista: &[String]) -> Vec<u8> {
    let mut u: Vec<u16> = Vec::new();
    for s in lista {
        u.extend(s.encode_utf16());
        u.push(0);
    }
    u.push(0);
    u.iter().flat_map(|c| c.to_le_bytes()).collect()
}

fn tipo_de_numero(n: u32) -> Result<RegType, String> {
    Ok(match n {
        0 => RegType::REG_NONE,
        1 => RegType::REG_SZ,
        2 => RegType::REG_EXPAND_SZ,
        3 => RegType::REG_BINARY,
        4 => RegType::REG_DWORD,
        5 => RegType::REG_DWORD_BIG_ENDIAN,
        6 => RegType::REG_LINK,
        7 => RegType::REG_MULTI_SZ,
        8 => RegType::REG_RESOURCE_LIST,
        9 => RegType::REG_FULL_RESOURCE_DESCRIPTOR,
        10 => RegType::REG_RESOURCE_REQUIREMENTS_LIST,
        11 => RegType::REG_QWORD,
        _ => return Err(format!("Tipo de registro desconhecido: {n}")),
    })
}

fn de_regvalue(v: &RegValue) -> ValorReg {
    let b: &[u8] = &v.bytes;
    match v.vtype {
        RegType::REG_DWORD if b.len() >= 4 => ValorReg::Dword(u32::from_le_bytes([b[0], b[1], b[2], b[3]])),
        RegType::REG_QWORD if b.len() >= 8 => ValorReg::Qword(u64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]])),
        RegType::REG_SZ => ValorReg::Sz(texto_de_bytes(b)),
        RegType::REG_EXPAND_SZ => ValorReg::ExpandSz(texto_de_bytes(b)),
        RegType::REG_MULTI_SZ => ValorReg::MultiSz(multi_de_bytes(b)),
        RegType::REG_BINARY => ValorReg::Binario(b.to_vec()),
        ref outro => ValorReg::Bruto { tipo: outro.clone() as u32, dados: b.to_vec() },
    }
}

fn para_regvalue(valor: &ValorReg) -> Result<RegValue<'static>, String> {
    let (bytes, vtype) = match valor {
        ValorReg::Dword(n) => (n.to_le_bytes().to_vec(), RegType::REG_DWORD),
        ValorReg::Qword(n) => (n.to_le_bytes().to_vec(), RegType::REG_QWORD),
        ValorReg::Sz(s) => (bytes_de_texto(s), RegType::REG_SZ),
        ValorReg::ExpandSz(s) => (bytes_de_texto(s), RegType::REG_EXPAND_SZ),
        ValorReg::MultiSz(l) => (bytes_de_multi(l), RegType::REG_MULTI_SZ),
        ValorReg::Binario(b) => (b.clone(), RegType::REG_BINARY),
        ValorReg::Bruto { tipo, dados } => (dados.clone(), tipo_de_numero(*tipo)?),
    };
    Ok(RegValue { bytes: Cow::Owned(bytes), vtype })
}
