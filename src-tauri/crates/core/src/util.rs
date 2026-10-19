//! Utilidades: texto UTF-16, datas, identificadores e gravação segura de arquivos.

use serde::{de::DeserializeOwned, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::Path;
use std::sync::atomic::{AtomicU32, Ordering};

/// Marca de início dos arquivos UTF-8 gravados por alguns programas do Windows.
pub const BOM_UTF8: [u8; 3] = [0xEF, 0xBB, 0xBF];

/// Texto → UTF-16 terminado em zero (para as APIs "W" do Windows).
pub fn wide(texto: &str) -> Vec<u16> {
    texto.encode_utf16().chain(std::iter::once(0)).collect()
}

/// UTF-16 (com ou sem zero no fim) → texto.
pub fn de_wide(buf: &[u16]) -> String {
    let fim = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    String::from_utf16_lossy(&buf[..fim])
}

/// Data e hora local em ISO 8601 (ex.: 2026-10-18T21:30:05-03:00).
pub fn agora_iso() -> String {
    chrono::Local::now().format("%Y-%m-%dT%H:%M:%S%:z").to_string()
}

/// Data e hora local legível (ex.: 2026-10-18 21:30:05).
pub fn agora_legivel() -> String {
    chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

static CONTADOR: AtomicU32 = AtomicU32::new(0);

/// Identificador único e ordenável pelo tempo: `<prefixo>-AAAAMMDD-HHMMSS-xxxxxxxx`.
pub fn gerar_id(prefixo: &str) -> String {
    let agora = chrono::Local::now();
    let n = CONTADOR.fetch_add(1, Ordering::Relaxed);
    let mistura = agora.timestamp_subsec_nanos() ^ std::process::id().rotate_left(16) ^ n.wrapping_mul(0x9E37_79B9);
    format!("{prefixo}-{}-{mistura:08x}", agora.format("%Y%m%d-%H%M%S"))
}

/// Confere se um identificador é seguro para virar nome de arquivo (evita "..\" e afins).
pub fn id_valido(id: &str) -> bool {
    !id.is_empty() && id.len() <= 64 && id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
}

/// Nome seguro para arquivo (troca caracteres estranhos por "_").
pub fn nome_seguro(texto: &str) -> String {
    texto
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || "-_.".contains(c) { c } else { '_' })
        .take(80)
        .collect()
}

/// Grava um arquivo de forma atômica: escreve num temporário, força o disco e troca o nome.
/// Se faltar energia no meio, o arquivo antigo continua inteiro.
pub fn escrever_atomico(caminho: &Path, dados: &[u8]) -> io::Result<()> {
    if let Some(pai) = caminho.parent() {
        fs::create_dir_all(pai)?;
    }
    let tmp = caminho.with_extension(format!("tmp{}", std::process::id()));
    {
        let mut f = File::create(&tmp)?;
        f.write_all(dados)?;
        f.sync_all()?;
    }
    fs::rename(&tmp, caminho).inspect_err(|_| {
        let _ = fs::remove_file(&tmp);
    })
}

/// Grava JSON de forma atômica.
pub fn escrever_json<T: Serialize>(caminho: &Path, valor: &T) -> io::Result<()> {
    let dados = serde_json::to_vec_pretty(valor).map_err(io::Error::other)?;
    escrever_atomico(caminho, &dados)
}

/// Lê JSON (aceita arquivo com BOM). Devolve `None` se não existir ou estiver inválido.
pub fn ler_json<T: DeserializeOwned>(caminho: &Path) -> Option<T> {
    let dados = fs::read(caminho).ok()?;
    let sem_bom = dados.strip_prefix(&BOM_UTF8[..]).unwrap_or(&dados[..]);
    serde_json::from_slice(sem_bom).ok()
}

/// Acrescenta uma linha a um arquivo e força a gravação no disco (usado pelo diário).
pub fn anexar_linha(caminho: &Path, linha: &str) -> io::Result<()> {
    let mut f = OpenOptions::new().create(true).append(true).open(caminho)?;
    let mut texto = String::with_capacity(linha.len() + 1);
    texto.push_str(linha);
    texto.push('\n');
    f.write_all(texto.as_bytes())?;
    f.sync_data()
}

/// Bytes em hexadecimal, limitado a `maximo` bytes (para descrições legíveis).
pub fn hex(bytes: &[u8], maximo: usize) -> String {
    let mut s = bytes.iter().take(maximo).map(|b| format!("{b:02X}")).collect::<Vec<_>>().join(" ");
    if bytes.len() > maximo {
        s.push_str(" …");
    }
    s
}
