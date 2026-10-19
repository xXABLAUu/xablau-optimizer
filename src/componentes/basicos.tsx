// Componentes visuais reutilizáveis do Xablau Optimizer.
import { motion } from 'framer-motion';
import type { ReactNode } from 'react';
import type { Risco } from '@/lib/tipos';
import { ShieldCheck, AlertTriangle, Flame } from 'lucide-react';

export function Cartao({ children, className = '', vidro = true }: { children: ReactNode; className?: string; vidro?: boolean }) {
  return <div className={`${vidro ? 'vidro' : 'cartao'} p-4 ${className}`}>{children}</div>;
}

export function TituloSecao({ titulo, descricao, acao }: { titulo: string; descricao?: string; acao?: ReactNode }) {
  return (
    <div className="flex items-start justify-between gap-4 mb-4">
      <div>
        <h2 className="text-xl font-bold tracking-tight text-texto">{titulo}</h2>
        {descricao && <p className="text-sm text-suave mt-0.5 max-w-prose">{descricao}</p>}
      </div>
      {acao}
    </div>
  );
}

const SELO: Record<Risco, { classe: string; rotulo: string; Icone: typeof ShieldCheck }> = {
  verde: { classe: 'selo-verde', rotulo: 'Seguro', Icone: ShieldCheck },
  amarelo: { classe: 'selo-amarelo', rotulo: 'Atenção', Icone: AlertTriangle },
  vermelho: { classe: 'selo-vermelho', rotulo: 'Avançado', Icone: Flame },
};

export function SeloRisco({ risco }: { risco: Risco }) {
  const { classe, rotulo, Icone } = SELO[risco];
  return (
    <span className={`selo ${classe}`}>
      <Icone size={13} aria-hidden /> {rotulo}
    </span>
  );
}

export function Interruptor({
  ligado, aoMudar, rotulo, desabilitado,
}: { ligado: boolean; aoMudar: (v: boolean) => void; rotulo?: string; desabilitado?: boolean }) {
  return (
    <button
      role="switch"
      aria-checked={ligado}
      aria-label={rotulo}
      disabled={desabilitado}
      onClick={() => aoMudar(!ligado)}
      className="relative inline-flex h-6 w-11 shrink-0 items-center rounded-full transition-colors disabled:opacity-50"
      style={{ background: ligado ? 'rgb(var(--c-destaque))' : 'rgb(var(--c-borda))' }}
    >
      <motion.span
        layout
        transition={{ type: 'spring', stiffness: 500, damping: 34 }}
        className="inline-block h-4 w-4 rounded-full bg-white shadow"
        style={{ marginLeft: ligado ? 24 : 4 }}
      />
    </button>
  );
}

export function Vazio({ titulo, descricao, icone }: { titulo: string; descricao?: string; icone?: ReactNode }) {
  return (
    <div className="flex flex-col items-center justify-center text-center py-14 px-6 text-suave">
      {icone && <div className="mb-3 opacity-60">{icone}</div>}
      <p className="font-semibold text-texto">{titulo}</p>
      {descricao && <p className="text-sm mt-1 max-w-sm">{descricao}</p>}
    </div>
  );
}

export function LinhaEsqueleto({ largura = '100%' }: { largura?: string }) {
  return <div className="esqueleto h-4 my-2" style={{ width: largura }} />;
}

export function Rotulo({ children }: { children: ReactNode }) {
  return <span className="text-xs font-semibold text-suave">{children}</span>;
}

export function BarraProgresso({ valor, cor = 'destaque' }: { valor: number; cor?: string }) {
  return (
    <div className="h-2 w-full rounded-full bg-borda/50 overflow-hidden">
      <motion.div
        className="h-full rounded-full"
        style={{ background: `rgb(var(--c-${cor}))` }}
        initial={{ width: 0 }}
        animate={{ width: `${Math.max(0, Math.min(100, valor))}%` }}
        transition={{ type: 'spring', stiffness: 120, damping: 20 }}
      />
    </div>
  );
}
