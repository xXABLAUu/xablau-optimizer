// Aba Diagnóstico: só analisa, não altera nada. Hardware, saúde dos SSDs, culpados e ficha do PC.
import { useEffect, useState } from 'react';
import { HardDrive, Activity, FileText, Copy, Check, AlertTriangle, RefreshCw } from 'lucide-react';
import type { Culpado, Hardware, SaudeSsd } from '@/lib/tipos';
import { cmd } from '@/lib/comandos';
import { Cartao, TituloSecao, LinhaEsqueleto, Vazio } from '@/componentes/basicos';

export function AbaDiagnostico() {
  const [hw, setHw] = useState<Hardware | null>(null);
  const [ssds, setSsds] = useState<SaudeSsd[] | null>(null);
  const [culpados, setCulpados] = useState<Culpado[] | null>(null);
  const [copiado, setCopiado] = useState(false);

  const carregar = () => {
    cmd.hardware().then(setHw).catch(() => {});
    cmd.saudeSsds().then(setSsds).catch(() => setSsds([]));
    cmd.culpados(12).then(setCulpados).catch(() => setCulpados([]));
  };
  useEffect(carregar, []);

  const copiarFicha = async () => {
    try { const t = await cmd.fichaPc(false); await navigator.clipboard.writeText(t); setCopiado(true); setTimeout(() => setCopiado(false), 2000); } catch { /* vazio */ }
  };

  return (
    <div className="space-y-5">
      <TituloSecao titulo="Diagnóstico" descricao="Esta aba só analisa o seu PC. Nada é alterado aqui."
        acao={<button className="botao" onClick={carregar}><RefreshCw size={15} /> Atualizar</button>} />

      <div className="grid lg:grid-cols-2 gap-5">
        <Cartao>
          <div className="flex items-center gap-2 mb-3 text-suave"><FileText size={16} /><span className="text-sm font-semibold">Ficha do PC</span></div>
          {!hw ? <><LinhaEsqueleto /><LinhaEsqueleto largura="80%" /><LinhaEsqueleto largura="60%" /></> : (
            <dl className="text-sm space-y-1.5">
              <Linha rotulo="Processador" valor={hw.cpu} />
              <Linha rotulo="Núcleos" valor={`${hw.nucleos_fisicos} físicos / ${hw.nucleos_logicos} lógicos · ${hw.ccds} CCD(s)`} />
              <Linha rotulo="Placa de vídeo" valor={hw.gpu.join(', ') || '—'} />
              <Linha rotulo="Placa-mãe" valor={hw.placa_mae} />
              <Linha rotulo="BIOS" valor={hw.bios} />
              <Linha rotulo="Memória" valor={`${hw.ram_total_gb.toFixed(0)} GB @ ${hw.ram_velocidade_mhz} MHz`} />
              <Linha rotulo="RAM (XMP/DOCP)" valor={hw.ram_modo} />
              <Linha rotulo="Sistema" valor={hw.windows} />
            </dl>
          )}
          <button className="botao mt-4" onClick={copiarFicha}>
            {copiado ? <><Check size={15} /> Copiado!</> : <><Copy size={15} /> Copiar ficha (sem dados pessoais)</>}
          </button>
        </Cartao>

        <Cartao>
          <div className="flex items-center gap-2 mb-3 text-suave"><HardDrive size={16} /><span className="text-sm font-semibold">Saúde dos SSDs</span></div>
          {!ssds ? <><LinhaEsqueleto /><LinhaEsqueleto largura="70%" /></> :
            ssds.length === 0 ? <Vazio titulo="Nenhum disco lido" /> : (
            <div className="space-y-3">
              {ssds.map((d, i) => (
                <div key={i} className="cartao p-3">
                  <div className="flex items-center justify-between gap-2">
                    <span className="font-semibold text-sm truncate">{d.modelo}</span>
                    <span className="text-xs text-suave shrink-0">{d.tipo} · {d.tamanho_gb.toFixed(0)} GB</span>
                  </div>
                  {d.confiavel ? (
                    <div className="flex flex-wrap gap-x-4 gap-y-1 text-xs text-suave mt-2">
                      {d.desgaste_pct != null && <span>Desgaste: <b className="text-texto">{d.desgaste_pct}%</b></span>}
                      {d.temperatura_c != null && <span>Temp.: <b className="text-texto">{d.temperatura_c}°C</b></span>}
                      {d.horas_ligado != null && <span>Horas ligado: <b className="text-texto">{d.horas_ligado}</b></span>}
                      {d.erros != null && <span>Erros: <b className="text-texto">{d.erros}</b></span>}
                    </div>
                  ) : (
                    <div className="flex items-start gap-2 text-xs text-amarelo mt-2">
                      <AlertTriangle size={14} className="shrink-0 mt-0.5" />
                      <span>{d.aviso}</span>
                    </div>
                  )}
                </div>
              ))}
            </div>
          )}
        </Cartao>
      </div>

      <Cartao>
        <div className="flex items-center gap-2 mb-3 text-suave"><Activity size={16} /><span className="text-sm font-semibold">Quem está consumindo mais</span></div>
        {!culpados ? <><LinhaEsqueleto /><LinhaEsqueleto largura="90%" /></> :
          culpados.length === 0 ? <Vazio titulo="Sem dados" /> : (
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead><tr className="text-left text-xs text-suave border-b border-borda/60">
                <th className="py-2 font-semibold">Programa</th>
                <th className="py-2 font-semibold text-right">Memória</th>
                <th className="py-2 font-semibold text-right">Disco (E/S)</th>
                <th className="py-2 font-semibold text-center">Eficiência</th>
              </tr></thead>
              <tbody>
                {culpados.map((c) => (
                  <tr key={c.pid} className="border-b border-borda/30">
                    <td className="py-2 truncate max-w-[280px]">{c.nome}{c.protegido && <span className="text-xs text-suave"> (sistema)</span>}</td>
                    <td className="py-2 text-right tabular-nums">{c.memoria_mb} MB</td>
                    <td className="py-2 text-right tabular-nums">{c.es_mb} MB</td>
                    <td className="py-2 text-center">{c.eficiencia === true ? '🌱' : c.eficiencia === false ? '—' : '?'}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </Cartao>
    </div>
  );
}

function Linha({ rotulo, valor }: { rotulo: string; valor: string }) {
  return (
    <div className="flex justify-between gap-4">
      <dt className="text-suave shrink-0">{rotulo}</dt>
      <dd className="text-right font-medium truncate selecionavel">{valor}</dd>
    </div>
  );
}
