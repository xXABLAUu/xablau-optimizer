//! Diário de transações (journal): cada passo é gravado ANTES de ser executado.
//!
//! Formato: um arquivo `.jsonl` por operação em `%ProgramData%\XablauOptimizer\diario`, uma linha
//! JSON por evento, com `fsync` a cada linha. Se o PC desligar no meio, a operação fica sem a
//! linha de "fim" e, ao abrir o programa (ou o kit de emergência), é oferecido desfazer.
//! Uma linha cortada pela queda de energia é simplesmente ignorada na leitura.

use crate::mudancas::Mudanca;
use crate::{caminhos, log, registro, util};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader};
use std::os::windows::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "t", rename_all = "snake_case")]
enum Linha {
    Inicio { op: String, titulo: String, origem: String, quando: String, versao: String },
    PontoRestauracao { descricao: String, quando: String },
    Pendente {
        id: u32,
        ajuste: String,
        descricao: String,
        mudanca: Mudanca,
        #[serde(default)]
        backup_reg: Option<String>,
        quando: String,
    },
    Feita { id: u32, quando: String },
    Falhou { id: u32, erro: String, quando: String },
    Desfeita { id: u32, origem: String, quando: String },
    FalhaDesfazer { id: u32, erro: String, quando: String },
    Fim { status: String, quando: String },
}

