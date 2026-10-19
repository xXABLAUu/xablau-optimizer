//! Xablau — Restaurar tudo (kit de emergência).
//!
//! Programa de console separado, em Rust nativo, SEM WebView2 e sem depender da interface.
//! Lê o diário e os backups em %ProgramData%\XablauOptimizer e desfaz tudo ou item a item.
//! Também abre a Restauração do Sistema do Windows (rstrui). Funciona no Modo de Segurança —
//! lá não dá para criar ponto de restauração, mas restaurar chaves do registro e configuração
//! de serviços funciona normalmente.

#![cfg_attr(not(debug_assertions), windows_subsystem = "console")]

use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use std::time::Duration;
use xablau_core::diario::{self, EstadoItem, Operacao, ResultadoDesfazer};
use xablau_core::{comando, log, sistema};

const ORIGEM: &str = "kit";

fn ler_linha(prompt: &str) -> String {
    print!("{prompt}");
    let _ = io::stdout().flush();
    let mut linha = String::new();
    let _ = io::stdin().lock().read_line(&mut linha);
    linha.trim().to_string()
}

fn pausar() {
    let _ = ler_linha("\nPressione Enter para continuar...");
}

fn confirmar(pergunta: &str) -> bool {
    matches!(ler_linha(&format!("{pergunta} (s/N): ")).to_lowercase().as_str(), "s" | "sim" | "y" | "yes")
}

fn titulo() {
    println!("============================================================");
    println!("   Xablau — Restaurar tudo  (kit de emergência)");
    println!("   Criado por Xablau Studios — Luiz   |   versão {}", xablau_core::VERSAO);
    println!("============================================================");
    if sistema::modo_seguranca() {
        println!("  [Modo de Segurança detectado]");
        println!("  Restaurar registro, serviços e configurações funciona normalmente.");
        println!("  (Criar novo ponto de restauração não funciona no Modo de Segurança.)");
    }
    if !sistema::e_administrador() {
        println!("  ATENÇÃO: o programa não está como administrador. Feche e abra como administrador");
        println!("  para conseguir restaurar tudo.");
    }
    println!();
}

fn resumo_operacao(op: &Operacao) -> String {
    let feitas = op.itens.iter().filter(|i| i.estado == EstadoItem::Feita).count();
    let pendentes = op.itens.iter().filter(|i| i.estado == EstadoItem::Pendente).count();
    let desfeitas = op.itens.iter().filter(|i| i.estado == EstadoItem::Desfeita).count();
    let marca = if op.incompleta() { " [INTERROMPIDA]" } else { "" };
    format!(
        "{} — {}{}  ({} aplicadas, {} pendentes, {} já desfeitas, {} podem ser desfeitas)",
        op.inicio, op.titulo, marca, feitas, pendentes, desfeitas, op.desfaziveis()
    )
}

fn mostrar_resultado(r: &ResultadoDesfazer) {
    println!("\n  Desfeitas: {}   |   Ignoradas: {}   |   Falhas: {}", r.desfeitas, r.ignoradas, r.falhas.len());
    for f in &r.falhas {
        println!("   • {f}");
    }
    if !r.falhas.is_empty() {
        println!("\n  Dica: se alguma coisa não voltou, use a opção 4 (Restauração do Sistema do Windows).");
    }
}

fn listar(operacoes: &[Operacao]) {
    if operacoes.is_empty() {
        println!("  Nenhuma alteração registrada. Nada a desfazer. 🎉");
        return;
    }
    for (i, op) in operacoes.iter().enumerate() {
        println!("  [{}] {}", i + 1, resumo_operacao(op));
    }
}

fn desfazer_tudo() {
    println!("\nIsto vai desfazer TODAS as alterações que o Xablau Optimizer já fez,");
    println!("da mais recente para a mais antiga.");
    if !confirmar("Tem certeza") {
        println!("Cancelado.");
        return;
    }
    match diario::desfazer_tudo(ORIGEM) {
        Ok(resultados) => {
            let mut total = ResultadoDesfazer::default();
            for (_, r) in &resultados {
                total.desfeitas += r.desfeitas;
                total.ignoradas += r.ignoradas;
                total.falhas.extend(r.falhas.clone());
            }
            println!("\nConcluído.");
            mostrar_resultado(&total);
        }
        Err(e) => println!("\nNão foi possível desfazer: {e}"),
    }
}

