// Aba Desempenho: ajustes de desempenho/jogos + resumo da topologia (CCDs) e perfis por app.
import { useEffect, useState } from 'react';
import { Cpu, Layers } from 'lucide-react';
import type { Ajuste, Topologia } from '@/lib/tipos';
import { cmd } from '@/lib/comandos';
import { Cartao, TituloSecao } from '@/componentes/basicos';
import { ListaAjustes } from '@/componentes/ListaAjustes';
import { usarApp } from '@/store/app';

export function AbaDesempenho({ ajustes }: { ajustes: Ajuste[] }) {
  const modo = usarApp((e) => e.modo);
  const [topo, setTopo] = useState<Topologia | null>(null);
  useEffect(() => { cmd.topologia().then(setTopo).catch(() => {}); }, []);

  return (
    <div className="space-y-5">
      <TituloSecao titulo="Desempenho" />

      {modo === 'avancado' && topo && (
        <Cartao>
          <div className="flex items-center gap-2 mb-3 text-suave"><Layers size={16} /><span className="text-sm font-semibold">Topologia do processador (detectada)</span></div>
          <div className="flex flex-wrap gap-3">
            {topo.grupos.map((g) => (
              <div key={g.indice} className="cartao p-3 min-w-[150px]">
                <div className="flex items-center gap-2"><Cpu size={15} className="text-destaque2" /><span className="font-semibold text-sm">Grupo de cache {g.indice}</span></div>
                <p className="text-xs text-suave mt-1">{g.nucleos} núcleos · {g.logicos} lógicos</p>
                {g.l3_bytes > 0 && <p className="text-xs text-suave">L3: {(g.l3_bytes / 1048576).toFixed(0)} MB</p>}
              </div>
            ))}
          </div>
          <p className="text-xs text-suave mt-3">
            {topo.hibrido ? 'Processador híbrido (núcleos de desempenho e eficiência).' :
             topo.grupos.length > 1 ? 'Vários grupos de cache (CCDs) — dá para separar jogos em um grupo nos perfis por aplicativo.' :
             'Um único grupo de cache.'}
            {topo.smt ? ' SMT/Hyper-Threading ativo.' : ''}
          </p>
        </Cartao>
      )}

      <ListaAjustes ajustes={ajustes} aba="desempenho" titulo="Desempenho" />
    </div>
  );
}
