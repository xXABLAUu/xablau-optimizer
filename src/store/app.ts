// Estado global da interface (Zustand): configurações, tema, modo, aba atual, status do vigia.
import { create } from 'zustand';
import { cmd } from '@/lib/comandos';
import type { Configuracoes, Modo, Sobre, StatusVigia } from '@/lib/tipos';

export type IdTema =
  | 'escuro' | 'claro' | 'amoled' | 'neon' | 'oceano'
  | 'floresta' | 'por-do-sol' | 'grafite' | 'roxo-xablau' | 'alto-contraste';

export interface PreferenciasInterface {
  tema: IdTema;
  escala: number;      // 0.85 a 1.3
  densidade: number;   // 0.85, 1, 1.15
  animacoes: 'normais' | 'reduzidas';
  fonte: 'inter' | 'sistema';
}

export const TEMAS: { id: IdTema; nome: string; amostra: [string, string] }[] = [
  { id: 'escuro', nome: 'Escuro', amostra: ['#0d0c16', '#a75cf6'] },
  { id: 'claro', nome: 'Claro', amostra: ['#f4f4fa', '#7c3aed'] },
  { id: 'amoled', nome: 'AMOLED', amostra: ['#000000', '#b46eff'] },
  { id: 'neon', nome: 'Neon', amostra: ['#0b0818', '#f43fb4'] },
  { id: 'oceano', nome: 'Oceano', amostra: ['#081420', '#2dd4bf'] },
  { id: 'floresta', nome: 'Floresta', amostra: ['#0c1610', '#4ade80'] },
  { id: 'por-do-sol', nome: 'Pôr do sol', amostra: ['#1a0e14', '#fb923c'] },
  { id: 'grafite', nome: 'Grafite', amostra: ['#121214', '#94a3b8'] },
  { id: 'roxo-xablau', nome: 'Roxo Xablau', amostra: ['#140a22', '#c084fc'] },
  { id: 'alto-contraste', nome: 'Alto contraste', amostra: ['#000000', '#ffea00'] },
];

const PREFS_PADRAO: PreferenciasInterface = {
  tema: 'escuro', escala: 1, densidade: 1, animacoes: 'normais', fonte: 'inter',
};

interface EstadoApp {
  carregado: boolean;
  sobre: Sobre | null;
  config: Configuracoes | null;
  prefs: PreferenciasInterface;
  modo: Modo;
  abaAtual: string;
  vigia: StatusVigia | null;
  turbo: boolean;
  buscaAberta: boolean;

  iniciar: () => Promise<void>;
  aplicarPrefs: (p: Partial<PreferenciasInterface>) => Promise<void>;
  definirModo: (m: Modo) => Promise<void>;
  irParaAba: (aba: string) => void;
  setBusca: (aberta: boolean) => void;
  recarregarConfig: () => Promise<void>;
  salvarConfig: (c: Configuracoes) => Promise<void>;
  setVigia: (v: StatusVigia | null) => void;
  setTurbo: (t: boolean) => void;
}

function aplicarNoDocumento(p: PreferenciasInterface) {
  const r = document.documentElement;
  r.setAttribute('data-tema', p.tema);
  r.setAttribute('data-animacoes', p.animacoes);
  r.style.setProperty('--escala', String(p.escala));
  r.style.setProperty('--densidade', String(p.densidade));
  r.style.setProperty(
    '--fonte-app',
    p.fonte === 'sistema'
      ? "'Segoe UI Variable Text','Segoe UI',system-ui,sans-serif"
      : "'Inter Variable','Segoe UI Variable Text','Segoe UI',system-ui,sans-serif",
  );
}

function lerPrefs(config: Configuracoes | null): PreferenciasInterface {
  const bruto = (config?.interface ?? {}) as Partial<PreferenciasInterface>;
  return { ...PREFS_PADRAO, ...bruto };
}

export const usarApp = create<EstadoApp>((set, get) => ({
  carregado: false,
  sobre: null,
  config: null,
  prefs: PREFS_PADRAO,
  modo: 'basico',
  abaAtual: 'inicio',
  vigia: null,
  turbo: false,
  buscaAberta: false,

  iniciar: async () => {
    const [sobre, config, turbo] = await Promise.all([cmd.sobre(), cmd.carregarConfig(), cmd.turboAtivo()]);
    const prefs = lerPrefs(config);
    aplicarNoDocumento(prefs);
    set({ sobre, config, prefs, modo: config.modo, turbo, carregado: true });
  },

  aplicarPrefs: async (p) => {
    const prefs = { ...get().prefs, ...p };
    aplicarNoDocumento(prefs);
    set({ prefs });
    const config = get().config;
    if (config) {
      const novo = { ...config, interface: { ...config.interface, ...prefs } };
      set({ config: novo });
      await cmd.salvarConfig(novo);
    }
  },

  definirModo: async (m) => {
    set({ modo: m });
    const config = get().config;
    if (config) {
      const novo = { ...config, modo: m };
      set({ config: novo });
      await cmd.salvarConfig(novo);
    }
  },

  irParaAba: (aba) => set({ abaAtual: aba, buscaAberta: false }),
  setBusca: (aberta) => set({ buscaAberta: aberta }),

  recarregarConfig: async () => {
    const config = await cmd.carregarConfig();
    set({ config, modo: config.modo, prefs: lerPrefs(config) });
  },

  salvarConfig: async (c) => {
    set({ config: c, modo: c.modo });
    await cmd.salvarConfig(c);
  },

  setVigia: (v) => set({ vigia: v }),
  setTurbo: (t) => set({ turbo: t }),
}));
