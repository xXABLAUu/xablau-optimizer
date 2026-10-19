// "Saiba mais": explica o que faz, como funciona, o benefício e o risco de um ajuste.
import { AnimatePresence, motion } from 'framer-motion';
import { X } from 'lucide-react';
import type { SaibaMais as TSaibaMais } from '@/lib/tipos';
import { SeloRisco } from './basicos';
import type { Risco } from '@/lib/tipos';

export function SaibaMaisModal({
  titulo, risco, dados, aoFechar,
}: { titulo: string; risco: Risco; dados: TSaibaMais; aoFechar: () => void }) {
  const linhas: [string, string][] = [
    ['O que faz', dados.o_que],
    ['Como funciona', dados.como],
    ['Benefício', dados.beneficio],
    ['Risco', dados.risco],
  ];
  return (
    <AnimatePresence>
      <motion.div
        className="fixed inset-0 z-50 grid place-items-center bg-black/60 p-4"
        initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }}
        onClick={aoFechar}
      >
        <motion.div
          className="vidro max-w-lg w-full p-6 selecionavel"
          initial={{ scale: 0.95, y: 10 }} animate={{ scale: 1, y: 0 }} exit={{ scale: 0.95, opacity: 0 }}
          onClick={(e) => e.stopPropagation()}
        >
          <div className="flex items-start justify-between gap-4 mb-4">
            <div className="flex items-center gap-3">
              <h3 className="text-lg font-bold">{titulo}</h3>
              <SeloRisco risco={risco} />
            </div>
            <button className="botao !p-2" onClick={aoFechar} aria-label="Fechar"><X size={16} /></button>
          </div>
          <dl className="space-y-3">
            {linhas.map(([t, v]) => (
              <div key={t}>
                <dt className="text-xs font-bold uppercase tracking-wide text-destaque2">{t}</dt>
                <dd className="text-sm text-texto/90 mt-0.5">{v}</dd>
              </div>
            ))}
          </dl>
        </motion.div>
      </motion.div>
    </AnimatePresence>
  );
}
