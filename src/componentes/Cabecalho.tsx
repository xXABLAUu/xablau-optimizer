// Cabeçalho: chave Básico/Avançado, Modo Turbo, status do vigia e atalho "O que há de novo".
import { Zap, Rocket, PartyPopper } from 'lucide-react';
import { usarApp } from '@/store/app';
import { cmd } from '@/lib/comandos';
import { Interruptor } from './basicos';

export function Cabecalho({ aoAbrirNovidades }: { aoAbrirNovidades: () => void }) {
  const modo = usarApp((e) => e.modo);
  const definirModo = usarApp((e) => e.definirModo);
  const turbo = usarApp((e) => e.turbo);
  const setTurbo = usarApp((e) => e.setTurbo);
  const vigia = usarApp((e) => e.vigia);
  const abaAtual = usarApp((e) => e.abaAtual);

  const alternarTurbo = async () => {
    const novo = !turbo;
    setTurbo(novo);
    try { await cmd.definirTurbo(novo); } catch { setTurbo(!novo); }
  };

  return (
    <header className="h-16 shrink-0 flex items-center justify-between gap-4 px-6 border-b border-borda/60 bg-superficie/40 backdrop-blur">
      <h1 className="text-lg font-bold capitalize">{nomeAba(abaAtual)}</h1>

      <div className="flex items-center gap-3">
        {vigia && (
          <div className="hidden md:flex items-center gap-2 text-xs text-suave mr-1">
            <span className="h-2 w-2 rounded-full animate-brilhar" style={{ background: vigia.pausado ? 'rgb(var(--c-amarelo))' : 'rgb(var(--c-verde))' }} />
            {vigia.pausado ? 'Automações pausadas' : vigia.regra_ativa}
          </div>
        )}

        <button onClick={aoAbrirNovidades} className="botao !px-2.5" title="O que há de novo" aria-label="O que há de novo">
          <PartyPopper size={16} />
        </button>

        <button onClick={alternarTurbo} className={`botao ${turbo ? 'botao-primario' : ''}`} title="Modo Turbo">
          {turbo ? <Rocket size={16} /> : <Zap size={16} />} Turbo
        </button>

        <div className="flex items-center gap-2 pl-3 border-l border-borda/70">
          <span className={`text-xs font-semibold ${modo === 'basico' ? 'text-texto' : 'text-suave'}`}>Básico</span>
          <Interruptor ligado={modo === 'avancado'} aoMudar={(v) => definirModo(v ? 'avancado' : 'basico')} rotulo="Alternar modo" />
          <span className={`text-xs font-semibold ${modo === 'avancado' ? 'text-texto' : 'text-suave'}`}>Avançado</span>
        </div>
      </div>
    </header>
  );
}

function nomeAba(aba: string): string {
  const m: Record<string, string> = {
    inicio: 'Início', diagnostico: 'Diagnóstico', desempenho: 'Desempenho',
    limpeza: 'Limpeza', sistema: 'Sistema', seguranca: 'Segurança', config: 'Configurações',
  };
  return m[aba] ?? aba;
}
