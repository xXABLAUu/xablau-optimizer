// Elemento memorável do app: o medidor animado de saúde do sistema (anel + número que sobe).
import { motion, useMotionValue, useTransform, animate } from 'framer-motion';
import { useEffect, useState } from 'react';

function cor(nota: number): string {
  if (nota >= 80) return 'var(--c-verde)';
  if (nota >= 55) return 'var(--c-destaque2)';
  if (nota >= 35) return 'var(--c-amarelo)';
  return 'var(--c-vermelho)';
}

export function MedidorSaude({ nota, rotulo, tamanho = 220 }: { nota: number; rotulo?: string; tamanho?: number }) {
  const progresso = useMotionValue(0);
  const [exibido, setExibido] = useState(0);
  const raio = tamanho / 2 - 16;
  const circ = 2 * Math.PI * raio;
  const offset = useTransform(progresso, (v) => circ - (circ * v) / 100);

  useEffect(() => {
    const controle = animate(progresso, nota, { duration: 1.4, ease: [0.16, 1, 0.3, 1] });
    const naoAssinar = progresso.on('change', (v) => setExibido(Math.round(v)));
    return () => { controle.stop(); naoAssinar(); };
  }, [nota, progresso]);

  const corAtual = cor(exibido);

  return (
    <div className="relative grid place-items-center" style={{ width: tamanho, height: tamanho }}>
      <svg width={tamanho} height={tamanho} className="-rotate-90">
        <defs>
          <linearGradient id="grad-saude" x1="0%" y1="0%" x2="100%" y2="100%">
            <stop offset="0%" stopColor="rgb(var(--c-destaque))" />
            <stop offset="100%" stopColor="rgb(var(--c-destaque2))" />
          </linearGradient>
        </defs>
        <circle cx={tamanho / 2} cy={tamanho / 2} r={raio} fill="none" stroke="rgb(var(--c-borda) / 0.5)" strokeWidth={14} />
        <motion.circle
          cx={tamanho / 2} cy={tamanho / 2} r={raio} fill="none"
          stroke={`rgb(${corAtual})`} strokeWidth={14} strokeLinecap="round"
          strokeDasharray={circ} style={{ strokeDashoffset: offset }}
        />
      </svg>
      <div className="absolute inset-0 grid place-items-center text-center">
        <div>
          <div className="text-5xl font-black tabular-nums" style={{ color: `rgb(${corAtual})` }}>{exibido}</div>
          <div className="text-xs font-semibold text-suave mt-1">{rotulo ?? 'de saúde'}</div>
        </div>
      </div>
    </div>
  );
}
