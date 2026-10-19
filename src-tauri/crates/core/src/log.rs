//! Log completo, com data e hora de cada alteração e de cada decisão automática
//! ("quem mudou o quê"). Um arquivo por mês em `%ProgramData%\XablauOptimizer\logs`.

use crate::{caminhos, util};
use serde::Serialize;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

static TRAVA: Mutex<()> = Mutex::new(());

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub enum Nivel {
    Info,
    Aviso,
    Erro,
}

impl Nivel {
    fn rotulo(self) -> &'static str {
        match self {
            Nivel::Info => "INFO",
            Nivel::Aviso => "AVISO",
            Nivel::Erro => "ERRO",
        }
    }
}

fn arquivo_do_mes() -> PathBuf {
    caminhos::logs().join(format!("xablau-{}.log", chrono::Local::now().format("%Y-%m")))
}

/// Registra uma linha no log. `origem`: "interface", "vigia", "kit", "manutencao"...
pub fn registrar(origem: &str, nivel: Nivel, mensagem: &str) {
    let _g = TRAVA.lock().unwrap_or_else(|e| e.into_inner());
    let _ = fs::create_dir_all(caminhos::logs());
    let limpa: String = mensagem.chars().map(|c| if c == '\r' || c == '\n' { ' ' } else { c }).collect();
    let linha = format!("{} | {:<5} | {:<10} | {}\r\n", util::agora_legivel(), nivel.rotulo(), origem, limpa);
    if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(arquivo_do_mes()) {
        let _ = f.write_all(linha.as_bytes());
    }
}

pub fn info(origem: &str, mensagem: &str) {
    registrar(origem, Nivel::Info, mensagem);
}
pub fn aviso(origem: &str, mensagem: &str) {
    registrar(origem, Nivel::Aviso, mensagem);
}
pub fn erro(origem: &str, mensagem: &str) {
    registrar(origem, Nivel::Erro, mensagem);
}

fn arquivos_de_log() -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = fs::read_dir(caminhos::logs())
        .map(|it| {
            it.filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| p.extension().is_some_and(|x| x == "log"))
                .collect()
        })
        .unwrap_or_default();
    v.sort();
    v
}

/// Últimas linhas do log (mais recentes por último).
pub fn ler_recentes(maximo: usize) -> Vec<String> {
    let mut linhas: Vec<String> = Vec::new();
    for arq in arquivos_de_log().iter().rev().take(2).rev() {
        if let Ok(texto) = fs::read_to_string(arq) {
            linhas.extend(texto.lines().map(str::to_string));
        }
    }
    let inicio = linhas.len().saturating_sub(maximo);
    linhas.split_off(inicio)
}

/// Junta todos os logs em um único arquivo `.txt` na pasta indicada. Devolve o caminho criado.
pub fn exportar(pasta_destino: &Path) -> io::Result<PathBuf> {
    let destino = pasta_destino.join(format!("Xablau-Optimizer-log-{}.txt", chrono::Local::now().format("%Y-%m-%d_%H%M%S")));
    let mut saida = fs::File::create(&destino)?;
    writeln!(saida, "Xablau Optimizer {} — log exportado em {}\r", crate::VERSAO, util::agora_legivel())?;
    for arq in arquivos_de_log() {
        writeln!(saida, "\r\n===== {} =====\r", arq.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default())?;
        saida.write_all(&fs::read(&arq)?)?;
    }
    saida.sync_all()?;
    Ok(destino)
}
