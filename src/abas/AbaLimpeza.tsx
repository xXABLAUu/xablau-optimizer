// Aba Limpeza: privacidade com semáforo + ajustes de limpeza/privacidade do catálogo.
import type { Ajuste } from '@/lib/tipos';
import { TituloSecao } from '@/componentes/basicos';
import { ListaAjustes } from '@/componentes/ListaAjustes';

export function AbaLimpeza({ ajustes }: { ajustes: Ajuste[] }) {
  return (
    <div className="space-y-5">
      <TituloSecao titulo="Limpeza e privacidade" />
      <ListaAjustes ajustes={ajustes} aba="limpeza" titulo="Limpeza e privacidade" />
    </div>
  );
}