/// Estado de cada mudança dentro de uma operação.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EstadoItem {
    /// Gravada no diário, mas a execução não foi confirmada (pode ter sido interrompida).
    Pendente,
    Feita,
    Falhou,
    Desfeita,
    FalhaAoDesfazer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemMudanca {
    pub id: u32,
    pub ajuste: String,
    pub descricao: String,
    pub mudanca: Mudanca,
    pub backup_reg: Option<String>,
    pub estado: EstadoItem,
    pub erro: Option<String>,
    pub quando: String,
    pub desfeito_em: Option<String>,
    pub reversivel: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operacao {
    pub id: String,
    pub titulo: String,
    pub origem: String,
    pub versao: String,
    pub inicio: String,
    pub fim: Option<String>,
    pub status: Option<String>,
    pub ponto_restauracao: Option<String>,
    pub itens: Vec<ItemMudanca>,
}

impl Operacao {
    /// Sem a linha de "fim": a operação foi interrompida (PC desligou, programa travou...).
    pub fn incompleta(&self) -> bool {
        self.fim.is_none()
    }

    /// Quantas mudanças ainda podem ser desfeitas.
    pub fn desfaziveis(&self) -> usize {
        self.itens.iter().filter(|i| precisa_desfazer(i)).count()
    }
}

fn precisa_desfazer(i: &ItemMudanca) -> bool {
    match i.estado {
        EstadoItem::Desfeita => false,
        EstadoItem::Falhou => i.mudanca.desfazer_e_idempotente(),
        EstadoItem::Pendente | EstadoItem::Feita | EstadoItem::FalhaAoDesfazer => i.reversivel,
    }
}

/// Trava exclusiva: só uma operação (ou desfazer) por vez, entre todos os programas do Xablau.
pub struct Trava {
    _arquivo: File,
}

/// Pega a trava. Erro `WouldBlock` = outra operação está em andamento.
pub fn travar() -> io::Result<Trava> {
    fs::create_dir_all(caminhos::diario())?;
    match OpenOptions::new().create(true).truncate(false).write(true).share_mode(0).open(caminhos::diario().join(".trava")) {
        Ok(f) => Ok(Trava { _arquivo: f }),
        Err(e) if e.raw_os_error() == Some(32) => {
            Err(io::Error::new(io::ErrorKind::WouldBlock, "Outra operação do Xablau Optimizer está em andamento."))
        }
        Err(e) => Err(e),
    }
}

/// Existe operação em andamento agora (em qualquer programa do Xablau)?
pub fn operacao_em_andamento() -> bool {
    matches!(travar(), Err(e) if e.kind() == io::ErrorKind::WouldBlock)
}

/// "Journal aberto": operação em andamento OU interrompida esperando decisão.
/// Enquanto isso, o programa não se atualiza.
pub fn diario_aberto() -> bool {
    operacao_em_andamento() || !operacoes_incompletas().is_empty()
}

fn arquivo_da_operacao(id: &str) -> io::Result<PathBuf> {
    if !util::id_valido(id) {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Identificador de operação inválido."));
    }
    Ok(caminhos::diario().join(format!("{id}.jsonl")))
}

fn gravar(caminho: &Path, linha: &Linha) -> io::Result<()> {
    let texto = serde_json::to_string(linha).map_err(io::Error::other)?;
    util::anexar_linha(caminho, &texto)
}

/// Operação em andamento: grava cada passo no diário antes de executar.
pub struct Diario {
    pub id: String,
    pub titulo: String,
    origem: String,
    caminho: PathBuf,
    proximo: u32,
    exportadas: HashSet<String>,
    _trava: Trava,
}

impl Diario {
    /// Abre uma operação nova (pega a trava exclusiva).
    pub fn iniciar(titulo: &str, origem: &str) -> io::Result<Diario> {
        caminhos::garantir()?;
        let trava = travar()?;
        let id = util::gerar_id("op");
        let caminho = arquivo_da_operacao(&id)?;
        OpenOptions::new().write(true).create_new(true).open(&caminho)?;
        let d = Diario {
            id: id.clone(),
            titulo: titulo.to_string(),
            origem: origem.to_string(),
            caminho,
            proximo: 1,
            exportadas: HashSet::new(),
            _trava: trava,
        };
        gravar(
            &d.caminho,
            &Linha::Inicio { op: id, titulo: titulo.to_string(), origem: origem.to_string(), quando: util::agora_iso(), versao: crate::VERSAO.to_string() },
        )?;
        log::info(origem, &format!("Operação iniciada: {titulo} ({})", d.id));
        Ok(d)
    }

    /// Pasta de backup desta operação.
    pub fn pasta_backup(&self) -> PathBuf {
        caminhos::backups().join(&self.id)
    }

    /// Registra o ponto de restauração confirmado.
    pub fn ponto_restauracao(&mut self, descricao: &str) -> io::Result<()> {
        gravar(&self.caminho, &Linha::PontoRestauracao { descricao: descricao.to_string(), quando: util::agora_iso() })
    }

    fn exportar_chave(&mut self, chave: &str) -> io::Result<Option<String>> {
        let normal = registro::normalizar(chave);
        if !registro::chave_existe(&normal) {
            return Ok(None);
        }
        let destino = self.pasta_backup().join(format!(
            "{:03}-{}.reg",
            self.exportadas.len() + 1,
            util::nome_seguro(normal.rsplit('\\').next().unwrap_or("chave"))
        ));
        if self.exportadas.contains(&normal.to_lowercase()) {
            return Ok(None);
        }
        registro::exportar(&normal, &destino).map_err(io::Error::other)?;
        self.exportadas.insert(normal.to_lowercase());
        Ok(Some(destino.to_string_lossy().into_owned()))
    }

    /// Grava a mudança como PENDENTE (com backup granular da chave do registro) e devolve o id.
    /// Só depois disso a mudança pode ser aplicada. Se o backup falhar, nada é gravado e a
    /// mudança não deve ser aplicada.
    pub fn pendente(&mut self, ajuste: &str, mudanca: &Mudanca) -> io::Result<u32> {
        let backup_reg = match mudanca {
            Mudanca::Registro { chave, .. } => self.exportar_chave(chave)?,
            Mudanca::Servico { nome, .. } => self.exportar_chave(&format!(r"HKLM\SYSTEM\CurrentControlSet\Services\{nome}"))?,
            _ => None,
        };
        let id = self.proximo;
        self.proximo += 1;
        gravar(
            &self.caminho,
            &Linha::Pendente {
                id,
                ajuste: ajuste.to_string(),
                descricao: mudanca.descrever(),
                mudanca: mudanca.clone(),
                backup_reg,
                quando: util::agora_iso(),
            },
        )?;
        Ok(id)
    }

    /// Confirma que a mudança foi aplicada.
    pub fn concluida(&mut self, id: u32) -> io::Result<()> {
        gravar(&self.caminho, &Linha::Feita { id, quando: util::agora_iso() })
    }

    /// Registra que a mudança falhou ao aplicar.
    pub fn falhou(&mut self, id: u32, erro: &str) -> io::Result<()> {
        log::erro(&self.origem, &format!("Falhou: {erro}"));
        gravar(&self.caminho, &Linha::Falhou { id, erro: erro.to_string(), quando: util::agora_iso() })
    }

    /// Fecha a operação ("ok", "parcial", "cancelada"...).
    pub fn finalizar(self, status: &str) -> io::Result<()> {
        gravar(&self.caminho, &Linha::Fim { status: status.to_string(), quando: util::agora_iso() })?;
        log::info(&self.origem, &format!("Operação finalizada ({status}): {} ({})", self.titulo, self.id));
        Ok(())
    }
}

fn ler_arquivo(caminho: &Path) -> Option<Operacao> {
    let f = File::open(caminho).ok()?;
    let mut op: Option<Operacao> = None;
    let mut indice: HashMap<u32, usize> = HashMap::new();
    for linha in BufReader::new(f).lines() {
        let Ok(linha) = linha else { break };
        let linha = linha.trim();
        if linha.is_empty() {
            continue;
        }
        let Ok(evento) = serde_json::from_str::<Linha>(linha) else { continue };
        if let Linha::Inicio { op: id, titulo, origem, quando, versao } = &evento {
            op = Some(Operacao {
                id: id.clone(),
                titulo: titulo.clone(),
                origem: origem.clone(),
                versao: versao.clone(),
                inicio: quando.clone(),
                fim: None,
                status: None,
                ponto_restauracao: None,
                itens: Vec::new(),
            });
            continue;
        }
        let Some(o) = op.as_mut() else { continue };
        match evento {
            Linha::Inicio { .. } => {}
            Linha::PontoRestauracao { descricao, .. } => o.ponto_restauracao = Some(descricao),
            Linha::Pendente { id, ajuste, descricao, mudanca, backup_reg, quando } => {
                indice.insert(id, o.itens.len());
                let reversivel = mudanca.reversivel();
                o.itens.push(ItemMudanca {
                    id,
                    ajuste,
                    descricao,
                    mudanca,
                    backup_reg,
                    estado: EstadoItem::Pendente,
                    erro: None,
                    quando,
                    desfeito_em: None,
                    reversivel,
                });
            }
            Linha::Feita { id, .. } => {
                if let Some(&i) = indice.get(&id) {
                    o.itens[i].estado = EstadoItem::Feita;
                }
            }
            Linha::Falhou { id, erro, .. } => {
                if let Some(&i) = indice.get(&id) {
                    o.itens[i].estado = EstadoItem::Falhou;
                    o.itens[i].erro = Some(erro);
                }
            }
            Linha::Desfeita { id, quando, .. } => {
                if let Some(&i) = indice.get(&id) {
                    o.itens[i].estado = EstadoItem::Desfeita;
                    o.itens[i].desfeito_em = Some(quando);
                    o.itens[i].erro = None;
                }
            }
            Linha::FalhaDesfazer { id, erro, .. } => {
                if let Some(&i) = indice.get(&id) {
                    o.itens[i].estado = EstadoItem::FalhaAoDesfazer;
                    o.itens[i].erro = Some(erro);
                }
            }
            Linha::Fim { status, quando } => {
                o.fim = Some(quando);
                o.status = Some(status);
            }
        }
    }
    op
}

/// Todas as operações, da mais nova para a mais antiga.
pub fn listar_operacoes() -> Vec<Operacao> {
    let mut v: Vec<Operacao> = fs::read_dir(caminhos::diario())
        .map(|it| {
            it.filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| p.extension().is_some_and(|x| x == "jsonl"))
                .filter_map(|p| ler_arquivo(&p))
                .collect()
        })
        .unwrap_or_default();
    v.sort_by(|a, b| b.id.cmp(&a.id));
    v
}

