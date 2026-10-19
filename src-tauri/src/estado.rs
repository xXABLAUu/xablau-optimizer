//! Estado compartilhado do backend (catálogo, detecção de hardware em cache) e tipos de resposta.

use crate::catalogo::Catalogo;
use std::sync::Mutex;

/// Estado mantido pelo Tauri (via `app.manage`).
pub struct Estado {
    pub catalogo: Catalogo,
    /// Cache da topologia do processador (não muda durante a execução).
    pub topologia: xablau_core::topologia::Topologia,
    /// Última checagem de atualização (para a interface consultar sem rechecar).
    pub atualizacao: Mutex<Option<crate::atualizador::InfoAtualizacao>>,
}

impl Estado {
    pub fn novo() -> Result<Self, String> {
        Ok(Self {
            catalogo: Catalogo::carregar()?,
            topologia: xablau_core::topologia::ler(),
            atualizacao: Mutex::new(None),
        })
    }
}
