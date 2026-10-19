// Aba Segurança: desfazer tudo, desfazer por operação/item, histórico e recuperação de interrompidas.
import { useEffect, useState } from 'react';
import { Undo2, History, ShieldAlert, Loader2, RotateCcw, AlertTriangle } from 'lucide-react';
import type { Operacao, ResultadoDesfazer, Revertida } from '@/lib/tipos';
import { cmd } from '@/lib/comandos';
import { Cartao, TituloSecao, Vazio, LinhaEsqueleto } from '@/componentes/basicos';

export function AbaSeguranca() {
  const [ops, setOps] = useState<Operacao[] | null>(null);
  const [interrompidas, setInterrompidas] = useState<Operacao[]>([]);
  const [revertidas, setRevertidas] = useState<Revertida[]>([]);
  const [ocupado, setOcupado] = useState(false);
  const [aviso, setAviso] = useState<string | null>(null);
  const [expandida, setExpandida] = useState<string | null>(null);

  const carregar = () => {
    cmd.operacoes().then(setOps).catch(() => setOps([]));
    cmd.interrompidas().then(setInterrompidas).catch(() => {});
    cmd.revertidas().then(setRevertidas).catch(() => {});
  };
  useEffect(carregar, []);

  const desfazerTudo = async () => {
    if (!confirm('Isto vai desfazer TODAS as alterações que o Xablau já fez. Continuar?')) return;
    setOcupado(true); setAviso(null);
    try {
      const r = await cmd.desfazerTudo();
      const total = r.reduce((s, [, x]) => s + x.desfeitas, 0);
      setAviso(`${total} alteração(ões) desfeita(s).`); carregar();
    } catch (e) { setAviso(String(e)); } finally { setOcupado(false); }
  };

  const desfazerOp = async (op: Operacao) => {
    setOcupado(true); setAviso(null);
    try { const r: ResultadoDesfazer = await cmd.desfazerOperacao(op.id); setAviso(`${r.desfeitas} desfeita(s), ${r.falhas.length} falha(s).`); carregar(); }
    catch (e) { setAviso(String(e)); } finally { setOcupado(false); }
  };

  const desfazerItem = async (opId: string, itemId: number) => {
    setOcupado(true); setAviso(null);
    try { const r = await cmd.desfazerItem(opId, itemId); setAviso(r.desfeitas ? 'Item desfeito.' : (r.falhas[0] ?? 'Nada a desfazer.')); carregar(); }
    catch (e) { setAviso(String(e)); } finally { setOcupado(false); }
  };

  return (
    <div className="space-y-5">
      <TituloSecao titulo="Segurança" descricao="Tudo o que o programa muda fica registrado aqui e pode ser desfeito."
        acao={<button className="botao botao-perigo" onClick={desfazerTudo} disabled={ocupado}>
          {ocupado ? <Loader2 size={15} className="animate-spin" /> : <Undo2 size={15} />} Desfazer tudo
        </button>} />

      {aviso && <div className="vidro p-3 text-sm">{aviso}</div>}

      {interrompidas.length > 0 && (
        <Cartao className="!border-amarelo/40">
          <div className="flex items-center gap-2 text-amarelo mb-2"><ShieldAlert size={18} /><span className="font-semibold">Operação interrompida detectada</span></div>
          <p className="text-sm text-suave mb-3">O PC pode ter desligado no meio de uma otimização. Você pode desfazer para voltar ao estado anterior.</p>
          {interrompidas.map((op) => (
            <div key={op.id} className="flex items-center justify-between gap-3 text-sm py-1">
              <span>{op.titulo} <span className="text-suave">· {op.inicio}</span></span>
              <div className="flex gap-2">
                <button className="botao !py-1.5" onClick={() => desfazerOp(op)} disabled={ocupado}>Desfazer</button>
                <button className="botao !py-1.5" onClick={async () => { await cmd.encerrarInterrompida(op.id); carregar(); }} disabled={ocupado}>Manter</button>
              </div>
            </div>
          ))}
        </Cartao>
      )}

      {revertidas.length > 0 && (
        <Cartao className="!border-azul/40">
          <div className="flex items-center gap-2 text-azul mb-2"><AlertTriangle size={18} /><span className="font-semibold">O Windows desfez {revertidas.length} ajuste(s)</span></div>
          <p className="text-sm text-suave mb-2">Provavelmente após uma atualização. Veja quais e reaplique se quiser (nada é reaplicado sozinho):</p>
          <ul className="text-sm space-y-1">{revertidas.map((r, i) => <li key={i} className="text-suave">• {r.descricao}</li>)}</ul>
        </Cartao>
      )}

      <Cartao>
        <div className="flex items-center gap-2 mb-3 text-suave"><History size={16} /><span className="text-sm font-semibold">Histórico de alterações</span></div>
        {!ops ? <><LinhaEsqueleto /><LinhaEsqueleto largura="80%" /></> :
          ops.length === 0 ? <Vazio titulo="Nenhuma alteração ainda" descricao="Quando você otimizar algo, aparece aqui para poder desfazer." icone={<History size={30} />} /> : (
          <div className="space-y-2">
            {ops.map((op) => (
              <div key={op.id} className="cartao p-3">
                <div className="flex items-center justify-between gap-3">
                  <button className="text-left flex-1 min-w-0" onClick={() => setExpandida(expandida === op.id ? null : op.id)}>
                    <div className="font-semibold text-sm truncate">{op.titulo}</div>
                    <div className="text-xs text-suave">{op.inicio} · {op.itens.length} alteração(ões) {op.status ? `· ${op.status}` : '· interrompida'}</div>
                  </button>
                  <button className="botao !py-1.5 shrink-0" onClick={() => desfazerOp(op)} disabled={ocupado}><RotateCcw size={14} /> Desfazer</button>
                </div>
                {expandida === op.id && (
                  <ul className="mt-3 space-y-1.5 border-t border-borda/40 pt-3">
                    {op.itens.map((it) => (
                      <li key={it.id} className="flex items-center justify-between gap-3 text-xs">
                        <span className="text-suave truncate">{it.descricao}</span>
                        <div className="flex items-center gap-2 shrink-0">
                          <EstadoItem estado={it.estado} />
                          {it.reversivel && it.estado !== 'desfeita' && (
                            <button className="botao !py-1 !px-2 !text-xs" onClick={() => desfazerItem(op.id, it.id)} disabled={ocupado}>Desfazer</button>
                          )}
                        </div>
                      </li>
                    ))}
                  </ul>
                )}
              </div>
            ))}
          </div>
        )}
      </Cartao>

      <p className="text-xs text-suave">
        Em caso de problema grave, abra o kit <strong>“Xablau - Restaurar tudo”</strong> no Menu Iniciar — ele funciona até no Modo de Segurança.
      </p>
    </div>
  );
}

function EstadoItem({ estado }: { estado: string }) {
  const m: Record<string, [string, string]> = {
    feita: ['Aplicada', 'text-verde'], pendente: ['Pendente', 'text-suave'],
    falhou: ['Falhou', 'text-vermelho'], desfeita: ['Desfeita', 'text-suave'],
    falha_ao_desfazer: ['Erro ao desfazer', 'text-amarelo'],
  };
  const [txt, cor] = m[estado] ?? [estado, 'text-suave'];
  return <span className={`${cor} font-medium`}>{txt}</span>;
}
