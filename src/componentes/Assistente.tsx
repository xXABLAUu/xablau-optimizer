// Assistente de primeiro uso (animado): escolhe modo (Básico/Avançado) e uso principal.
import { AnimatePresence, motion } from 'framer-motion';
import { Gamepad2, Clapperboard, Monitor, Sparkles, ChevronRight } from 'lucide-react';
import { useState } from 'react';
import { usarApp } from '@/store/app';
import type { Modo, UsoPrincipal } from '@/lib/tipos';

export function Assistente({ aoConcluir }: { aoConcluir: () => void }) {
  const config = usarApp((e) => e.config);
  const salvarConfig = usarApp((e) => e.salvarConfig);
  const [passo, setPasso] = useState(0);
  const [modo, setModo] = useState<Modo>('basico');
  const [uso, setUso] = useState<UsoPrincipal>('geral');

  const concluir = async () => {
    if (config) {
      await salvarConfig({ ...config, modo, uso_principal: uso, assistente_concluido: true });
    }
    aoConcluir();
  };

  return (
    <div className="fixed inset-0 z-[60] grid place-items-center bg-fundo p-6">
      <motion.div
        className="vidro w-full max-w-2xl p-8 overflow-hidden"
        initial={{ opacity: 0, y: 16 }} animate={{ opacity: 1, y: 0 }}
      >
        <div className="flex items-center gap-3 mb-6">
          <LogoX />
          <div>
            <h1 className="text-2xl font-black">Bem-vindo ao Xablau Optimizer</h1>
            <p className="text-suave text-sm">Vamos deixar tudo do seu jeito em dois passos rápidos.</p>
          </div>
        </div>

        <AnimatePresence mode="wait">
          {passo === 0 ? (
            <motion.div key="modo" initial={{ opacity: 0, x: 24 }} animate={{ opacity: 1, x: 0 }} exit={{ opacity: 0, x: -24 }}>
              <h2 className="font-bold mb-3">Como você prefere usar o programa?</h2>
              <div className="grid sm:grid-cols-2 gap-3">
                <CartaoEscolha
                  ativo={modo === 'basico'} aoClicar={() => setModo('basico')}
                  icone={<Sparkles />} titulo="Modo Básico"
                  texto="Poucas opções, linguagem simples e otimização automática só com ajustes seguros. Ideal se você não entende de computador."
                />
                <CartaoEscolha
                  ativo={modo === 'avancado'} aoClicar={() => setModo('avancado')}
                  icone={<Monitor />} titulo="Modo Avançado"
                  texto="Todas as abas e funções, perfis por aplicativo e regras automáticas. Para quem gosta de controlar cada detalhe."
                />
              </div>
              <p className="text-xs text-suave mt-3">Pode trocar quando quiser, numa chave no topo do programa.</p>
            </motion.div>
          ) : (
            <motion.div key="uso" initial={{ opacity: 0, x: 24 }} animate={{ opacity: 1, x: 0 }} exit={{ opacity: 0, x: -24 }}>
              <h2 className="font-bold mb-3">Para que você mais usa este PC?</h2>
              <div className="grid sm:grid-cols-3 gap-3">
                <CartaoEscolha ativo={uso === 'jogos'} aoClicar={() => setUso('jogos')} icone={<Gamepad2 />} titulo="Jogos" texto="Foco em FPS e baixa latência." />
                <CartaoEscolha ativo={uso === 'edicao'} aoClicar={() => setUso('edicao')} icone={<Clapperboard />} titulo="Trabalho / Edição" texto="CapCut, Chrome e apps pesados." />
                <CartaoEscolha ativo={uso === 'geral'} aoClicar={() => setUso('geral')} icone={<Monitor />} titulo="Uso geral" texto="Um pouco de tudo." />
              </div>
            </motion.div>
          )}
        </AnimatePresence>

        <div className="flex items-center justify-between mt-8">
          <div className="flex gap-1.5">
            {[0, 1].map((p) => (
              <span key={p} className="h-1.5 rounded-full transition-all" style={{ width: p === passo ? 28 : 10, background: p === passo ? 'rgb(var(--c-destaque))' : 'rgb(var(--c-borda))' }} />
            ))}
          </div>
          <div className="flex gap-2">
            {passo > 0 && <button className="botao" onClick={() => setPasso(0)}>Voltar</button>}
            {passo === 0 ? (
              <button className="botao botao-primario" onClick={() => setPasso(1)}>Continuar <ChevronRight size={16} /></button>
            ) : (
              <button className="botao botao-primario" onClick={concluir}>Começar a usar</button>
            )}
          </div>
        </div>
      </motion.div>
    </div>
  );
}

function CartaoEscolha({ ativo, aoClicar, icone, titulo, texto }: {
  ativo: boolean; aoClicar: () => void; icone: React.ReactNode; titulo: string; texto: string;
}) {
  return (
    <button
      onClick={aoClicar}
      className="text-left rounded-xl2 p-4 border transition-all"
      style={{
        borderColor: ativo ? 'rgb(var(--c-destaque))' : 'rgb(var(--c-borda))',
        background: ativo ? 'rgb(var(--c-destaque) / 0.1)' : 'rgb(var(--c-elevado))',
        boxShadow: ativo ? '0 0 0 1px rgb(var(--c-destaque) / 0.5)' : 'none',
      }}
    >
      <div className="mb-2 text-destaque2">{icone}</div>
      <div className="font-bold">{titulo}</div>
      <p className="text-xs text-suave mt-1">{texto}</p>
    </button>
  );
}

function LogoX() {
  return (
    <div className="grid place-items-center h-12 w-12 rounded-xl shrink-0"
      style={{ background: 'linear-gradient(135deg, rgb(var(--c-destaque)), rgb(var(--c-destaque2)))' }}>
      <span className="text-2xl font-black text-white">X</span>
    </div>
  );
}
