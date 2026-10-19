// Aba Sistema: serviços, Explorer e outros ajustes do catálogo (mais presentes no Modo Avançado).
import type { Ajuste } from '@/lib/tipos';
import { TituloSecao } from '@/componentes/basicos';
import { ListaAjustes } from '@/componentes/ListaAjustes';

export function AbaSistema({ ajustes }: { ajustes: Ajuste[] }) {
  return (
    <div className="space-y-5">
      <TituloSecao titulo="Sistema" />
      <ListaAjustes ajustes={ajustes} aba="sistema" titulo="Sistema" />
    </div>
  );
}
