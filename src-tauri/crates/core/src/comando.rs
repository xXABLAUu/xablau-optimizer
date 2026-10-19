//! Execução de programas do sistema sem janela, com tempo limite e saída decodificada.
//!
//! Segurança: os programas são chamados pelo caminho completo (System32), nunca pelo nome solto,
//! para evitar que um executável falso na pasta atual ou no pendrive seja executado como administrador.
//! Parâmetros variáveis vão por variáveis de ambiente, nunca colados no texto do comando.

use std::io::Read;
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// Não abre janela de console.
pub const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// Resultado de um programa executado.
#[derive(Debug, Clone)]
pub struct Saida {
    pub codigo: i32,
    pub stdout: String,
    pub stderr: String,
}

impl Saida {
    pub fn ok(&self) -> bool {
        self.codigo == 0
    }
    /// Mensagem de erro amigável (stderr, ou stdout se o stderr vier vazio).
    pub fn erro(&self) -> String {
        let e = self.stderr.trim();
        if e.is_empty() { self.stdout.trim().to_string() } else { e.to_string() }
    }
}

/// Executa um programa com argumentos, sem janela, com tempo limite.
pub fn executar(programa: &Path, args: &[&str], limite: Duration) -> Result<Saida, String> {
    executar_com_env(programa, args, &[], limite)
}

/// Executa um programa de `System32` pelo nome (ex.: "reg.exe").
pub fn executar_sistema(exe: &str, args: &[&str], limite: Duration) -> Result<Saida, String> {
    executar(&crate::sistema::dir_sistema32().join(exe), args, limite)
}

/// Executa passando variáveis de ambiente extras (o jeito seguro de passar parâmetros).
pub fn executar_com_env(programa: &Path, args: &[&str], env: &[(&str, &str)], limite: Duration) -> Result<Saida, String> {
    let nome = programa.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    let mut cmd = Command::new(programa);
    cmd.args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .creation_flags(CREATE_NO_WINDOW);
    for (k, v) in env {
        cmd.env(k, v);
    }
    let mut filho = cmd.spawn().map_err(|e| format!("Não foi possível executar {nome}: {e}"))?;
    let mut saida = filho.stdout.take().ok_or("saída padrão indisponível")?;
    let mut erros = filho.stderr.take().ok_or("saída de erro indisponível")?;
    let leitor_saida = std::thread::spawn(move || {
        let mut v = Vec::new();
        let _ = saida.read_to_end(&mut v);
        v
    });
    let leitor_erros = std::thread::spawn(move || {
        let mut v = Vec::new();
        let _ = erros.read_to_end(&mut v);
        v
    });
    let inicio = Instant::now();
    let status = loop {
        match filho.try_wait() {
            Ok(Some(s)) => break s,
            Ok(None) if inicio.elapsed() > limite => {
                let _ = filho.kill();
                let _ = filho.wait();
                return Err(format!("{nome} demorou mais que {} s e foi interrompido.", limite.as_secs()));
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(40)),
            Err(e) => return Err(format!("Erro ao aguardar {nome}: {e}")),
        }
    };
    let so = leitor_saida.join().unwrap_or_default();
    let se = leitor_erros.join().unwrap_or_default();
    Ok(Saida { codigo: status.code().unwrap_or(-1), stdout: decodificar(&so), stderr: decodificar(&se) })
}

/// Caminho completo do Windows PowerShell 5.1.
pub fn caminho_powershell() -> PathBuf {
    crate::sistema::dir_sistema32().join(r"WindowsPowerShell\v1.0\powershell.exe")
}

/// Executa um comando PowerShell curto e fixo (escrito no código, nunca vindo do usuário).
/// Dados variáveis entram por `env` e são lidos no script como `$env:NOME`.
/// O script não pode conter aspas duplas (mantém a linha de comando simples e segura).
pub fn powershell(script: &str, env: &[(&str, &str)], limite: Duration) -> Result<Saida, String> {
    if script.contains('"') {
        return Err("Script interno inválido (aspas duplas não são permitidas).".into());
    }
    let completo = format!(
        "$ErrorActionPreference='Stop';$ProgressPreference='SilentlyContinue';[Console]::OutputEncoding=New-Object System.Text.UTF8Encoding $false;{script}"
    );
    executar_com_env(
        &caminho_powershell(),
        &["-NoLogo", "-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-Command", &completo],
        env,
        limite,
    )
}

/// Decodifica a saída de programas do Windows: UTF-16, UTF-8 ou página de código OEM do console.
pub fn decodificar(bytes: &[u8]) -> String {
    if bytes.is_empty() {
        return String::new();
    }
    let com_bom16 = bytes.starts_with(&[0xFF, 0xFE]);
    if com_bom16 || (bytes.len() >= 4 && bytes[1] == 0 && bytes[3] == 0) {
        let inicio = if com_bom16 { 2 } else { 0 };
        let u: Vec<u16> = bytes[inicio..].chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect();
        return String::from_utf16_lossy(&u);
    }
    let sem_bom = bytes.strip_prefix(&crate::util::BOM_UTF8[..]).unwrap_or(bytes);
    match std::str::from_utf8(sem_bom) {
        Ok(s) => s.to_string(),
        Err(_) => oem_para_texto(sem_bom),
    }
}

fn oem_para_texto(bytes: &[u8]) -> String {
    use windows_sys::Win32::Globalization::{MultiByteToWideChar, CP_OEMCP};
    let tamanho = i32::try_from(bytes.len()).unwrap_or(i32::MAX);
    unsafe {
        let n = MultiByteToWideChar(CP_OEMCP, 0, bytes.as_ptr(), tamanho, std::ptr::null_mut(), 0);
        if n <= 0 {
            return String::from_utf8_lossy(bytes).into_owned();
        }
        let mut buf = vec![0u16; n as usize];
        let escritos = MultiByteToWideChar(CP_OEMCP, 0, bytes.as_ptr(), tamanho, buf.as_mut_ptr(), n);
        buf.truncate(escritos.max(0) as usize);
        String::from_utf16_lossy(&buf)
    }
}
