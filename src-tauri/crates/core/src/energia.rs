//! Planos de energia e configurações de energia, pela API oficial (powrprof.dll).

use crate::util;
use serde::Serialize;
use std::ffi::c_void;
use std::ptr::{null, null_mut};
use windows_sys::core::GUID;
use windows_sys::Win32::Foundation::LocalFree;
use windows_sys::Win32::System::Power::{
    PowerDeleteScheme, PowerDuplicateScheme, PowerEnumerate, PowerGetActiveScheme, PowerReadACValueIndex,
    PowerReadDCValueIndex, PowerReadFriendlyName, PowerSetActiveScheme, PowerWriteACValueIndex, PowerWriteDCValueIndex,
    PowerWriteFriendlyName, ACCESS_SCHEME,
};

/// Plano "Equilibrado".
pub const EQUILIBRADO: &str = "381b4222-f694-41f0-9685-ff5bb260df2e";
/// Plano "Alto desempenho".
pub const ALTO_DESEMPENHO: &str = "8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c";
/// Plano "Economia de energia".
pub const ECONOMIA: &str = "a1841308-3541-4fab-bc81-f71556f20b4a";
/// Modelo oculto "Desempenho Máximo" (Ultimate Performance), usado para criar o plano do Xablau.
pub const DESEMPENHO_MAXIMO_MODELO: &str = "e9a42b02-d5df-448d-aa00-03f14749eb61";
/// Nome do plano criado pelo Xablau.
pub const NOME_PLANO_XABLAU: &str = "Xablau Desempenho Máximo";

/// Converte texto em GUID ("8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c", com ou sem chaves).
pub fn guid(texto: &str) -> Option<GUID> {
    let t = texto.trim().trim_start_matches('{').trim_end_matches('}');
    let partes: Vec<&str> = t.split('-').collect();
    if partes.len() != 5 || partes.iter().zip([8, 4, 4, 4, 12]).any(|(p, n)| p.len() != n) {
        return None;
    }
    let d1 = u32::from_str_radix(partes[0], 16).ok()?;
    let d2 = u16::from_str_radix(partes[1], 16).ok()?;
    let d3 = u16::from_str_radix(partes[2], 16).ok()?;
    let d4a = u16::from_str_radix(partes[3], 16).ok()?;
    let d4b = u64::from_str_radix(partes[4], 16).ok()?;
    let mut d4 = [0u8; 8];
    d4[0] = (d4a >> 8) as u8;
    d4[1] = d4a as u8;
    for (i, b) in d4[2..].iter_mut().enumerate() {
        *b = (d4b >> (8 * (5 - i))) as u8;
    }
    Some(GUID { data1: d1, data2: d2, data3: d3, data4: d4 })
}

/// GUID → texto minúsculo.
pub fn texto(g: &GUID) -> String {
    format!(
        "{:08x}-{:04x}-{:04x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        g.data1, g.data2, g.data3, g.data4[0], g.data4[1], g.data4[2], g.data4[3], g.data4[4], g.data4[5], g.data4[6], g.data4[7]
    )
}

fn exigir(t: &str) -> Result<GUID, String> {
    guid(t).ok_or_else(|| format!("Identificador de energia inválido: {t}"))
}

/// Plano de energia.
#[derive(Debug, Clone, Serialize)]
pub struct Plano {
    pub guid: String,
    pub nome: String,
    pub ativo: bool,
}

/// GUID do plano ativo.
pub fn plano_ativo() -> Result<String, String> {
    unsafe {
        let mut p: *mut GUID = null_mut();
        let r = PowerGetActiveScheme(null_mut(), &mut p);
        if r != 0 || p.is_null() {
            return Err(format!("Não foi possível ler o plano de energia ativo (código {r})."));
        }
        let g = *p;
        LocalFree(p as *mut c_void);
        Ok(texto(&g))
    }
}

/// Ativa um plano de energia.
pub fn ativar(plano: &str) -> Result<(), String> {
    let g = exigir(plano)?;
    let r = unsafe { PowerSetActiveScheme(null_mut(), &g) };
    if r != 0 {
        Err(format!("Não foi possível ativar o plano de energia (código {r})."))
    } else {
        Ok(())
    }
}

fn nome_do_plano(g: &GUID) -> Option<String> {
    unsafe {
        let mut tam: u32 = 0;
        PowerReadFriendlyName(null_mut(), g, null(), null(), null_mut(), &mut tam);
        if tam == 0 || tam > 8192 {
            return None;
        }
        let mut buf = vec![0u8; tam as usize];
        if PowerReadFriendlyName(null_mut(), g, null(), null(), buf.as_mut_ptr(), &mut tam) != 0 {
            return None;
        }
        let u: Vec<u16> = buf.chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect();
        Some(util::de_wide(&u))
    }
}

