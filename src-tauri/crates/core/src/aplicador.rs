//! Motor de aplicação: transforma uma lista de ajustes em mudanças concretas, simula (antes → depois),
//! cria o ponto de restauração e aplica tudo dentro de uma operação do diário (reversível).
//!
//! Fluxo seguro (nunca pule etapas):
//!   verificações → capturar mudanças → SIMULAÇÃO aprovada → ponto de restauração confirmado
//!   → para cada mudança: grava no diário (com backup) ANTES de aplicar → aplica → registra.
//! Se o ponto de restauração falhar, nada é aplicado.

use crate::mudancas::Mudanca;
use crate::{diario, log, restauracao};
use serde::{Deserialize, Serialize};

const ORIGEM: &str = "interface";

/// Uma mudança já capturada (com antes/depois reais), pronta para simular ou aplicar.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MudancaPlanejada {
    /// Identificador do ajuste do catálogo (ex.: "privacidade.telemetria").
    pub ajuste: String,
    /// Título amigável do ajuste.
    pub titulo: String,
    pub descricao: String,
    pub mudanca: Mudanca,
}

/// O que a simulação mostra ao usuário.
#[derive(Debug, Clone, Serialize)]
pub struct Simulacao {
    pub mudancas: Vec<MudancaPlanejada>,
    /// Ajustes pedidos que já estavam como desejado (nada a fazer).
    pub sem_efeito: Vec<String>,
    /// Ajustes que não puderam ser preparados (com o motivo).
    pub erros: Vec<(String, String)>,
    pub total: usize,
}

/// Resultado de aplicar uma operação.
#[derive(Debug, Clone, Serialize)]
pub struct ResultadoAplicacao {
    pub op_id: String,
    pub aplicadas: u32,
    pub falhas: Vec<String>,
    pub ponto_restauracao: restauracao::ResultadoPonto,
    pub status: String,
}

/// Monta a simulação a partir de mudanças já capturadas (quem captura é o catálogo).
pub fn simular(planejadas: Vec<MudancaPlanejada>, sem_efeito: Vec<String>, erros: Vec<(String, String)>) -> Simulacao {
    let total = planejadas.len();
    Simulacao { mudancas: planejadas, sem_efeito, erros, total }
}

/// Aplica as mudanças dentro de uma operação do diário.
///
/// `criar_ponto`: normalmente `true`. O ponto é criado uma vez, antes de aplicar. Se falhar,
/// a função retorna erro e NADA é aplicado. (Ações leves e reversíveis do modo automático podem
/// passar `false`, conforme a seção 5.12.)
pub fn aplicar(titulo: &str, planejadas: &[MudancaPlanejada], criar_ponto: bool) -> Result<ResultadoAplicacao, String> {
    if planejadas.is_empty() {
        return Err("Nenhuma mudança para aplicar.".into());
    }

    // Ponto de restauração ANTES de abrir a operação. Se falhar, nada é feito.
    let ponto = if criar_ponto {
        let p = restauracao::criar(&format!("Xablau Optimizer — {titulo}"));
        if !p.criado {
            return Err(format!("Ponto de restauração não confirmado: {} Nenhuma alteração foi aplicada.", p.mensagem));
        }
        p
    } else {
        restauracao::ResultadoPonto {
            criado: false,
            numero_sequencia: None,
            descricao: titulo.to_string(),
            mensagem: "Ação leve e reversível: aplicada sem ponto de restauração.".into(),
            protecao_ligada_agora: false,
        }
    };

    let mut diario = diario::Diario::iniciar(titulo, ORIGEM).map_err(|e| format!("Não foi possível abrir o diário: {e}"))?;
    if ponto.criado {
        let _ = diario.ponto_restauracao(&ponto.descricao);
    }
    let op_id = diario.id.clone();
    let _ = std::fs::create_dir_all(diario.pasta_backup());

    let mut aplicadas = 0u32;
    let mut falhas = Vec::new();

    for p in planejadas {
        // Grava como PENDENTE (com backup) ANTES de aplicar. Se o backup falhar, pula a mudança.
        let id = match diario.pendente(&p.ajuste, &p.mudanca) {
            Ok(id) => id,
            Err(e) => {
                let msg = format!("{}: não foi possível preparar o backup ({e}). Esta mudança foi ignorada.", p.titulo);
                log::erro(ORIGEM, &msg);
                falhas.push(msg);
                continue;
            }
        };
        match p.mudanca.aplicar() {
            Ok(()) => {
                let _ = diario.concluida(id);
                aplicadas += 1;
            }
            Err(e) => {
                let _ = diario.falhou(id, &e);
                falhas.push(format!("{}: {e}", p.titulo));
            }
        }
    }

    let status = if falhas.is_empty() {
        "ok"
    } else if aplicadas > 0 {
        "parcial"
    } else {
        "falhou"
    };
    let _ = diario.finalizar(status);

    Ok(ResultadoAplicacao { op_id, aplicadas, falhas, ponto_restauracao: ponto, status: status.to_string() })
}
