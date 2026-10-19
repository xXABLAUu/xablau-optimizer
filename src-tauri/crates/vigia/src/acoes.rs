//! Ações que o menu da bandeja envia ao loop principal do vigia.

#[derive(Debug, Clone)]
pub enum Acao {
    AbrirInterface,
    AbrirKit,
    AlternarTurbo,
    AlternarPausa,
    AlternarPerfil(String),
    Encerrar,
}
