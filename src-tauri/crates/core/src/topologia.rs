//! Topologia real do processador: CPU Sets, núcleos, SMT, grupos de cache L3 (CCDs/CCXs do Ryzen)
//! e núcleos de desempenho/eficiência (Intel híbrido). Nada é suposto: tudo vem do Windows.
//! Exemplo: o Ryzen 9 5900XT aparece com 16 núcleos em 2 grupos de L3 (2 CCDs de 8).

use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use windows_sys::Win32::System::SystemInformation::{
    CpuSetInformation, GetLogicalProcessorInformationEx, GetSystemCpuSetInformation, RelationCache,
    SYSTEM_CPU_SET_INFORMATION, SYSTEM_LOGICAL_PROCESSOR_INFORMATION_EX,
};

/// Um processador lógico visto pelo Windows.
#[derive(Debug, Clone, Serialize)]
pub struct CpuSet {
    pub id: u32,
    pub grupo: u16,
    pub logico: u8,
    pub nucleo: u8,
    pub cache: u8,
    pub numa: u8,
    pub eficiencia: u8,
}

/// Processadores que compartilham o mesmo cache L3 (um CCD/CCX no Ryzen).
#[derive(Debug, Clone, Serialize)]
pub struct GrupoCache {
    pub indice: u8,
    pub cpu_sets: Vec<u32>,
    pub logicos: usize,
    pub nucleos: usize,
    pub l3_bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct Topologia {
    pub logicos: usize,
    pub nucleos: usize,
    pub smt: bool,
    pub hibrido: bool,
    pub grupos: Vec<GrupoCache>,
    pub cpu_sets: Vec<CpuSet>,
}

impl Topologia {
    /// IDs dos CPU Sets de um grupo de cache.
    pub fn cpu_sets_do_grupo(&self, indice: u8) -> Vec<u32> {
        self.grupos.iter().find(|g| g.indice == indice).map(|g| g.cpu_sets.clone()).unwrap_or_default()
    }

    /// IDs dos CPU Sets dos núcleos de desempenho (maior classe de eficiência).
    pub fn cpu_sets_desempenho(&self) -> Vec<u32> {
        let maior = self.cpu_sets.iter().map(|c| c.eficiencia).max().unwrap_or(0);
        self.cpu_sets.iter().filter(|c| c.eficiencia == maior).map(|c| c.id).collect()
    }

    /// Máscara de afinidade (grupo 0) para um grupo de cache.
    pub fn mascara_do_grupo(&self, indice: u8) -> usize {
        self.cpu_sets
            .iter()
            .filter(|c| c.cache == indice && c.grupo == 0 && u32::from(c.logico) < usize::BITS)
            .fold(0usize, |m, c| m | (1usize << c.logico))
    }
}

fn ler_cpu_sets() -> Vec<CpuSet> {
    let mut v = Vec::new();
    unsafe {
        let mut tam: u32 = 0;
        GetSystemCpuSetInformation(std::ptr::null_mut(), 0, &mut tam, std::ptr::null_mut(), 0);
        if tam == 0 {
            return v;
        }
        let mut buf: Vec<u64> = vec![0; (tam as usize).div_ceil(8)];
        if GetSystemCpuSetInformation(buf.as_mut_ptr() as *mut SYSTEM_CPU_SET_INFORMATION, tam, &mut tam, std::ptr::null_mut(), 0) == 0 {
            return v;
        }
        let base = buf.as_ptr() as *const u8;
        let mut desloc = 0usize;
        while desloc + std::mem::size_of::<SYSTEM_CPU_SET_INFORMATION>() <= tam as usize {
            let info = &*(base.add(desloc) as *const SYSTEM_CPU_SET_INFORMATION);
            if info.Size == 0 {
                break;
            }
            if info.Type == CpuSetInformation {
                let c = info.Anonymous.CpuSet;
                v.push(CpuSet {
                    id: c.Id,
                    grupo: c.Group,
                    logico: c.LogicalProcessorIndex,
                    nucleo: c.CoreIndex,
                    cache: c.LastLevelCacheIndex,
                    numa: c.NumaNodeIndex,
                    eficiencia: c.EfficiencyClass,
                });
            }
            desloc += info.Size as usize;
        }
    }
    v
}

/// Caches L3: (grupo de processadores, máscara, tamanho em bytes).
fn caches_l3() -> Vec<(u16, usize, u64)> {
    let mut v = Vec::new();
    unsafe {
        let mut tam: u32 = 0;
        GetLogicalProcessorInformationEx(RelationCache, std::ptr::null_mut(), &mut tam);
        if tam == 0 {
            return v;
        }
        let mut buf: Vec<u64> = vec![0; (tam as usize).div_ceil(8)];
        if GetLogicalProcessorInformationEx(RelationCache, buf.as_mut_ptr() as *mut SYSTEM_LOGICAL_PROCESSOR_INFORMATION_EX, &mut tam) == 0 {
            return v;
        }
        let base = buf.as_ptr() as *const u8;
        let mut desloc = 0usize;
        while desloc + 8 <= tam as usize {
            let info = &*(base.add(desloc) as *const SYSTEM_LOGICAL_PROCESSOR_INFORMATION_EX);
            if info.Size == 0 {
                break;
            }
            if info.Relationship == RelationCache {
                let c = &info.Anonymous.Cache;
                if c.Level == 3 {
                    let g = c.Anonymous.GroupMask;
                    v.push((g.Group, g.Mask, u64::from(c.CacheSize)));
                }
            }
            desloc += info.Size as usize;
        }
    }
    v
}

/// Lê a topologia do processador.
pub fn ler() -> Topologia {
    let cpu_sets = ler_cpu_sets();
    let caches = caches_l3();
    let nucleos: BTreeSet<(u16, u8)> = cpu_sets.iter().map(|c| (c.grupo, c.nucleo)).collect();
    let classes: BTreeSet<u8> = cpu_sets.iter().map(|c| c.eficiencia).collect();
    let mut por_cache: BTreeMap<u8, Vec<&CpuSet>> = BTreeMap::new();
    for c in &cpu_sets {
        por_cache.entry(c.cache).or_default().push(c);
    }
    let grupos = por_cache
        .into_iter()
        .map(|(indice, membros)| {
            let nucleos_grupo: BTreeSet<(u16, u8)> = membros.iter().map(|c| (c.grupo, c.nucleo)).collect();
            let l3_bytes = membros
                .first()
                .and_then(|m| {
                    caches.iter().find(|(g, mascara, _)| {
                        *g == m.grupo && u32::from(m.logico) < usize::BITS && (mascara >> m.logico) & 1 == 1
                    })
                })
                .map(|(_, _, tam)| *tam)
                .unwrap_or(0);
            GrupoCache { indice, cpu_sets: membros.iter().map(|c| c.id).collect(), logicos: membros.len(), nucleos: nucleos_grupo.len(), l3_bytes }
        })
        .collect();
    Topologia { logicos: cpu_sets.len(), nucleos: nucleos.len(), smt: cpu_sets.len() > nucleos.len(), hibrido: classes.len() > 1, grupos, cpu_sets }
}
