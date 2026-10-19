// Lista de ajustes do catálogo com selo de risco, "Saiba mais" e seleção, mais o fluxo de
// simulação → aprovação → aplicação (com o resultado do ponto de restauração).
import { AnimatePresence, motion } from 'framer-motion';
import { Info, Check, Loader2, ShieldCheck } from 'lucide-react';
import { useMemo, useState } from 'react';
import type { Ajuste, ResultadoAplicacao, Simulacao } from '@/lib/tipos';
import { cmd } from '@/lib/comandos';
import { usarApp } from '@/store/app';
import { SeloRisco, Vazio } from './basicos';
import { SaibaMaisModal } from './SaibaMais';

export function ListaAjustes({ ajustes, aba, titulo }: {
  ajustes: Ajuste[]; aba: string; titulo: string;
}) {
  const modo = usarApp((e) => e.modo);
  const [sel, setSel] = useState<Set<string>>(new Set());
  const [saibaMais, setSaibaMais] = useState<Ajuste | null>(null);
  const [simulacao, setSimulacao] = useState<Simulacao | null>(null);
  const [resultado, setResultado] = useState<ResultadoAplicacao | null>(null);
  const [ocupado, setOcupado] = useState(false);
  const [erro, setErro] = useState<string | null>(null);
  const [recarregar, setRecarregar] = useState(0);

  const doGrupo = useMemo(
    () => ajustes.filter((a) => a.aba === aba && (modo === 'avancado' || a.modos.includes('basico'))),
    [ajustes, aba, modo, recarregar],
  );

  const alternar = (id: string) => {
    setSel((s) => { const n = new Set(s); n.has(id) ? n.delete(id) : n.add(id); return n; });
  };

  const abrirSimulacao = async () => {
    setErro(null); setOcupado(true);
    try {
      const sim = await cmd.simular([...sel]);
      setSimulacao(sim);
    } catch (e) { setErro(String(e)); } finally { setOcupado(false); }
  };

  const aplicar = async () => {
    setErro(null); setOcupado(true);
    try {
      const r = await cmd.aplicar(`${titulo} (${sel.size} ajuste${sel.size > 1 ? 's' : ''})`, [...sel]);
      setResultado(r); setSimulacao(null); setSel(new Set()); setRecarregar((n) => n + 1);
    } catch (e) { setErro(String(e)); setSimulacao(null); } finally { setOcupado(false); }
  };

  if (doGrupo.length === 0) {
    return <Vazio titulo="Nenhum ajuste nesta seção" descricao="No Modo Avançado aparecem mais opções." icone={<ShieldCheck size={32} />} />;
  }

  return (
    <div>
      <div className="space-y-2.5">
        {doGrupo.map((a) => {
          const marcado = sel.has(a.id);
          return (
            <motion.div key={a.id} layout className="vidro p-4 flex items-start gap-4">
              <button
                onClick={() => a.aplicavel && alternar(a.id)}
                disabled={!a.aplicavel}
                className="mt-0.5 h-5 w-5 rounded-md border grid place-items-center shrink-0 transition-colors disabled:opacity-40"
                style={{ borderColor: marcado ? 'rgb(var(--c-destaque))' : 'rgb(var(--c-borda))', background: marcado ? 'rgb(var(--c-destaque))' : 'transparent' }}
                aria-label={`Selecionar ${a.titulo}`}
              >
                {marcado && <Check size={14} className="text-white" />}
              </button>
              <div className="flex-1 min-w-0">
                <div className="flex items-center gap-2 flex-wrap">
                  <span className="font-semibold">{a.titulo}</span>
                  <SeloRisco risco={a.risco} />
                  {a.aplicado === true && <span className="selo selo-verde">Já aplicado</span>}
                  {!a.aplicavel && <span className="text-xs text-suave">(não se aplica a este PC)</span>}
                </div>
                <p className="text-sm text-suave mt-0.5">{a.descricao}</p>
              </div>
              <button className="botao !p-2 shrink-0" onClick={() => setSaibaMais(a)} aria-label="Saiba mais"><Info size={16} /></button>
            </motion.div>
          );
        })}
      </div>

      {erro && <p className="mt-4 text-sm text-vermelho">{erro}</p>}

      <AnimatePresence>
        {sel.size > 0 && (
          <motion.div
            initial={{ y: 60, opacity: 0 }} animate={{ y: 0, opacity: 1 }} exit={{ y: 60, opacity: 0 }}
            className="sticky bottom-4 mt-5 vidro p-3 flex items-center justify-between gap-3"
          >
            <span className="text-sm font-medium px-2">{sel.size} selecionado{sel.size > 1 ? 's' : ''}</span>
            <div className="flex gap-2">
              <button className="botao" onClick={() => setSel(new Set())}>Limpar</button>
              <button className="botao botao-primario" onClick={abrirSimulacao} disabled={ocupado}>
                {ocupado ? <Loader2 size={16} className="animate-spin" /> : <ShieldCheck size={16} />} Revisar e aplicar
              </button>
            </div>
          </motion.div>
        )}
      </AnimatePresence>

      {saibaMais && <SaibaMaisModal titulo={saibaMais.titulo} risco={saibaMais.risco} dados={saibaMais.saiba_mais} aoFechar={() => setSaibaMais(null)} />}

      <AnimatePresence>
        {simulacao && (
          <DialogoSimulacao simulacao={simulacao} ocupado={ocupado} aoAplicar={aplicar} aoCancelar={() => setSimulacao(null)} />
        )}
      </AnimatePresence>

      <AnimatePresence>
        {resultado && <DialogoResultado resultado={resultado} aoFechar={() => setResultado(null)} />}
      </AnimatePresence>
    </div>
  );
}

