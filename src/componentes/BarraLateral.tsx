// Barra lateral: navegação entre as 7 abas. No Modo Básico, mostra menos abas.
import { motion } from 'framer-motion';
import { Home, Stethoscope, Gauge, Sparkles, Settings2, ShieldCheck, Sliders } from 'lucide-react';
import { usarApp } from '@/store/app';

const ABAS = [
  { id: 'inicio', nome: 'Início', Icone: Home, basico: true },
  { id: 'diagnostico', nome: 'Diagnóstico', Icone: Stethoscope, basico: true },
  { id: 'desempenho', nome: 'Desempenho', Icone: Gauge, basico: true },
  { id: 'limpeza', nome: 'Limpeza', Icone: Sparkles, basico: true },
  { id: 'sistema', nome: 'Sistema', Icone: Settings2, basico: false },
  { id: 'seguranca', nome: 'Segurança', Icone: ShieldCheck, basico: true },
  { id: 'config', nome: 'Configurações', Icone: Sliders, basico: true },
];

export function BarraLateral() {
  const abaAtual = usarApp((e) => e.abaAtual);
  const irParaAba = usarApp((e) => e.irParaAba);
  const modo = usarApp((e) => e.modo);
  const sobre = usarApp((e) => e.sobre);
  const abas = ABAS.filter((a) => modo === 'avancado' || a.basico);

  return (
    <aside className="w-[220px] shrink-0 h-full flex flex-col border-r border-borda/60 bg-superficie/60 backdrop-blur px-3 py-4">
      <div className="flex items-center gap-2.5 px-2 mb-6">
        <div className="grid place-items-center h-9 w-9 rounded-xl shrink-0" style={{ background: 'linear-gradient(135deg, rgb(var(--c-destaque)), rgb(var(--c-destaque2)))' }}>
          <span className="text-lg font-black text-white">X</span>
        </div>
        <div className="leading-tight">
          <div className="font-black text-sm">Xablau Optimizer</div>
          <div className="text-[10px] text-suave">v{sobre?.versao ?? '1.0.0'} · por Xablau</div>
        </div>
      </div>

      <nav className="flex-1 space-y-1">
        {abas.map((a) => {
          const ativo = abaAtual === a.id;
          return (
            <button
              key={a.id}
              onClick={() => irParaAba(a.id)}
              className="relative w-full flex items-center gap-3 px-3 py-2.5 rounded-xl text-sm font-medium transition-colors"
              style={{ color: ativo ? 'rgb(var(--c-texto))' : 'rgb(var(--c-suave))' }}
            >
              {ativo && (
                <motion.span layoutId="aba-ativa" className="absolute inset-0 rounded-xl -z-10"
                  style={{ background: 'rgb(var(--c-destaque) / 0.16)', boxShadow: 'inset 0 0 0 1px rgb(var(--c-destaque) / 0.4)' }} />
              )}
              <a.Icone size={18} className={ativo ? 'text-destaque2' : ''} />
              {a.nome}
            </button>
          );
        })}
      </nav>

      <button onClick={() => usarApp.getState().setBusca(true)} className="botao w-full justify-between text-suave text-xs mt-2">
        Buscar função <kbd className="border border-borda rounded px-1.5 py-0.5 text-[10px]">Ctrl K</kbd>
      </button>
    </aside>
  );
}
