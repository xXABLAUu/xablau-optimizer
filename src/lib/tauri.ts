// Ponte fina com o backend Tauri. Em desenvolvimento no navegador (sem Tauri), devolve dados
// de exemplo para a interface poder ser trabalhada; dentro do app, chama o Rust de verdade.
import { invoke as invokeTauri } from '@tauri-apps/api/core';

export const dentroDoTauri = (): boolean =>
  typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

export async function invoke<T>(comando: string, args?: Record<string, unknown>): Promise<T> {
  if (!dentroDoTauri()) {
    return exemplo<T>(comando, args);
  }
  return invokeTauri<T>(comando, args);
}

// ---- Dados de exemplo (só fora do app, para desenvolver a interface) ----
function exemplo<T>(comando: string, _args?: Record<string, unknown>): Promise<T> {
  const mapa: Record<string, unknown> = {
    sobre: { nome: 'Xablau Optimizer', criador: 'Xablau Studios — Luiz', versao: '1.0.0', portatil: false, admin: true, modo_seguranca: false },
    obter_hardware: {
      cpu: 'AMD Ryzen 9 5900XT 16-Core', nucleos_fisicos: 16, nucleos_logicos: 32, ccds: 2,
      gpu: ['NVIDIA GeForce RTX 5070'], placa_mae: 'ASUS TUF GAMING B550', bios: '3405 (2024-11-02)',
      ram_total_gb: 31.9, ram_velocidade_mhz: 3200, ram_modo: 'XMP/DOCP provavelmente ativo',
      discos: ['Kingston SNV2S1000G', 'Kingdian S280 240GB'], windows: 'Windows 11 (build 26100.2314)', notebook: false,
    },
    uso_tempo_real: { cpu_pct: 14 + Math.random() * 20, ram_pct: 46, ram_usada_gb: 14.7, ram_total_gb: 31.9, na_bateria: false, bateria_pct: null },
    turbo_ativo: false,
    carregar_config: { versao_esquema: 1, modo: 'basico', assistente_concluido: false, uso_principal: 'geral', ultima_versao_vista: '', interface: {}, automacao: { iniciar_com_windows: false, vigia_ativo: true, perfis_ativos: true, modo_jogo_auto: true, idlesaver: { ativo: false, minutos: 10, cpu_max: 12, gpu_max: 10, plano_ocioso: 'equilibrado', plano_uso: 'maximo' }, watchdog_ativo: false, regras_watchdog: [], manutencao_ociosa: false, inicio_atrasado: [], intervalo_inicio_atrasado_s: 8, avisar_reversoes: true, notificacoes: true }, perfis: [], jogos: [], drivers: { video: 'manual' }, alertas_ssd: { desgaste_pct: 80, temperatura_c: 70 }, atualizacao: { automatica_no_basico: true, lembrar_depois_ate: null } },
    listar_ajustes: [],
    status_vigia: null,
    listar_operacoes: [],
    operacoes_interrompidas: [],
    alteracoes_revertidas: [],
    monitor_culpados: [],
    saude_ssds: [],
    verificacoes_previas: { itens: [], versao_windows: { major: 10, minor: 0, build: 26100, ubr: 2314, windows11: true }, energia: { na_tomada: true, tem_bateria: false, bateria_pct: null, economia_de_bateria: false }, administrador: true, modo_seguranca: false, espaco_livre_gb: 312.4, pode_prosseguir: true },
    checar_atualizacao: { disponivel: false, versao_atual: '1.0.0', versao_nova: null, notas: null, data: null, pode_instalar: false, motivo_bloqueio: null, portatil: false },
  };
  return Promise.resolve((mapa[comando] ?? null) as T);
}
