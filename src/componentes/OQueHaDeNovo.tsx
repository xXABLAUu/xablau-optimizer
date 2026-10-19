// Tela "O que há de novo": mostra as novidades da versão atual (lida do changelog embutido).
import { motion } from 'framer-motion';
import { X, Sparkles } from 'lucide-react';
import changelog from '@/lib/changelog.json';
import type { VersaoChangelog } from '@/lib/tipos';

export function OQueHaDeNovo({ aoFechar }: { aoFechar: () => void }) {
  const versoes = (changelog as { versoes: VersaoChangelog[] }).versoes;
  const atual = versoes[0];
  if (!atual) return null;
  return (
    <motion.div
      className="fixed inset-0 z-50 grid place-items-center bg-black/60 p-4"
      initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }} onClick={aoFechar}
    >
      <motion.div
        className="vidro w-full max-w-2xl max-h-[85vh] flex flex-col"
        initial={{ scale: 0.96, y: 12 }} animate={{ scale: 1, y: 0 }}
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-start justify-between gap-4 p-6 pb-4 border-b border-borda/70">
          <div className="flex items-center gap-3">
            <div className="grid place-items-center h-10 w-10 rounded-xl" style={{ background: 'linear-gradient(135deg, rgb(var(--c-destaque)), rgb(var(--c-destaque2)))' }}>
              <Sparkles size={20} className="text-white" />
            </div>
            <div>
              <h2 className="text-xl font-black">O que há de novo</h2>
              <p className="text-sm text-suave">Versão {atual.versao} — {atual.titulo}</p>
            </div>
          </div>
          <button className="botao !p-2" onClick={aoFechar} aria-label="Fechar"><X size={16} /></button>
        </div>
        <div className="overflow-y-auto p-6 space-y-5 selecionavel">
          {atual.destaques.length > 0 && (
            <div className="rounded-xl p-4" style={{ background: 'rgb(var(--c-destaque) / 0.1)' }}>
              <ul className="space-y-1.5">
                {atual.destaques.map((d, i) => (
                  <li key={i} className="text-sm flex gap-2"><span className="text-destaque">◆</span>{d}</li>
                ))}
              </ul>
            </div>
          )}
          {atual.novidades.map((grupo) => (
            <div key={grupo.area}>
              <h3 className="font-bold text-destaque2 mb-1.5">{grupo.area}</h3>
              <ul className="space-y-1 text-sm text-texto/90">
                {grupo.itens.map((item, i) => (<li key={i} className="flex gap-2"><span className="text-suave">•</span>{item}</li>))}
              </ul>
            </div>
          ))}
          {atual.correcoes.length > 0 && (
            <div>
              <h3 className="font-bold text-verde mb-1.5">Correções</h3>
              <ul className="space-y-1 text-sm text-texto/90">
                {atual.correcoes.map((c, i) => (<li key={i} className="flex gap-2"><span className="text-suave">•</span>{c}</li>))}
              </ul>
            </div>
          )}
        </div>
        <div className="p-4 border-t border-borda/70 flex justify-end">
          <button className="botao botao-primario" onClick={aoFechar}>Entendi</button>
        </div>
      </motion.div>
    </motion.div>
  );
}
