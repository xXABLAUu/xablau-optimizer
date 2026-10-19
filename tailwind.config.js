/** Cores vêm de variáveis CSS definidas por tema (src/estilos/temas.css). */
const cor = (nome) => `rgb(var(--c-${nome}) / <alpha-value>)`;

/** @type {import('tailwindcss').Config} */
export default {
  content: ['./index.html', './src/**/*.{ts,tsx}'],
  theme: {
    extend: {
      colors: {
        fundo: cor('fundo'),
        superficie: cor('superficie'),
        elevado: cor('elevado'),
        borda: cor('borda'),
        texto: cor('texto'),
        suave: cor('suave'),
        destaque: cor('destaque'),
        destaque2: cor('destaque2'),
        sobreDestaque: cor('sobre-destaque'),
        verde: cor('verde'),
        amarelo: cor('amarelo'),
        vermelho: cor('vermelho'),
        azul: cor('azul'),
      },
      fontFamily: {
        sans: ['var(--fonte-app)', 'Inter Variable', 'Segoe UI Variable Text', 'Segoe UI', 'system-ui', 'sans-serif'],
        mono: ['Cascadia Mono', 'Consolas', 'ui-monospace', 'monospace'],
      },
      borderRadius: {
        xl2: '1.25rem',
      },
      boxShadow: {
        brilho: '0 0 0 1px rgb(var(--c-destaque) / 0.35), 0 8px 32px -8px rgb(var(--c-destaque) / 0.45)',
        cartao: '0 1px 0 0 rgb(255 255 255 / 0.04) inset, 0 12px 40px -18px rgb(0 0 0 / 0.55)',
      },
      keyframes: {
        brilhar: { '0%,100%': { opacity: '0.55' }, '50%': { opacity: '1' } },
        esqueleto: { '0%': { backgroundPosition: '-200% 0' }, '100%': { backgroundPosition: '200% 0' } },
      },
      animation: {
        brilhar: 'brilhar 2.4s ease-in-out infinite',
        esqueleto: 'esqueleto 1.6s linear infinite',
      },
    },
  },
  plugins: [],
};
