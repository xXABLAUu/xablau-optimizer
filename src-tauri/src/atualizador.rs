//! Atualização automática do próprio programa, pelo plugin updater do Tauri.
//!
//! A verificação lê o latest.json publicado no GitHub Releases (via HTTPS) e confere a assinatura
//! com a chave pública embutida. O plugin recusa qualquer atualização sem assinatura válida.
//! Regras de segurança (seção 11.2):
//!   - nunca atualizar durante uma operação em andamento (diário aberto) nem com jogo aberto;
//!   - no modo portátil, não se autoatualiza: só avisa e abre a página da Release.

use serde::Serialize;
use tauri::AppHandle;
use tauri_plugin_updater::UpdaterExt;

/// Informação de atualização para a interface.
#[derive(Debug, Clone, Serialize)]
pub struct InfoAtualizacao {
    pub disponivel: bool,
    pub versao_atual: String,
    pub versao_nova: Option<String>,
    pub notas: Option<String>,
    pub data: Option<String>,
    /// Pode instalar agora? (false se houver operação/jogo em andamento ou se for portátil)
    pub pode_instalar: bool,
    pub motivo_bloqueio: Option<String>,
    pub portatil: bool,
}

/// Modo portátil: existe o marcador `portatil.txt` ao lado do executável.
pub fn e_portatil() -> bool {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("portatil.txt").exists()))
        .unwrap_or(false)
}

/// Motivo pelo qual não se pode instalar agora (ou `None` se pode).
fn motivo_bloqueio() -> Option<String> {
    if xablau_core::diario::diario_aberto() {
        return Some("Há uma operação em andamento. A atualização será feita depois que terminar.".into());
    }
    if jogo_aberto() {
        return Some("Um jogo está aberto. Feche o jogo para atualizar.".into());
    }
    None
}

fn jogo_aberto() -> bool {
    let cfg = xablau_core::config::carregar();
    xablau_core::config::ler_status_vigia().and_then(|s| s.jogo_aberto).is_some()
        || {
            let proc = xablau_core::processos::instantaneo();
            cfg.jogos.iter().any(|j| proc.iter().any(|p| p.nome.eq_ignore_ascii_case(j)))
        }
}

/// Checa se há atualização. Nunca instala — só informa.
pub async fn checar(app: &AppHandle) -> Result<InfoAtualizacao, String> {
    let atual = xablau_core::VERSAO.to_string();
    let portatil = e_portatil();

    let updater = match app.updater() {
        Ok(u) => u,
        Err(_) => {
            // Atualizador não configurado (compilado sem chaves): informa sem erro ruidoso.
            return Ok(InfoAtualizacao {
                disponivel: false,
                versao_atual: atual,
                versao_nova: None,
                notas: None,
                data: None,
                pode_instalar: false,
                motivo_bloqueio: Some("Atualização automática não está configurada nesta compilação.".into()),
                portatil,
            });
        }
    };

    match updater.check().await {
        Ok(Some(update)) => {
            let bloqueio = if portatil {
                Some("Versão portátil: baixe a nova versão manualmente na página de Releases.".into())
            } else {
                motivo_bloqueio()
            };
            Ok(InfoAtualizacao {
                disponivel: true,
                versao_atual: atual,
                versao_nova: Some(update.version.clone()),
                notas: update.body.clone(),
                data: update.date.map(|d| d.to_string()),
                pode_instalar: bloqueio.is_none() && !portatil,
                motivo_bloqueio: bloqueio,
                portatil,
            })
        }
        Ok(None) => Ok(InfoAtualizacao {
            disponivel: false,
            versao_atual: atual,
            versao_nova: None,
            notas: None,
            data: None,
            pode_instalar: false,
            motivo_bloqueio: None,
            portatil,
        }),
        Err(e) => Err(format!("Não foi possível verificar atualizações: {e}")),
    }
}

/// Baixa e instala a atualização (com as travas de segurança). No Windows, o app fecha para instalar.
pub async fn instalar(app: &AppHandle) -> Result<(), String> {
    if e_portatil() {
        return Err("A versão portátil não se atualiza sozinha. Baixe a nova versão na página de Releases.".into());
    }
    if let Some(motivo) = motivo_bloqueio() {
        return Err(motivo);
    }
    let updater = app.updater().map_err(|_| "Atualização automática não está configurada.".to_string())?;
    let update = updater
        .check()
        .await
        .map_err(|e| format!("Não foi possível verificar a atualização: {e}"))?
        .ok_or("Nenhuma atualização disponível.")?;

    xablau_core::log::info("interface", &format!("Instalando atualização para a versão {}", update.version));
    update
        .download_and_install(|_baixado, _total| {}, || {})
        .await
        .map_err(|e| format!("Falha ao baixar/instalar a atualização: {e}"))?;
    Ok(())
}
