// Busca global (Ctrl+K): encontra ajustes e abas por palavra e leva direto até eles.
import { AnimatePresence, motion } from 'framer-motion';
import { Search, CornerDownLeft } from 'lucide-react';
import { useEffect, useMemo, useRef, useState } from 'react';
import { usarApp } from '@/store/app';
import type { Ajuste } from '@/lib/tipos';

interface Alvo { tipo: 'aba' | 'ajuste'; id: string; titulo: string; sub: string; aba: string; }

export function BuscaGlobal({ ajustes }: { ajustes: Ajuste[] }) {
  const aberta = usarApp((e) => e.buscaAberta);
  const setBusca = usarApp((e) => e.setBusca);
  const irParaAba = usarApp((e) => e.irParaAba);
  const [texto, setTexto] = useState('');
  const [sel, setSel] = useState(0);
  const campo = useRef<HTMLInputElement>(null);

  const abas: Alvo[] = [
    { tipo: 'aba', id: 'inicio', titulo: 'Início', sub: 'Saúde, Turbo, otimização rápida', aba: 'inicio' },
    { tipo: 'aba', id: 'diagnostico', titulo: 'Diagnóstico', sub: 'BIOS, SSDs, culpados, FPS', aba: 'diagnostico' },
    { tipo: 'aba', id: 'desempenho', titulo: 'Desempenho', sub: 'Energia, núcleos, perfis', aba: 'desempenho' },
    { tipo: 'aba', id: 'limpeza', titulo: 'Limpeza', sub: 'Temporários, privacidade, apps', aba: 'limpeza' },
    { tipo: 'aba', id: 'sistema', titulo: 'Sistema', sub: 'Serviços, drivers, Windows Update', aba: 'sistema' },
    { tipo: 'aba', id: 'seguranca', titulo: 'Segurança', sub: 'Desfazer, histórico, restauração', aba: 'seguranca' },
    { tipo: 'aba', id: 'config', titulo: 'Configurações', sub: 'Temas, inicialização, sobre', aba: 'config' },
  ];

  const resultados = useMemo<Alvo[]>(() => {
    const t = texto.trim().toLowerCase();
    const dosAjustes: Alvo[] = ajustes.map((a) => ({
      tipo: 'ajuste', id: a.id, titulo: a.titulo, sub: a.descricao, aba: a.aba,
    }));
    const todos = [...abas, ...dosAjustes];
    if (!t) return abas;
    const pontua = (a: Alvo, origem?: Ajuste) => {
      const alvo = `${a.titulo} ${a.sub} ${(origem?.busca ?? []).join(' ')}`.toLowerCase();
      return alvo.includes(t) ? (a.titulo.toLowerCase().includes(t) ? 2 : 1) : 0;
    };
    return todos
      .map((a) => ({ a, p: pontua(a, ajustes.find((x) => x.id === a.id)) }))
      .filter((x) => x.p > 0)
      .sort((x, y) => y.p - x.p)
      .slice(0, 12)
      .map((x) => x.a);
  }, [texto, ajustes]);

  useEffect(() => { if (aberta) { setTexto(''); setSel(0); setTimeout(() => campo.current?.focus(), 30); } }, [aberta]);
  useEffect(() => { setSel(0); }, [texto]);

  if (!aberta) return null;

  const escolher = (a: Alvo) => { irParaAba(a.aba); setBusca(false); };

  return (
    <AnimatePresence>
      <motion.div
        className="fixed inset-0 z-50 flex items-start justify-center pt-28 bg-black/55 px-4"
        initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }}
        onClick={() => setBusca(false)}
      >
        <motion.div
          className="vidro w-full max-w-xl overflow-hidden"
          initial={{ scale: 0.97, y: -8 }} animate={{ scale: 1, y: 0 }} exit={{ scale: 0.97, opacity: 0 }}
          onClick={(e) => e.stopPropagation()}
        >
          <div className="flex items-center gap-3 px-4 py-3 border-b border-borda/70">
            <Search size={18} className="text-suave" />
            <input
              ref={campo}
              value={texto}
              onChange={(e) => setTexto(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === 'ArrowDown') { e.preventDefault(); setSel((s) => Math.min(s + 1, resultados.length - 1)); }
                else if (e.key === 'ArrowUp') { e.preventDefault(); setSel((s) => Math.max(s - 1, 0)); }
                else if (e.key === 'Enter' && resultados[sel]) escolher(resultados[sel]);
                else if (e.key === 'Escape') setBusca(false);
              }}
              placeholder="Buscar função ou aba… (ex.: CapCut, Copilot, drivers)"
              className="flex-1 bg-transparent outline-none text-texto placeholder:text-suave"
            />
            <kbd className="text-[10px] text-suave border border-borda rounded px-1.5 py-0.5">Esc</kbd>
          </div>
          <ul className="max-h-80 overflow-y-auto py-2">
            {resultados.length === 0 && <li className="px-4 py-6 text-center text-suave text-sm">Nada encontrado para “{texto}”.</li>}
            {resultados.map((a, i) => (
              <li key={`${a.tipo}-${a.id}`}>
                <button
                  onMouseEnter={() => setSel(i)}
                  onClick={() => escolher(a)}
                  className={`w-full text-left px-4 py-2.5 flex items-center justify-between gap-3 ${i === sel ? 'bg-destaque/15' : ''}`}
                >
                  <span>
                    <span className="font-semibold">{a.titulo}</span>
                    <span className="block text-xs text-suave">{a.sub}</span>
                  </span>
                  <span className="text-[10px] uppercase font-bold text-suave shrink-0">
                    {a.tipo === 'aba' ? 'Aba' : `Ajuste · ${nomeAba(a.aba)}`}
                  </span>
                </button>
              </li>
            ))}
          </ul>
          <div className="px-4 py-2 border-t border-borda/70 text-[11px] text-suave flex items-center gap-2">
            <CornerDownLeft size={12} /> Enter para abrir · ↑↓ para navegar
          </div>
        </motion.div>
      </motion.div>
    </AnimatePresence>
  );
}

function nomeAba(aba: string): string {
  const m: Record<string, string> = {
    inicio: 'Início', diagnostico: 'Diagnóstico', desempenho: 'Desempenho',
    limpeza: 'Limpeza', sistema: 'Sistema', seguranca: 'Segurança', config: 'Configurações',
  };
  return m[aba] ?? aba;
}
