//! Uso da GPU lido dos contadores de desempenho do Windows (chave "GPU Engine" do registro de
//! performance), sem depender de bibliotecas da NVIDIA/AMD. Usado pelo IdleSaver para decidir
//! se o PC está realmente ocioso (não queremos economizar durante uma exportação do CapCut).

use std::collections::HashMap;
use std::time::Instant;
use windows_sys::Win32::Foundation::ERROR_SUCCESS;
use windows_sys::Win32::System::Registry::{RegQueryValueExW, HKEY_PERFORMANCE_DATA, REG_NONE};

/// Soma dos tempos de execução dos engines de GPU (todos os processos), em unidades de 100 ns.
fn tempo_total_gpu() -> Option<u64> {
    // "Running Time" de todos os engines fica no objeto de performance 4098 ("GPU Engine").
    let consulta: Vec<u16> = "4098\0".encode_utf16().collect();
    let mut tamanho: u32 = 512 * 1024;
    for _ in 0..6 {
        let mut buf = vec![0u8; tamanho as usize];
        let mut tipo: u32 = REG_NONE;
        let mut tam = tamanho;
        let r = unsafe {
            RegQueryValueExW(HKEY_PERFORMANCE_DATA, consulta.as_ptr(), std::ptr::null_mut(), &mut tipo, buf.as_mut_ptr(), &mut tam)
        };
        if r == ERROR_SUCCESS {
            let total = somar_running_time(&buf[..tam as usize]);
            unsafe {
                // Fecha a chave de performance para não vazar recursos do sistema.
                windows_sys::Win32::System::Registry::RegCloseKey(HKEY_PERFORMANCE_DATA);
            }
            return total;
        }
        // ERROR_MORE_DATA (234): aumenta o buffer e tenta de novo.
        if r == 234u32 {
            tamanho = tamanho.saturating_mul(2);
            continue;
        }
        return None;
    }
    None
}

/// Percorre o bloco de dados de performance e soma o contador "Running Time" de cada engine.
fn somar_running_time(dados: &[u8]) -> Option<u64> {
    #[repr(C)]
    struct BlocoObjetos {
        _assinatura: [u16; 4],
        _versao: u32,
        _revisao: u32,
        _total_byte_length: u32,
        _header_length: u32,
        num_object_types: u32,
    }
    #[repr(C)]
    struct TipoObjeto {
        total_byte_length: u32,
        definition_length: u32,
        header_length: u32,
        object_name_title_index: u32,
        _resto: [u32; 2],
        _detail_level: u32,
        num_counters: u32,
        _default_counter: i32,
        num_instances: i32,
        _code_page: u32,
        _per_time: i64,
        _freq: i64,
    }
    #[repr(C)]
    struct DefContador {
        byte_length: u32,
        counter_name_title_index: u32,
        _rest1: [u32; 3],
        _counter_type: u32,
        counter_size: u32,
        counter_offset: u32,
    }
    #[repr(C)]
    struct DefInstancia {
        byte_length: u32,
        _parent: [u32; 2],
        _name_offset: u32,
        _name_length: u32,
    }

    if dados.len() < std::mem::size_of::<BlocoObjetos>() {
        return None;
    }
    let bloco = unsafe { &*(dados.as_ptr() as *const BlocoObjetos) };
    let mut desloc = bloco._header_length as usize;
    const RUNNING_TIME: u32 = 2; // índice do contador "Running Time" dentro de "GPU Engine"
    for _ in 0..bloco.num_object_types {
        if desloc + std::mem::size_of::<TipoObjeto>() > dados.len() {
            break;
        }
        let tipo = unsafe { &*(dados.as_ptr().add(desloc) as *const TipoObjeto) };
        if tipo.total_byte_length == 0 {
            break;
        }
        if tipo.object_name_title_index == 4098 {
            let mut off_contador = desloc + tipo.header_length as usize;
            let mut offset_running: Option<(u32, u32)> = None;
            for _ in 0..tipo.num_counters {
                if off_contador + std::mem::size_of::<DefContador>() > dados.len() {
                    break;
                }
                let c = unsafe { &*(dados.as_ptr().add(off_contador) as *const DefContador) };
                if c.byte_length == 0 {
                    break;
                }
                if c.counter_name_title_index == RUNNING_TIME {
                    offset_running = Some((c.counter_offset, c.counter_size));
                }
                off_contador += c.byte_length as usize;
            }
            let (coff, _csize) = offset_running?;
            let mut total: u64 = 0;
            let mut off_inst = desloc + tipo.definition_length as usize;
            let instancias = if tipo.num_instances < 0 { 1 } else { tipo.num_instances };
            for _ in 0..instancias {
                if off_inst + std::mem::size_of::<DefInstancia>() > dados.len() {
                    break;
                }
                let inst = unsafe { &*(dados.as_ptr().add(off_inst) as *const DefInstancia) };
                if inst.byte_length == 0 {
                    break;
                }
                let bloco_contador = off_inst + inst.byte_length as usize;
                let pos = bloco_contador + coff as usize;
                if pos + 8 <= dados.len() {
                    let v = u64::from_le_bytes(dados[pos..pos + 8].try_into().ok()?);
                    total = total.saturating_add(v);
                }
                // próxima instância: bloco do contador tem o próprio tamanho no início
                let tam_bloco = if bloco_contador + 4 <= dados.len() {
                    u32::from_le_bytes(dados[bloco_contador..bloco_contador + 4].try_into().ok()?) as usize
                } else {
                    break;
                };
                off_inst = bloco_contador + tam_bloco.max(4);
            }
            return Some(total);
        }
        desloc += tipo.total_byte_length as usize;
    }
    None
}

/// Mede o uso aproximado da GPU (0 a 100%) entre duas chamadas.
#[derive(Default)]
pub struct AmostradorGpu {
    anterior: Option<(u64, Instant)>,
    _marcador: HashMap<(), ()>,
}

impl AmostradorGpu {
    pub fn novo() -> Self {
        let mut a = Self::default();
        let _ = a.amostrar();
        a
    }

    /// Porcentagem de uso desde a última amostra. `None` = não foi possível medir.
    pub fn amostrar(&mut self) -> Option<f32> {
        let total = tempo_total_gpu()?;
        let agora = Instant::now();
        let uso = match self.anterior {
            Some((antes, quando)) => {
                let dt = agora.duration_since(quando).as_nanos() as f64 / 100.0; // em unidades de 100 ns
                if dt <= 0.0 {
                    0.0
                } else {
                    (total.saturating_sub(antes) as f64 / dt * 100.0) as f32
                }
            }
            None => 0.0,
        };
        self.anterior = Some((total, agora));
        Some(uso.clamp(0.0, 100.0))
    }
}
