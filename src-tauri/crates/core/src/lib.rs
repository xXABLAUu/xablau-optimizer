//! Núcleo compartilhado do Xablau Optimizer.
//!
//! Usado pelos três executáveis do projeto:
//! - **interface** (Tauri): planeja, simula e aplica ajustes;
//! - **vigia** (bandeja): perfis por aplicativo, Modo Jogo, IdleSaver e Watchdog;
//! - **kit de emergência** "Xablau — Restaurar tudo": desfaz alterações sem depender da
//!   interface (funciona até no Modo de Segurança).
//!
//! Regra de ouro: nunca quebrar o sistema, nunca corromper arquivos, tudo reversível.

pub mod aplicador;
pub mod caminhos;
pub mod comando;
pub mod config;
pub mod diario;
pub mod energia;
pub mod log;
pub mod mudancas;
pub mod processos;
pub mod protecao;
pub mod registro;
pub mod restauracao;
pub mod servicos;
pub mod sistema;
pub mod tarefas;
pub mod topologia;
pub mod verificacoes;
pub mod util;

/// Nome oficial do produto.
pub const NOME_PRODUTO: &str = "Xablau Optimizer";
/// Criador do programa.
pub const CRIADOR: &str = "Xablau Studios — Luiz";
/// Versão do programa (sincronizada com o package.json por scripts/versao.mjs).
pub const VERSAO: &str = env!("CARGO_PKG_VERSION");