fn desfazer_uma(operacoes: &[Operacao]) {
    let escolha = ler_linha("\nNúmero da alteração para desfazer (ou Enter para voltar): ");
    if escolha.is_empty() {
        return;
    }
    let Ok(n) = escolha.parse::<usize>() else {
        println!("Número inválido.");
        return;
    };
    let Some(op) = operacoes.get(n.wrapping_sub(1)) else {
        println!("Número fora da lista.");
        return;
    };
    println!("\n{}", resumo_operacao(op));
    for item in &op.itens {
        let marca = match item.estado {
            EstadoItem::Desfeita => "✓ desfeita",
            EstadoItem::Falhou => "✗ falhou ao aplicar",
            EstadoItem::FalhaAoDesfazer => "! falhou ao desfazer",
            EstadoItem::Pendente => "• pendente",
            EstadoItem::Feita => "• aplicada",
        };
        println!("    [{:>2}] {}  — {}", item.id, item.descricao, marca);
    }
    let sub = ler_linha("\n(T) desfazer tudo desta operação, número do item, ou Enter para voltar: ");
    if sub.eq_ignore_ascii_case("t") {
        if confirmar("Desfazer toda esta operação") {
            match diario::desfazer_operacao(&op.id, ORIGEM) {
                Ok(r) => mostrar_resultado(&r),
                Err(e) => println!("Erro: {e}"),
            }
        }
    } else if let Ok(item_id) = sub.parse::<u32>() {
        let conflitos = diario::conflitos_posteriores(&op.id, item_id);
        if !conflitos.is_empty() {
            println!("\n  Atenção: este ajuste foi alterado depois por outras operações:");
            for c in &conflitos {
                println!("   • {c}");
            }
            if !confirmar("Desfazer mesmo assim") {
                return;
            }
        }
        match diario::desfazer_item(&op.id, item_id, ORIGEM) {
            Ok(r) => mostrar_resultado(&r),
            Err(e) => println!("Erro: {e}"),
        }
    }
}

fn abrir_restauracao_sistema() {
    println!("\nAbrindo a Restauração do Sistema do Windows (rstrui)...");
    println!("Escolha um ponto criado pelo Xablau Optimizer e siga o assistente.");
    let rstrui = sistema::dir_sistema32().join("rstrui.exe");
    match comando::executar(&rstrui, &[], Duration::from_secs(10)) {
        Ok(_) => {}
        Err(e) => println!("Não foi possível abrir a Restauração do Sistema: {e}"),
    }
}

fn mostrar_pasta_dados() {
    let base: PathBuf = xablau_core::caminhos::base();
    println!("\n  Diário e backups ficam em:");
    println!("    {}", base.display());
    println!("  (Essa pasta continua lá mesmo depois de desinstalar o programa.)");
}

fn menu() {
    loop {
        let operacoes = diario::listar_operacoes();
        let incompletas = diario::operacoes_incompletas();
        println!("\n------------------------------------------------------------");
        if !incompletas.is_empty() {
            println!("  ⚠ Há {} operação(ões) INTERROMPIDA(S) (o PC pode ter desligado no meio).", incompletas.len());
        }
        println!("  O que você quer fazer?\n");
        println!("   1) Ver todas as alterações registradas");
        println!("   2) Desfazer TUDO (volta tudo ao estado anterior)");
        println!("   3) Desfazer uma operação ou item específico");
        println!("   4) Abrir a Restauração do Sistema do Windows");
        println!("   5) Onde ficam o diário e os backups");
        println!("   0) Sair");
        match ler_linha("\n  Opção: ").as_str() {
            "1" => {
                println!();
                listar(&operacoes);
                pausar();
            }
            "2" => desfazer_tudo(),
            "3" => {
                if operacoes.is_empty() {
                    println!("\n  Nenhuma alteração registrada.");
                } else {
                    listar(&operacoes);
                    desfazer_uma(&operacoes);
                }
            }
            "4" => abrir_restauracao_sistema(),
            "5" => mostrar_pasta_dados(),
            "0" | "sair" | "q" => break,
            _ => println!("  Opção inválida."),
        }
    }
}

fn main() {
    // No Modo de Segurança o log ainda funciona (grava em %ProgramData%).
    log::info(ORIGEM, "Kit de emergência aberto.");
    titulo();

    let incompletas = diario::operacoes_incompletas();
    if !incompletas.is_empty() {
        println!("Foram encontradas operações interrompidas:");
        for op in &incompletas {
            println!("  • {}", resumo_operacao(op));
        }
        println!("\nIsso costuma acontecer quando o PC desliga no meio de uma otimização.");
        if confirmar("Deseja desfazer agora essas operações interrompidas") {
            for op in &incompletas {
                match diario::desfazer_operacao(&op.id, ORIGEM) {
                    Ok(r) => {
                        println!("\n{}", op.titulo);
                        mostrar_resultado(&r);
                    }
                    Err(e) => println!("Erro ao desfazer {}: {e}", op.titulo),
                }
            }
        } else if confirmar("Manter as alterações e marcar como concluídas") {
            for op in &incompletas {
                let _ = diario::encerrar_incompleta(&op.id, ORIGEM);
            }
        }
    }

    menu();
    log::info(ORIGEM, "Kit de emergência fechado.");
    println!("\nAté logo!");
}
