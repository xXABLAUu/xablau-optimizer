// Lista FECHADA e tipada de chamadas ao backend. A interface só fala com o Rust por aqui.
import { invoke } from './tauri';
import type {
  Ajuste, Configuracoes, Culpado, Hardware, InfoAtualizacao, Operacao, RelatorioVerificacoes,
  ResultadoAplicacao, ResultadoDesfazer, ResultadoPonto, Revertida, SaudeSsd, Simulacao, Sobre,
  StatusVigia, Topologia, UsoTempoReal,
} from './tipos';

export const cmd = {
  sobre: () => invoke<Sobre>('sobre'),
  hardware: () => invoke<Hardware>('obter_hardware'),
  topologia: () => invoke<Topologia>('obter_topologia'),
  usoTempoReal: () => invoke<UsoTempoReal>('uso_tempo_real'),
  culpados: (limite?: number) => invoke<Culpado[]>('monitor_culpados', { limite }),
  saudeSsds: () => invoke<SaudeSsd[]>('saude_ssds'),
  verificacoes: () => invoke<RelatorioVerificacoes>('verificacoes_previas'),
  fichaPc: (markdown: boolean) => invoke<string>('ficha_pc', { markdown }),

  listarAjustes: () => invoke<Ajuste[]>('listar_ajustes'),
  simular: (ids: string[]) => invoke<Simulacao>('simular', { ids }),
  aplicar: (titulo: string, ids: string[]) => invoke<ResultadoAplicacao>('aplicar_ajustes', { titulo, ids }),
  criarPonto: (descricao: string) => invoke<ResultadoPonto>('criar_ponto_restauracao', { descricao }),

  operacoes: () => invoke<Operacao[]>('listar_operacoes'),
  interrompidas: () => invoke<Operacao[]>('operacoes_interrompidas'),
  desfazerTudo: () => invoke<[string, ResultadoDesfazer][]>('desfazer_tudo'),
  desfazerOperacao: (opId: string) => invoke<ResultadoDesfazer>('desfazer_operacao', { opId }),
  desfazerItem: (opId: string, itemId: number) => invoke<ResultadoDesfazer>('desfazer_item', { opId, itemId }),
  conflitos: (opId: string, itemId: number) => invoke<string[]>('conflitos_do_item', { opId, itemId }),
  encerrarInterrompida: (opId: string) => invoke<void>('encerrar_interrompida', { opId }),
  revertidas: () => invoke<Revertida[]>('alteracoes_revertidas'),

  logRecente: (linhas?: number) => invoke<string[]>('log_recente', { linhas }),
  exportarLog: (pasta: string) => invoke<string>('exportar_log', { pasta }),

  carregarConfig: () => invoke<Configuracoes>('carregar_config'),
  salvarConfig: (cfg: Configuracoes) => invoke<void>('salvar_config', { cfg }),

  statusVigia: () => invoke<StatusVigia | null>('status_vigia'),
  definirTurbo: (ativo: boolean) => invoke<void>('definir_turbo', { ativo }),
  turboAtivo: () => invoke<boolean>('turbo_ativo'),
  pausarAutomacoes: (pausar: boolean) => invoke<void>('pausar_automacoes', { pausar }),

  checarAtualizacao: () => invoke<InfoAtualizacao>('checar_atualizacao'),
  instalarAtualizacao: () => invoke<void>('instalar_atualizacao'),
  mostrarJanela: () => invoke<void>('mostrar_janela'),
};