function DialogoSimulacao({ simulacao, ocupado, aoAplicar, aoCancelar }: {
  simulacao: Simulacao; ocupado: boolean; aoAplicar: () => void; aoCancelar: () => void;
}) {
  return (
    <motion.div className="fixed inset-0 z-50 grid place-items-center bg-black/60 p-4"
      initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }} onClick={aoCancelar}>
      <motion.div className="vidro w-full max-w-2xl max-h-[85vh] flex flex-col"
        initial={{ scale: 0.96, y: 12 }} animate={{ scale: 1, y: 0 }} onClick={(e) => e.stopPropagation()}>
        <div className="p-6 pb-3 border-b border-borda/70">
          <h2 className="text-xl font-black">Revisar alterações</h2>
          <p className="text-sm text-suave mt-1">
            Nada foi aplicado ainda. Antes de aplicar, o programa cria um <strong>ponto de restauração</strong> e
            guarda uma cópia de segurança. Você poderá desfazer tudo depois.
          </p>
        </div>
        <div className="overflow-y-auto p-6 space-y-3 selecionavel">
          {simulacao.mudancas.length === 0 && (
            <p className="text-sm text-suave">Os ajustes escolhidos já estão como desejado — nada a mudar.</p>
          )}
          {simulacao.mudancas.map((m, i) => (
            <div key={i} className="text-sm">
              <div className="font-semibold">{m.titulo}</div>
              <div className="text-suave text-xs mt-0.5">{m.descricao}</div>
            </div>
          ))}
          {simulacao.sem_efeito.length > 0 && (
            <p className="text-xs text-suave pt-2">Já aplicados (sem efeito): {simulacao.sem_efeito.join(', ')}.</p>
          )}
          {simulacao.erros.length > 0 && (
            <div className="text-xs text-amarelo pt-2">
              {simulacao.erros.map(([t, e], i) => <div key={i}>{t}: {e}</div>)}
            </div>
          )}
        </div>
        <div className="p-4 border-t border-borda/70 flex justify-end gap-2">
          <button className="botao" onClick={aoCancelar}>Cancelar</button>
          <button className="botao botao-primario" onClick={aoAplicar} disabled={ocupado || simulacao.mudancas.length === 0}>
            {ocupado ? <Loader2 size={16} className="animate-spin" /> : <Check size={16} />} Aplicar com segurança
          </button>
        </div>
      </motion.div>
    </motion.div>
  );
}

function DialogoResultado({ resultado, aoFechar }: { resultado: ResultadoAplicacao; aoFechar: () => void }) {
  const ok = resultado.status === 'ok';
  return (
    <motion.div className="fixed inset-0 z-50 grid place-items-center bg-black/60 p-4"
      initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }} onClick={aoFechar}>
      <motion.div className="vidro w-full max-w-md p-6" initial={{ scale: 0.96 }} animate={{ scale: 1 }} onClick={(e) => e.stopPropagation()}>
        <div className="flex items-center gap-3 mb-3">
          <div className="grid place-items-center h-10 w-10 rounded-xl" style={{ background: ok ? 'rgb(var(--c-verde) / 0.2)' : 'rgb(var(--c-amarelo) / 0.2)' }}>
            {ok ? <Check className="text-verde" /> : <Info className="text-amarelo" />}
          </div>
          <h2 className="text-lg font-bold">{ok ? 'Tudo certo!' : 'Concluído com avisos'}</h2>
        </div>
        <p className="text-sm text-texto/90">{resultado.aplicadas} alteração(ões) aplicada(s) e registrada(s).</p>
        <p className="text-xs text-suave mt-1">{resultado.ponto_restauracao.mensagem}</p>
        {resultado.falhas.length > 0 && (
          <ul className="text-xs text-amarelo mt-3 space-y-1">{resultado.falhas.map((f, i) => <li key={i}>• {f}</li>)}</ul>
        )}
        <p className="text-xs text-suave mt-4">Para desfazer, vá em <strong>Segurança</strong>.</p>
        <div className="flex justify-end mt-5"><button className="botao botao-primario" onClick={aoFechar}>Fechar</button></div>
      </motion.div>
    </motion.div>
  );
}