/// Planos de energia instalados.
pub fn listar() -> Vec<Plano> {
    let ativo = plano_ativo().unwrap_or_default();
    let mut v = Vec::new();
    for i in 0..128u32 {
        let mut g: GUID = unsafe { std::mem::zeroed() };
        let mut tam = std::mem::size_of::<GUID>() as u32;
        let r = unsafe { PowerEnumerate(null_mut(), null(), null(), ACCESS_SCHEME, i, &mut g as *mut GUID as *mut u8, &mut tam) };
        if r != 0 {
            break;
        }
        let t = texto(&g);
        v.push(Plano { nome: nome_do_plano(&g).unwrap_or_else(|| t.clone()), ativo: t.eq_ignore_ascii_case(&ativo), guid: t });
    }
    v
}

/// O plano existe?
pub fn existe(plano: &str) -> bool {
    listar().iter().any(|p| p.guid.eq_ignore_ascii_case(plano))
}

/// Procura o plano criado pelo Xablau (pelo nome).
pub fn plano_xablau() -> Option<String> {
    listar().into_iter().find(|p| p.nome == NOME_PLANO_XABLAU).map(|p| p.guid)
}

/// Melhor plano de desempenho disponível SEM criar nada: o do Xablau ou o "Alto desempenho".
pub fn plano_desempenho_disponivel() -> Option<String> {
    plano_xablau().or_else(|| existe(ALTO_DESEMPENHO).then(|| ALTO_DESEMPENHO.to_string()))
}

/// Lê o valor de uma configuração (CA = na tomada, CC = na bateria).
pub fn ler_valor(esquema: &str, subgrupo: &str, configuracao: &str, ca: bool) -> Result<u32, String> {
    let (e, s, c) = (exigir(esquema)?, exigir(subgrupo)?, exigir(configuracao)?);
    let mut v: u32 = 0;
    let r = unsafe {
        if ca {
            PowerReadACValueIndex(null_mut(), &e, &s, &c, &mut v)
        } else {
            PowerReadDCValueIndex(null_mut(), &e, &s, &c, &mut v)
        }
    };
    if r != 0 {
        Err(format!("Não foi possível ler a configuração de energia (código {r})."))
    } else {
        Ok(v)
    }
}

/// Grava o valor de uma configuração e reaplica o plano se ele estiver ativo.
pub fn escrever_valor(esquema: &str, subgrupo: &str, configuracao: &str, ca: bool, valor: u32) -> Result<(), String> {
    let (e, s, c) = (exigir(esquema)?, exigir(subgrupo)?, exigir(configuracao)?);
    let r = unsafe {
        if ca {
            PowerWriteACValueIndex(null_mut(), &e, &s, &c, valor)
        } else {
            PowerWriteDCValueIndex(null_mut(), &e, &s, &c, valor)
        }
    };
    if r != 0 {
        return Err(format!("Não foi possível gravar a configuração de energia (código {r})."));
    }
    if plano_ativo().map(|a| a.eq_ignore_ascii_case(esquema)).unwrap_or(false) {
        ativar(esquema)?;
    }
    Ok(())
}

/// Duplica um plano (ex.: o modelo oculto "Desempenho Máximo"). Devolve o GUID novo.
pub fn duplicar(modelo: &str) -> Result<String, String> {
    let g = exigir(modelo)?;
    unsafe {
        let mut novo: *mut GUID = null_mut();
        let r = PowerDuplicateScheme(null_mut(), &g, &mut novo);
        if r != 0 || novo.is_null() {
            return Err(format!("Não foi possível criar o plano de energia (código {r})."));
        }
        let n = *novo;
        LocalFree(novo as *mut c_void);
        Ok(texto(&n))
    }
}

/// Renomeia um plano.
pub fn renomear(plano: &str, nome: &str) -> Result<(), String> {
    let g = exigir(plano)?;
    let bytes: Vec<u8> = util::wide(nome).iter().flat_map(|c| c.to_le_bytes()).collect();
    let r = unsafe { PowerWriteFriendlyName(null_mut(), &g, null(), null(), bytes.as_ptr(), bytes.len() as u32) };
    if r != 0 {
        Err(format!("Não foi possível renomear o plano (código {r})."))
    } else {
        Ok(())
    }
}

/// Apaga um plano (nunca o ativo nem os padrões do Windows).
pub fn apagar(plano: &str) -> Result<(), String> {
    if [EQUILIBRADO, ALTO_DESEMPENHO, ECONOMIA].iter().any(|p| p.eq_ignore_ascii_case(plano)) {
        return Err("Planos padrão do Windows não são apagados.".into());
    }
    if plano_ativo().map(|a| a.eq_ignore_ascii_case(plano)).unwrap_or(true) {
        return Err("O plano ativo não pode ser apagado.".into());
    }
    let g = exigir(plano)?;
    let r = unsafe { PowerDeleteScheme(null_mut(), &g) };
    if r != 0 {
        Err(format!("Não foi possível apagar o plano (código {r})."))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn guid_ida_e_volta() {
        let g = guid(ALTO_DESEMPENHO).expect("guid");
        assert_eq!(texto(&g), ALTO_DESEMPENHO);
        assert!(guid("nao-e-guid").is_none());
    }
}
