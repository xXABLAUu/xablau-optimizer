// Tipos espelhando as estruturas do backend (Rust). Mantidos em português, como o resto do app.

export type Risco = 'verde' | 'amarelo' | 'vermelho';
export type Modo = 'basico' | 'avancado';
export type UsoPrincipal = 'jogos' | 'edicao' | 'geral';

export interface Sobre {
  nome: string; criador: string; versao: string;
  portatil: boolean; admin: boolean; modo_seguranca: boolean;
}

export interface Hardware {
  cpu: string; nucleos_fisicos: number; nucleos_logicos: number; ccds: number;
  gpu: string[]; placa_mae: string; bios: string;
  ram_total_gb: number; ram_velocidade_mhz: number; ram_modo: string;
  discos: string[]; windows: string; notebook: boolean;
}

export interface UsoTempoReal {
  cpu_pct: number; ram_pct: number; ram_usada_gb: number; ram_total_gb: number;
  na_bateria: boolean; bateria_pct: number | null;
}

export interface SaibaMais { o_que: string; como: string; beneficio: string; risco: string; }
export interface Ajuste {
  id: string; titulo: string; descricao: string; saiba_mais: SaibaMais;
  risco: Risco; aba: string; secao: string; modos: Modo[]; busca: string[];
  aplicavel: boolean; aplicado: boolean | null;
}

export interface MudancaPlanejada { ajuste: string; titulo: string; descricao: string; }
export interface Simulacao {
  mudancas: MudancaPlanejada[]; sem_efeito: string[]; erros: [string, string][]; total: number;
}
export interface ResultadoPonto {
  criado: boolean; numero_sequencia: number | null; descricao: string;
  mensagem: string; protecao_ligada_agora: boolean;
}
export interface ResultadoAplicacao {
  op_id: string; aplicadas: number; falhas: string[];
  ponto_restauracao: ResultadoPonto; status: string;
}

export type NivelVerificacao = 'ok' | 'aviso' | 'bloqueio';
export interface Verificacao { chave: string; titulo: string; nivel: NivelVerificacao; detalhe: string; }
export interface RelatorioVerificacoes {
  itens: Verificacao[]; administrador: boolean; modo_seguranca: boolean;
  espaco_livre_gb: number; pode_prosseguir: boolean;
  energia: { na_tomada: boolean; tem_bateria: boolean; bateria_pct: number | null; economia_de_bateria: boolean };
  versao_windows: { major: number; minor: number; build: number; ubr: number; windows11: boolean };
}

export interface Culpado {
  pid: number; nome: string; memoria_mb: number; es_mb: number;
  eficiencia: boolean | null; protegido: boolean;
}
export interface SaudeSsd {
  modelo: string; tipo: string; tamanho_gb: number; confiavel: boolean;
  desgaste_pct: number | null; temperatura_c: number | null;
  horas_ligado: number | null; erros: number | null; aviso: string | null;
}

export type EstadoItem = 'pendente' | 'feita' | 'falhou' | 'desfeita' | 'falha_ao_desfazer';
export interface ItemMudanca {
  id: number; ajuste: string; descricao: string; estado: EstadoItem;
  erro: string | null; quando: string; desfeito_em: string | null; reversivel: boolean;
}
export interface Operacao {
  id: string; titulo: string; origem: string; versao: string;
  inicio: string; fim: string | null; status: string | null;
  ponto_restauracao: string | null; itens: ItemMudanca[];
}
export interface ResultadoDesfazer { desfeitas: number; ignoradas: number; falhas: string[]; }
export interface Revertida { op_id: string; item_id: number; ajuste: string; descricao: string; }

export interface InfoAtualizacao {
  disponivel: boolean; versao_atual: string; versao_nova: string | null;
  notas: string | null; data: string | null;
  pode_instalar: boolean; motivo_bloqueio: string | null; portatil: boolean;
}

export interface StatusVigia {
  pid: number; versao: string; iniciado: string; atualizado: string;
  turbo: boolean; pausado: boolean; regra_ativa: string; plano_atual: string;
  perfis_em_uso: string[]; jogo_aberto: string | null; ocioso: boolean; avisos: string[];
}

// Configurações (espelho de config.rs; só o que a interface edita diretamente aqui)
export interface Configuracoes {
  versao_esquema: number;
  modo: Modo;
  assistente_concluido: boolean;
  uso_principal: UsoPrincipal;
  ultima_versao_vista: string;
  interface: Record<string, unknown>;
  automacao: Automacao;
  perfis: PerfilApp[];
  jogos: string[];
  drivers: { video: 'manual' | 'automatico' | 'ignorar' };
  alertas_ssd: { desgaste_pct: number; temperatura_c: number };
  atualizacao: { automatica_no_basico: boolean; lembrar_depois_ate: string | null };
}
export interface Automacao {
  iniciar_com_windows: boolean; vigia_ativo: boolean; perfis_ativos: boolean;
  modo_jogo_auto: boolean; idlesaver: IdleSaver; watchdog_ativo: boolean;
  regras_watchdog: RegraWatchdog[]; manutencao_ociosa: boolean;
  inicio_atrasado: ItemInicioAtrasado[]; intervalo_inicio_atrasado_s: number;
  avisar_reversoes: boolean; notificacoes: boolean;
}
export interface IdleSaver {
  ativo: boolean; minutos: number; cpu_max: number; gpu_max: number;
  plano_ocioso: PlanoAlvo; plano_uso: PlanoAlvo;
}
export type PlanoAlvo = 'maximo' | 'alto' | 'equilibrado' | 'economia' | 'manter';
export type Prioridade = 'ociosa' | 'abaixo_do_normal' | 'normal' | 'acima_do_normal' | 'alta';
export type PrioridadeEs = 'muito_baixa' | 'baixa' | 'normal';
export type ModoNucleos = 'todos' | 'grupo_cache' | 'desempenho';
export interface Nucleos { modo: ModoNucleos; grupo: number; rigido: boolean; }
export interface PerfilApp {
  id: string; nome: string; executaveis: string[]; ativo: boolean; e_jogo: boolean;
  remover_eficiencia: boolean; incluir_filhos: boolean; prioridade: Prioridade;
  plano: PlanoAlvo; nucleos: Nucleos; prioridade_es: PrioridadeEs;
  prioridade_memoria: number; reduzir_segundo_plano: boolean;
}
export type RecursoWatchdog = 'cpu' | 'ram' | 'disco';
export type AcaoWatchdog = 'baixar_prioridade' | 'cpu_set' | 'prioridade_es_baixa' | 'notificar' | 'encerrar';
export interface RegraWatchdog {
  id: string; nome: string; ativo: boolean; processo: string; recurso: RecursoWatchdog;
  limite: number; duracao_s: number; acao: AcaoWatchdog; grupo_cache: number;
  espera_s: number; encerrar_confirmado: boolean;
}
export interface ItemInicioAtrasado { id: string; nome: string; comando: string; origem: string; ordem: number; }

export interface Topologia {
  logicos: number; nucleos: number; smt: boolean; hibrido: boolean;
  grupos: { indice: number; cpu_sets: number[]; logicos: number; nucleos: number; l3_bytes: number }[];
}

// Changelog ("O que há de novo")
export interface VersaoChangelog {
  versao: string; data: string; titulo: string; destaques: string[];
  novidades: { area: string; itens: string[] }[]; correcoes: string[];
}
