//! Catálogo de ajustes: lê o `catalogo/ajustes.json` (embutido no executável), expõe os textos
//! para a interface e transforma cada ação declarativa numa `Mudanca` reversível, já capturada
//! com o estado real (antes → depois). Nada aqui monta comandos a partir de texto livre do usuário.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use xablau_core::mudancas::{self, Mudanca};
use xablau_core::registro::ValorReg;
use xablau_core::servicos::InicioServico;
use xablau_core::{protecao, sistema};

/// O JSON do catálogo é embutido no binário (nunca baixado em tempo de execução).
const CATALOGO_JSON: &str = include_str!("../catalogo/ajustes.json");

#[derive(Debug, Clone, Deserialize)]
pub struct SaibaMais {
    pub o_que: String,
    pub como: String,
    pub beneficio: String,
    pub risco: String,
}

/// Uma ação declarativa do catálogo (vira uma `Mudanca`).
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "tipo", rename_all = "snake_case")]
pub enum Acao {
    Registro {
        chave: String,
        nome: String,
        #[serde(default)]
        valor: Option<ValorCatalogo>,
        #[serde(default)]
        apagar: bool,
    },
    Servico {
        nome: String,
        inicio: String,
    },
    Tarefa {
        caminho: String,
        ativar: bool,
    },
    Energia {
        subgrupo: String,
        config: String,
        #[serde(default = "verdadeiro")]
        ca: bool,
        valor: u32,
    },
    Hibernacao {
        ligar: bool,
    },
}

fn verdadeiro() -> bool {
    true
}

#[derive(Debug, Clone, Deserialize)]
pub struct ValorCatalogo {
    pub tipo: String,
    pub dado: serde_json::Value,
}

/// Um ajuste completo do catálogo.
#[derive(Debug, Clone, Deserialize)]
pub struct Ajuste {
    pub id: String,
    pub titulo: String,
    pub descricao: String,
    pub saiba_mais: SaibaMais,
    pub risco: String,
    pub aba: String,
    pub secao: String,
    #[serde(default)]
    pub modos: Vec<String>,
    #[serde(default)]
    pub condicoes: Vec<String>,
    #[serde(default)]
    pub busca: Vec<String>,
    pub acoes: Vec<Acao>,
}

#[derive(Debug, Clone, Deserialize)]
struct CatalogoBruto {
    ajustes: Vec<Ajuste>,
}

/// Ajuste como a interface o recebe (sem as ações internas), com status atual.
#[derive(Debug, Clone, Serialize)]
pub struct AjusteParaInterface {
    pub id: String,
    pub titulo: String,
    pub descricao: String,
    pub saiba_mais: SaibaMaisSaida,
    pub risco: String,
    pub aba: String,
    pub secao: String,
    pub modos: Vec<String>,
    pub busca: Vec<String>,
    /// O ajuste se aplica a este PC? (condições: windows11, desktop, nvidia...)
    pub aplicavel: bool,
    /// Já está aplicado? (None = não dá para saber sem aplicar)
    pub aplicado: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SaibaMaisSaida {
    pub o_que: String,
    pub como: String,
    pub beneficio: String,
    pub risco: String,
}

impl From<&SaibaMais> for SaibaMaisSaida {
    fn from(s: &SaibaMais) -> Self {
        Self { o_que: s.o_que.clone(), como: s.como.clone(), beneficio: s.beneficio.clone(), risco: s.risco.clone() }
    }
}

/// Catálogo carregado uma vez.
pub struct Catalogo {
    ajustes: Vec<Ajuste>,
    indice: HashMap<String, usize>,
}

impl Catalogo {
    pub fn carregar() -> Result<Self, String> {
        let bruto: CatalogoBruto = serde_json::from_str(CATALOGO_JSON).map_err(|e| format!("Catálogo inválido: {e}"))?;
        let mut indice = HashMap::new();
        for (i, a) in bruto.ajustes.iter().enumerate() {
            indice.insert(a.id.clone(), i);
        }
        Ok(Self { ajustes: bruto.ajustes, indice })
    }

    pub fn ajuste(&self, id: &str) -> Option<&Ajuste> {
        self.indice.get(id).map(|&i| &self.ajustes[i])
    }

    /// Lista para a interface, já com aplicabilidade e status atual.
    pub fn listar(&self) -> Vec<AjusteParaInterface> {
        let versao = sistema::versao_windows();
        let energia = sistema::status_energia();
        self.ajustes
            .iter()
            .map(|a| {
                let aplicavel = condicoes_ok(&a.condicoes, &versao, &energia);
                AjusteParaInterface {
                    id: a.id.clone(),
                    titulo: a.titulo.clone(),
                    descricao: a.descricao.clone(),
                    saiba_mais: (&a.saiba_mais).into(),
                    risco: a.risco.clone(),
                    aba: a.aba.clone(),
                    secao: a.secao.clone(),
                    modos: a.modos.clone(),
                    busca: a.busca.clone(),
                    aplicado: if aplicavel { self.esta_aplicado(a) } else { None },
                    aplicavel,
                }
            })
            .collect()
    }

