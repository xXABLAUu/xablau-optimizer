import { AnimatePresence, motion } from 'framer-motion';
import { useEffect, useState } from 'react';
import { usarApp } from '@/store/app';
import { cmd } from '@/lib/comandos';
import type { Ajuste } from '@/lib/tipos';
import { BarraLateral } from '@/componentes/BarraLateral';
import { Cabecalho } from '@/componentes/Cabecalho';
import { BuscaGlobal } from '@/componentes/BuscaGlobal';
import { Assistente } from '@/componentes/Assistente';
import { OQueHaDeNovo } from '@/componentes/OQueHaDeNovo';
import { AbaInicio } from '@/abas/AbaInicio';
import { AbaDiagnostico } from '@/abas/AbaDiagnostico';
import { AbaDesempenho } from '@/abas/AbaDesempenho';
import { AbaLimpeza } from '@/abas/AbaLimpeza';
import { AbaSistema } from '@/abas/AbaSistema';
import { AbaSeguranca } from '@/abas/AbaSeguranca';
import { AbaConfiguracoes } from '@/abas/AbaConfiguracoes';
import changelog from '@/lib/changelog.json';
import type { VersaoChangelog } from '@/lib/tipos';

export function App() {
  const carregado = usarApp((e) => e.carregado);
  const iniciar = usarApp((e) => e.iniciar);
  const abaAtual = usarApp((e) => e.abaAtual);
  const setBusca = usarApp((e) => e.setBusca);
  const config = usarApp((e) => e.config);
  const salvarConfig = usarApp((e) => e.salvarConfig);
  const setVigia = usarApp((e) => e.setVigia);

  const [ajustes, setAjustes] = useState<Ajuste[]>([]);
  const [mostrarAssistente, setMostrarAssistente] = useState(false);
  const [mostrarNovidades, setMostrarNovidades] = useState(false);

  useEffect(() => { iniciar(); }, [iniciar]);

  useEffect(() => {
    if (!carregado || !config) return;
    cmd.listarAjustes().then(setAjustes).catch(() => {});
    if (!config.assistente_concluido) setMostrarAssistente(true);
    else {
      const versaoAtual = (changelog as { versoes: VersaoChangelog[] }).versoes[0]?.versao;
      if (versaoAtual && config.ultima_versao_vista !== versaoAtual) setMostrarNovidades(true);
    }
  }, [carregado, config]);

  // Atalho Ctrl+K para a busca global.
  useEffect(() => {
    const aoTeclar = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'k') { e.preventDefault(); setBusca(true); }
    };
    window.addEventListener('keydown', aoTeclar);
    return () => window.removeEventListener('keydown', aoTeclar);
  }, [setBusca]);

  // Atualiza o status do vigia periodicamente.
  useEffect(() => {
    if (!carregado) return;
    let vivo = true;
    const tick = async () => { try { const v = await cmd.statusVigia(); if (vivo) setVigia(v); } catch { /* vazio */ } };
    tick();
    const t = setInterval(tick, 3000);
    return () => { vivo = false; clearInterval(t); };
  }, [carregado, setVigia]);

  const fecharNovidades = async () => {
    setMostrarNovidades(false);
    const versaoAtual = (changelog as { versoes: VersaoChangelog[] }).versoes[0]?.versao;
    if (config && versaoAtual) await salvarConfig({ ...config, ultima_versao_vista: versaoAtual });
  };

  if (!carregado) return <TelaCarregando />;

  if (mostrarAssistente) return <Assistente aoConcluir={() => { setMostrarAssistente(false); cmd.listarAjustes().then(setAjustes).catch(() => {}); }} />;

  return (
    <div className="flex h-full w-full overflow-hidden">
      <BarraLateral />
      <div className="flex-1 flex flex-col min-w-0">
        <Cabecalho aoAbrirNovidades={() => setMostrarNovidades(true)} />
        <main className="flex-1 overflow-y-auto px-6 py-5">
          <AnimatePresence mode="wait">
            <motion.div
              key={abaAtual}
              initial={{ opacity: 0, y: 8 }} animate={{ opacity: 1, y: 0 }} exit={{ opacity: 0, y: -4 }}
              transition={{ duration: 0.18 }}
            >
              {abaAtual === 'inicio' && <AbaInicio ajustes={ajustes} />}
              {abaAtual === 'diagnostico' && <AbaDiagnostico />}
              {abaAtual === 'desempenho' && <AbaDesempenho ajustes={ajustes} />}
              {abaAtual === 'limpeza' && <AbaLimpeza ajustes={ajustes} />}
              {abaAtual === 'sistema' && <AbaSistema ajustes={ajustes} />}
              {abaAtual === 'seguranca' && <AbaSeguranca />}
              {abaAtual === 'config' && <AbaConfiguracoes aoAbrirNovidades={() => setMostrarNovidades(true)} />}
            </motion.div>
          </AnimatePresence>
        </main>
      </div>
      <BuscaGlobal ajustes={ajustes} />
      <AnimatePresence>{mostrarNovidades && <OQueHaDeNovo aoFechar={fecharNovidades} />}</AnimatePresence>
    </div>
  );
}

function TelaCarregando() {
  return (
    <div className="grid h-full place-items-center bg-fundo">
      <div className="text-center">
        <motion.div
          className="mx-auto mb-4 grid h-16 w-16 place-items-center rounded-2xl"
          style={{ background: 'linear-gradient(135deg, rgb(var(--c-destaque)), rgb(var(--c-destaque2)))' }}
          animate={{ scale: [1, 1.08, 1] }} transition={{ duration: 1.4, repeat: Infinity }}
        >
          <span className="text-3xl font-black text-white">X</span>
        </motion.div>
        <p className="text-suave text-sm">Carregando o Xablau Optimizer…</p>
      </div>
    </div>
  );
}
