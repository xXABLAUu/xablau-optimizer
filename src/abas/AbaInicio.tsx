// Aba Início: medidor de saúde animado, uso em tempo real, otimização rápida e Turbo.
import { useEffect, useMemo, useState } from 'react';
import { Rocket, Zap, Cpu, MemoryStick, Loader2, ShieldCheck } from 'lucide-react';
import type { Ajuste, Hardware, ResultadoAplicacao, UsoTempoReal } from '@/lib/tipos';
import { cmd } from '@/lib/comandos';
import { usarApp } from '@/store/app';
import { Cartao, BarraProgresso } from '@/componentes/basicos';
import { MedidorSaude } from '@/componentes/MedidorSaude';

export function AbaInicio({ ajustes }: { ajustes: Ajuste[] }) {
  const [hw, setHw] = useState<Hardware | null>(null);
  const [uso, setUso] = useState<UsoTempoReal | null>(null);
  const turbo = usarApp((e) => e.turbo);
  const setTurbo = usarApp((e) => e.setTurbo);
  const [aplicando, setAplicando] = useState(false);
  const [resultado, setResultado] = useState<ResultadoAplicacao | null>(null);
  const [erro, setErro] = useState<string | null>(null);

  useEffect(() => { cmd.hardware().then(setHw).catch(() => {}); }, []);
  useEffect(() => {
    let vivo = true;
    const tick = async () => { try { const u = await cmd.usoTempoReal(); if (vivo) setUso(u); } catch { /* vazio */ } };
    tick(); const t = setInterval(tick, 1500);
    return () => { vivo = false; clearInterval(t); };
  }, []);

  // Saúde: pontos por ajustes seguros já aplicados + folga de memória + sem bateria baixa.
  const segurosDoBasico = useMemo(() => ajustes.filter((a) => a.risco === 'verde' && a.aplicavel && a.modos.includes('basico')), [ajustes]);
  const nota = useMemo(() => {
    if (ajustes.length === 0) return 72;
    const aplicados = segurosDoBasico.filter((a) => a.aplicado === true).length;
    const base = segurosDoBasico.length > 0 ? 55 + Math.round((aplicados / segurosDoBasico.length) * 40) : 75;
    const penalidadeRam = uso && uso.ram_pct > 85 ? -8 : 0;
    return Math.max(10, Math.min(100, base + penalidadeRam));
  }, [ajustes, segurosDoBasico, uso]);

  const pendentes = segurosDoBasico.filter((a) => a.aplicado !== true);

  const otimizarRapido = async () => {
    setErro(null); setAplicando(true);
    try {
      const ids = pendentes.map((a) => a.id);
      if (ids.length === 0) { setErro('Tudo que é seguro já está aplicado. 🎉'); return; }
      const r = await cmd.aplicar('Otimização rápida (perfil Seguro)', ids);
      setResultado(r);
    } catch (e) { setErro(String(e)); } finally { setAplicando(false); }
  };

  const alternarTurbo = async () => {
    const novo = !turbo; setTurbo(novo);
    try { await cmd.definirTurbo(novo); } catch { setTurbo(!novo); }
  };

  return (
    <div className="grid lg:grid-cols-[320px_1fr] gap-5">
      <Cartao className="flex flex-col items-center justify-center text-center">
        <MedidorSaude nota={nota} rotulo="de saúde" />
        <p className="text-sm text-suave mt-3 max-w-[260px]">
          {nota >= 80 ? 'Seu PC está bem ajustado!' : nota >= 55 ? 'Dá para melhorar com alguns ajustes seguros.' : 'Vale aplicar a otimização rápida.'}
        </p>
      </Cartao>

      <div className="space-y-5">
        <Cartao>
          <h3 className="font-bold mb-1">Otimização rápida</h3>
          <p className="text-sm text-suave mb-4">
            Aplica de uma vez os ajustes <strong>seguros</strong> que ainda faltam
            {pendentes.length > 0 ? ` (${pendentes.length} disponível${pendentes.length > 1 ? 'eis' : ''})` : ''}.
            Cria ponto de restauração antes e tudo pode ser desfeito.
          </p>
          <div className="flex flex-wrap gap-2">
            <button className="botao botao-primario" onClick={otimizarRapido} disabled={aplicando}>
              {aplicando ? <Loader2 size={16} className="animate-spin" /> : <ShieldCheck size={16} />} Otimizar com um clique
            </button>
            <button className={`botao ${turbo ? 'botao-primario' : ''}`} onClick={alternarTurbo}>
              {turbo ? <Rocket size={16} /> : <Zap size={16} />} {turbo ? 'Turbo ligado' : 'Modo Turbo'}
            </button>
          </div>
          {erro && <p className="text-sm text-amarelo mt-3">{erro}</p>}
        </Cartao>

        <div className="grid sm:grid-cols-2 gap-5">
          <Cartao>
            <div className="flex items-center gap-2 mb-2 text-suave"><Cpu size={16} /> <span className="text-sm font-semibold">Processador</span></div>
            <div className="text-3xl font-black tabular-nums">{uso ? Math.round(uso.cpu_pct) : '—'}<span className="text-lg text-suave">%</span></div>
            <div className="mt-2"><BarraProgresso valor={uso?.cpu_pct ?? 0} cor="destaque2" /></div>
            <p className="text-xs text-suave mt-2 truncate">{hw?.cpu ?? 'Lendo…'}</p>
          </Cartao>
          <Cartao>
            <div className="flex items-center gap-2 mb-2 text-suave"><MemoryStick size={16} /> <span className="text-sm font-semibold">Memória</span></div>
            <div className="text-3xl font-black tabular-nums">{uso ? uso.ram_pct : '—'}<span className="text-lg text-suave">%</span></div>
            <div className="mt-2"><BarraProgresso valor={uso?.ram_pct ?? 0} cor="destaque" /></div>
            <p className="text-xs text-suave mt-2">{uso ? `${uso.ram_usada_gb.toFixed(1)} de ${uso.ram_total_gb.toFixed(0)} GB` : 'Lendo…'}</p>
          </Cartao>
        </div>
      </div>

      {resultado && <ResultadoRapido resultado={resultado} aoFechar={() => setResultado(null)} />}
    </div>
  );
}

function ResultadoRapido({ resultado, aoFechar }: { resultado: ResultadoAplicacao; aoFechar: () => void }) {
  return (
    <div className="fixed inset-0 z-50 grid place-items-center bg-black/60 p-4" onClick={aoFechar}>
      <div className="vidro max-w-md w-full p-6" onClick={(e) => e.stopPropagation()}>
        <h3 className="text-lg font-bold mb-2">Otimização concluída</h3>
        <p className="text-sm text-texto/90">{resultado.aplicadas} ajuste(s) aplicado(s).</p>
        <p className="text-xs text-suave mt-1">{resultado.ponto_restauracao.mensagem}</p>
        <div className="flex justify-end mt-5"><button className="botao botao-primario" onClick={aoFechar}>Fechar</button></div>
      </div>
    </div>
  );
}