    /// `Some(true)` se TODAS as ações do ajuste já estão no estado desejado.
    fn esta_aplicado(&self, a: &Ajuste) -> Option<bool> {
        let mut algum_conhecido = false;
        for acao in &a.acoes {
            match capturar(acao) {
                Ok(None) => algum_conhecido = true, // já está como desejado
                Ok(Some(_)) => return Some(false),  // falta aplicar algo
                Err(_) => {}                          // não existe neste PC / indeterminado
            }
        }
        algum_conhecido.then_some(true)
    }

    /// Captura as mudanças de uma lista de ajustes (para simular/aplicar).
    /// Devolve (planejadas, sem_efeito, erros).
    pub fn planejar(&self, ids: &[String]) -> (Vec<xablau_core::aplicador::MudancaPlanejada>, Vec<String>, Vec<(String, String)>) {
        let versao = sistema::versao_windows();
        let energia = sistema::status_energia();
        let mut planejadas = Vec::new();
        let mut sem_efeito = Vec::new();
        let mut erros = Vec::new();

        for id in ids {
            let Some(a) = self.ajuste(id) else {
                erros.push((id.clone(), "Ajuste desconhecido.".into()));
                continue;
            };
            if !condicoes_ok(&a.condicoes, &versao, &energia) {
                erros.push((a.titulo.clone(), "Este ajuste não se aplica a este PC.".into()));
                continue;
            }
            let mut teve_mudanca = false;
            for acao in &a.acoes {
                match capturar(acao) {
                    Ok(Some(m)) => {
                        teve_mudanca = true;
                        planejadas.push(xablau_core::aplicador::MudancaPlanejada {
                            ajuste: a.id.clone(),
                            titulo: a.titulo.clone(),
                            descricao: m.descrever(),
                            mudanca: m,
                        });
                    }
                    Ok(None) => {}
                    Err(e) => erros.push((a.titulo.clone(), e)),
                }
            }
            if !teve_mudanca && !erros.iter().any(|(t, _)| t == &a.titulo) {
                sem_efeito.push(a.titulo.clone());
            }
        }
        (planejadas, sem_efeito, erros)
    }
}

fn condicoes_ok(condicoes: &[String], versao: &sistema::VersaoWindows, energia: &sistema::StatusEnergia) -> bool {
    condicoes.iter().all(|c| match c.as_str() {
        "windows11" => versao.windows11,
        "windows10" => versao.major == 10 && !versao.windows11,
        "desktop" => !energia.tem_bateria,
        "notebook" => energia.tem_bateria,
        _ => true,
    })
}

fn valor_reg(v: &ValorCatalogo) -> Result<ValorReg, String> {
    match v.tipo.as_str() {
        "dword" => v.dado.as_u64().map(|n| ValorReg::Dword(n as u32)).ok_or_else(|| "dword inválido".into()),
        "qword" => v.dado.as_u64().map(ValorReg::Qword).ok_or_else(|| "qword inválido".into()),
        "sz" => v.dado.as_str().map(|s| ValorReg::Sz(s.to_string())).ok_or_else(|| "texto inválido".into()),
        "expand_sz" => v.dado.as_str().map(|s| ValorReg::ExpandSz(s.to_string())).ok_or_else(|| "texto inválido".into()),
        outro => Err(format!("tipo de registro desconhecido no catálogo: {outro}")),
    }
}

fn inicio_servico(texto: &str) -> Result<InicioServico, String> {
    Ok(match texto {
        "manual" => InicioServico::Manual,
        "automatico" => InicioServico::Automatico,
        "automatico_atrasado" => InicioServico::AutomaticoAtrasado,
        "desativado" => InicioServico::Desativado,
        outro => return Err(format!("início de serviço inválido no catálogo: {outro}")),
    })
}

/// Captura uma ação como `Mudanca` (com antes/depois reais). `Ok(None)` = já está como desejado.
fn capturar(acao: &Acao) -> Result<Option<Mudanca>, String> {
    match acao {
        Acao::Registro { chave, nome, valor, apagar } => {
            let depois = if *apagar {
                None
            } else {
                Some(valor_reg(valor.as_ref().ok_or("ação de registro sem valor")?)?)
            };
            mudancas::capturar_registro(chave, nome, depois)
        }
        Acao::Servico { nome, inicio } => {
            if protecao::servico_protegido(nome) {
                return Err(format!("O serviço {nome} está protegido."));
            }
            mudancas::capturar_servico(nome, inicio_servico(inicio)?)
        }
        Acao::Tarefa { caminho, ativar } => mudancas::capturar_tarefa(caminho, *ativar),
        Acao::Energia { subgrupo, config, ca, valor } => mudancas::capturar_energia(subgrupo, config, *ca, *valor),
        Acao::Hibernacao { ligar } => mudancas::capturar_hibernacao(*ligar),
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn catalogo_carrega_e_e_consistente() {
        let c = Catalogo::carregar().expect("catálogo deve carregar");
        assert!(!c.ajustes.is_empty());
        for a in &c.ajustes {
            assert!(!a.acoes.is_empty(), "ajuste {} sem ações", a.id);
            assert!(["verde", "amarelo", "vermelho"].contains(&a.risco.as_str()));
            // nenhuma ação de serviço pode mirar um serviço protegido
            for acao in &a.acoes {
                if let Acao::Servico { nome, .. } = acao {
                    assert!(!protecao::servico_protegido(nome), "ajuste {} mira serviço protegido {}", a.id, nome);
                }
            }
        }
        assert!(c.ajuste("privacidade.telemetria").is_some());
    }
}