/// Uma operação pelo id.
pub fn ler_operacao(id: &str) -> Option<Operacao> {
    ler_arquivo(&arquivo_da_operacao(id).ok()?)
}

/// Operações interrompidas (sem "fim").
pub fn operacoes_incompletas() -> Vec<Operacao> {
    listar_operacoes().into_iter().filter(Operacao::incompleta).collect()
}

/// Resultado de um desfazer.
#[derive(Debug, Clone, Default, Serialize)]
pub struct ResultadoDesfazer {
    pub desfeitas: u32,
    pub ignoradas: u32,
    pub falhas: Vec<String>,
}

fn desfazer_item_sem_trava(op_id: &str, item: &ItemMudanca, origem: &str, r: &mut ResultadoDesfazer) -> io::Result<()> {
    let caminho = arquivo_da_operacao(op_id)?;
    match item.mudanca.desfazer() {
        Ok(()) => {
            gravar(&caminho, &Linha::Desfeita { id: item.id, origem: origem.to_string(), quando: util::agora_iso() })?;
            log::info(origem, &format!("Desfeito: {}", item.descricao));
            r.desfeitas += 1;
        }
        Err(e) => {
            gravar(&caminho, &Linha::FalhaDesfazer { id: item.id, erro: e.clone(), quando: util::agora_iso() })?;
            log::erro(origem, &format!("Não foi possível desfazer \"{}\": {e}", item.descricao));
            r.falhas.push(format!("{}: {e}", item.descricao));
        }
    }
    Ok(())
}

fn desfazer_operacao_sem_trava(op_id: &str, origem: &str) -> Result<ResultadoDesfazer, String> {
    let op = ler_operacao(op_id).ok_or("Operação não encontrada.")?;
    let mut r = ResultadoDesfazer::default();
    for item in op.itens.iter().rev() {
        if !precisa_desfazer(item) {
            r.ignoradas += 1;
            continue;
        }
        desfazer_item_sem_trava(op_id, item, origem, &mut r).map_err(|e| e.to_string())?;
    }
    if op.incompleta() {
        let caminho = arquivo_da_operacao(op_id).map_err(|e| e.to_string())?;
        gravar(&caminho, &Linha::Fim { status: "desfeita".into(), quando: util::agora_iso() }).map_err(|e| e.to_string())?;
    }
    Ok(r)
}

