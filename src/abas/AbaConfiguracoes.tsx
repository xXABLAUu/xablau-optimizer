// Aba Configurações: temas, fonte/tamanho/densidade, animações, automações, atualização e Sobre.
import { useEffect, useState } from 'react';
import { Palette, Type, Zap, RefreshCw, Info, PartyPopper, Loader2, Download } from 'lucide-react';
import { usarApp, TEMAS, type IdTema } from '@/store/app';
import { cmd } from '@/lib/comandos';
import type { InfoAtualizacao } from '@/lib/tipos';
import { Cartao, TituloSecao, Interruptor } from '@/componentes/basicos';

export function AbaConfiguracoes({ aoAbrirNovidades }: { aoAbrirNovidades: () => void }) {
  const prefs = usarApp((e) => e.prefs);
  const aplicarPrefs = usarApp((e) => e.aplicarPrefs);
  const config = usarApp((e) => e.config);
  const salvarConfig = usarApp((e) => e.salvarConfig);
  const sobre = usarApp((e) => e.sobre);
  const [atualizacao, setAtualizacao] = useState<InfoAtualizacao | null>(null);
  const [checando, setChecando] = useState(false);
  const [instalando, setInstalando] = useState(false);

  useEffect(() => { cmd.checarAtualizacao().then(setAtualizacao).catch(() => {}); }, []);

  const mudarAutomacao = async (chave: string, valor: boolean) => {
    if (!config) return;
    await salvarConfig({ ...config, automacao: { ...config.automacao, [chave]: valor } });
  };

  const checar = async () => { setChecando(true); try { setAtualizacao(await cmd.checarAtualizacao()); } finally { setChecando(false); } };
  const instalar = async () => { setInstalando(true); try { await cmd.instalarAtualizacao(); } catch (e) { alert(String(e)); } finally { setInstalando(false); } };

  return (
    <div className="space-y-5">
      <TituloSecao titulo="Configurações" descricao="Deixe o programa com a sua cara e escolha o que ele faz sozinho." />

      <Cartao>
        <div className="flex items-center gap-2 mb-3 text-suave"><Palette size={16} /><span className="text-sm font-semibold">Tema</span></div>
        <div className="grid grid-cols-2 sm:grid-cols-5 gap-2.5">
          {TEMAS.map((t) => (
            <button key={t.id} onClick={() => aplicarPrefs({ tema: t.id as IdTema })}
              className="rounded-xl p-2 border text-left transition-all"
              style={{ borderColor: prefs.tema === t.id ? 'rgb(var(--c-destaque))' : 'rgb(var(--c-borda))', boxShadow: prefs.tema === t.id ? '0 0 0 1px rgb(var(--c-destaque) / 0.5)' : 'none' }}>
              <div className="h-10 rounded-lg mb-1.5 flex overflow-hidden border border-borda/50">
                <span className="flex-1" style={{ background: t.amostra[0] }} />
                <span className="w-1/3" style={{ background: t.amostra[1] }} />
              </div>
              <span className="text-xs font-medium">{t.nome}</span>
            </button>
          ))}
        </div>
      </Cartao>

      <div className="grid sm:grid-cols-2 gap-5">
        <Cartao>
          <div className="flex items-center gap-2 mb-3 text-suave"><Type size={16} /><span className="text-sm font-semibold">Texto e aparência</span></div>
          <Controle rotulo="Fonte">
            <select className="botao !py-1.5" value={prefs.fonte} onChange={(e) => aplicarPrefs({ fonte: e.target.value as 'inter' | 'sistema' })}>
              <option value="inter">Inter (padrão)</option>
              <option value="sistema">Fonte do Windows</option>
            </select>
          </Controle>
          <Controle rotulo={`Tamanho do texto (${Math.round(prefs.escala * 100)}%)`}>
            <input type="range" min={0.85} max={1.3} step={0.05} value={prefs.escala} onChange={(e) => aplicarPrefs({ escala: Number(e.target.value) })} className="w-40" />
          </Controle>
          <Controle rotulo="Densidade">
            <select className="botao !py-1.5" value={prefs.densidade} onChange={(e) => aplicarPrefs({ densidade: Number(e.target.value) })}>
              <option value={0.85}>Compacta</option><option value={1}>Normal</option><option value={1.15}>Espaçosa</option>
            </select>
          </Controle>
          <Controle rotulo="Reduzir animações">
            <Interruptor ligado={prefs.animacoes === 'reduzidas'} aoMudar={(v) => aplicarPrefs({ animacoes: v ? 'reduzidas' : 'normais' })} />
          </Controle>
        </Cartao>

        <Cartao>
          <div className="flex items-center gap-2 mb-3 text-suave"><Zap size={16} /><span className="text-sm font-semibold">Automações</span></div>
          {config && (
            <>
              <Controle rotulo="Iniciar com o Windows (vigia em segundo plano)">
                <Interruptor ligado={config.automacao.iniciar_com_windows} aoMudar={(v) => mudarAutomacao('iniciar_com_windows', v)} />
              </Controle>
              <Controle rotulo="Vigia ativo (perfis por app, Modo Jogo)">
                <Interruptor ligado={config.automacao.vigia_ativo} aoMudar={(v) => mudarAutomacao('vigia_ativo', v)} />
              </Controle>
              <Controle rotulo="Modo Jogo automático">
                <Interruptor ligado={config.automacao.modo_jogo_auto} aoMudar={(v) => mudarAutomacao('modo_jogo_auto', v)} />
              </Controle>
              <Controle rotulo="Avisar quando o Windows desfizer um ajuste">
                <Interruptor ligado={config.automacao.avisar_reversoes} aoMudar={(v) => mudarAutomacao('avisar_reversoes', v)} />
              </Controle>
            </>
          )}
        </Cartao>
      </div>

      <Cartao>
        <div className="flex items-center gap-2 mb-3 text-suave"><RefreshCw size={16} /><span className="text-sm font-semibold">Atualização</span></div>
        {atualizacao?.disponivel ? (
          <div className="flex items-center justify-between gap-3">
            <div>
              <p className="font-semibold text-sm">Nova versão {atualizacao.versao_nova} disponível</p>
              <p className="text-xs text-suave">{atualizacao.motivo_bloqueio ?? 'Pronta para instalar.'}</p>
            </div>
            {atualizacao.pode_instalar ? (
              <button className="botao botao-primario" onClick={instalar} disabled={instalando}>
                {instalando ? <Loader2 size={15} className="animate-spin" /> : <Download size={15} />} Instalar
              </button>
            ) : <span className="text-xs text-suave">{atualizacao.portatil ? 'Baixe na página de Releases' : ''}</span>}
          </div>
        ) : (
          <div className="flex items-center justify-between gap-3">
            <p className="text-sm text-suave">Você está na versão {sobre?.versao ?? '1.0.0'} (a mais recente).</p>
            <button className="botao" onClick={checar} disabled={checando}>{checando ? <Loader2 size={15} className="animate-spin" /> : <RefreshCw size={15} />} Verificar</button>
          </div>
        )}
      </Cartao>

      <Cartao>
        <div className="flex items-center gap-2 mb-3 text-suave"><Info size={16} /><span className="text-sm font-semibold">Sobre</span></div>
        <p className="text-sm"><strong>{sobre?.nome}</strong> v{sobre?.versao}</p>
        <p className="text-sm text-suave">Criado por {sobre?.criador}</p>
        <p className="text-xs text-suave mt-2">
          {sobre?.admin ? 'Rodando como administrador.' : 'Não está como administrador.'}
          {sobre?.portatil ? ' Modo portátil.' : ''}
          {sobre?.modo_seguranca ? ' Modo de Segurança.' : ''}
        </p>
        <div className="flex gap-2 mt-4">
          <button className="botao" onClick={aoAbrirNovidades}><PartyPopper size={15} /> O que há de novo</button>
        </div>
        <p className="text-xs text-suave mt-4">
          Funções inspiradas em projetos de código aberto (WinUtil, Win11Debloat, Optimizer, Builtbybel) e no PresentMon (Intel). Licença MIT.
        </p>
      </Cartao>
    </div>
  );
}

function Controle({ rotulo, children }: { rotulo: string; children: React.ReactNode }) {
  return (
    <div className="flex items-center justify-between gap-4 py-2 border-b border-borda/30 last:border-0">
      <span className="text-sm">{rotulo}</span>
      {children}
    </div>
  );
}