/// Desfaz uma operação inteira (do último passo para o primeiro).
pub fn desfazer_operacao(op_id: &str, origem: &str) -> Result<ResultadoDesfazer, String> {
    let _trava = travar().map_err(|e| e.to_string())?;
    log::info(origem, &format!("Desfazendo a operação {op_id}"));
    desfazer_operacao_sem_trava(op_id, origem)
}

/// Desfaz só um item de uma operação.
pub fn desfazer_item(op_id: &str, item_id: u32, origem: &str) -> Result<ResultadoDesfazer, String> {
    let _trava = travar().map_err(|e| e.to_string())?;
    let op = ler_operacao(op_id).ok_or("Operação não encontrada.")?;
    let item = op.itens.iter().find(|i| i.id == item_id).ok_or("Item não encontrado.")?;
    let mut r = ResultadoDesfazer::default();
    if !precisa_desfazer(item) {
        r.ignoradas = 1;
        return Ok(r);
    }
    desfazer_item_sem_trava(op_id, item, origem, &mut r).map_err(|e| e.to_string())?;
    Ok(r)
}

/// Desfaz TUDO o que o Xablau já fez, da operação mais nova para a mais antiga.
pub fn desfazer_tudo(origem: &str) -> Result<Vec<(String, ResultadoDesfazer)>, String> {
    let _trava = travar().map_err(|e| e.to_string())?;
    log::info(origem, "Desfazendo todas as alterações");
    let mut resultados = Vec::new();
    for op in listar_operacoes() {
        if op.desfaziveis() > 0 || op.incompleta() {
            resultados.push((op.id.clone(), desfazer_operacao_sem_trava(&op.id, origem)?));
        }
    }
    Ok(resultados)
}

/// Mantém as alterações de uma operação interrompida e a marca como encerrada.
pub fn encerrar_incompleta(op_id: &str, origem: &str) -> Result<(), String> {
    let _trava = travar().map_err(|e| e.to_string())?;
    let op = ler_operacao(op_id).ok_or("Operação não encontrada.")?;
    if !op.incompleta() {
        return Ok(());
    }
    let caminho = arquivo_da_operacao(op_id).map_err(|e| e.to_string())?;
    gravar(&caminho, &Linha::Fim { status: "mantida_pelo_usuario".into(), quando: util::agora_iso() }).map_err(|e| e.to_string())?;
    log::info(origem, &format!("Operação interrompida mantida pelo usuário: {op_id}"));
    Ok(())
}

/// Mudanças feitas DEPOIS, por outras operações, no mesmo alvo deste item (aviso antes de
/// desfazer item a item).
pub fn conflitos_posteriores(op_id: &str, item_id: u32) -> Vec<String> {
    let ops = listar_operacoes();
    let Some(op) = ops.iter().find(|o| o.id == op_id) else { return Vec::new() };
    let Some(item) = op.itens.iter().find(|i| i.id == item_id) else { return Vec::new() };
    let alvo = item.mudanca.alvo();
    ops.iter()
        .filter(|o| o.id > op.id)
        .flat_map(|o| o.itens.iter().map(move |i| (o, i)))
        .filter(|(_, i)| i.mudanca.alvo() == alvo && !matches!(i.estado, EstadoItem::Desfeita | EstadoItem::Falhou))
        .map(|(o, i)| format!("{} ({}): {}", o.titulo, o.inicio, i.descricao))
        .collect()
}

/// Ajuste que o Windows (provavelmente uma atualização) desfez.
#[derive(Debug, Clone, Serialize)]
pub struct Revertida {
    pub op_id: String,
    pub item_id: u32,
    pub ajuste: String,
    pub descricao: String,
}

/// Procura ajustes aplicados que não estão mais em vigor. Só AVISA: nunca reaplica sozinho.
pub fn alteracoes_revertidas() -> Vec<Revertida> {
    let mut vistos: HashSet<String> = HashSet::new();
    let mut resultado = Vec::new();
    for op in listar_operacoes() {
        for item in op.itens.iter().rev() {
            let alvo = item.mudanca.alvo();
            if !vistos.insert(alvo) {
                continue;
            }
            if item.estado == EstadoItem::Feita && item.mudanca.esta_aplicada() == Some(false) {
                resultado.push(Revertida { op_id: op.id.clone(), item_id: item.id, ajuste: item.ajuste.clone(), descricao: item.descricao.clone() });
            }
        }
    }
    resultado
}
