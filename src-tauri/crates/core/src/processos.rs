//! Processos: lista (sem abrir nenhum processo), prioridade, Modo de Eficiência (EcoQoS),
//! prioridade de E/S e de memória, CPU Sets e afinidade.
//!
//! Anti-cheat: só APIs padrão do Windows, cada operação abre o processo com a permissão MÍNIMA
//! necessária, e nunca se pede leitura de memória (PROCESS_VM_READ). Se o anti-cheat negar,
//! o erro volta como `AcessoNegado` e quem chamou mostra um aviso amigável.

use crate::config::{Prioridade, PrioridadeEs};
use serde::Serialize;
use std::ffi::c_void;
use windows_sys::Win32::Foundation::{CloseHandle, GetLastError, ERROR_ACCESS_DENIED, ERROR_INVALID_PARAMETER, HANDLE};
use windows_sys::Win32::System::Threading::{
    GetPriorityClass, GetProcessAffinityMask, GetProcessInformation, OpenProcess, ProcessMemoryPriority, ProcessPowerThrottling,
    QueryFullProcessImageNameW, SetPriorityClass, SetProcessAffinityMask, SetProcessDefaultCpuSets, SetProcessInformation,
    TerminateProcess, ABOVE_NORMAL_PRIORITY_CLASS, BELOW_NORMAL_PRIORITY_CLASS, HIGH_PRIORITY_CLASS, IDLE_PRIORITY_CLASS,
    MEMORY_PRIORITY_INFORMATION, NORMAL_PRIORITY_CLASS, PROCESS_ACCESS_RIGHTS, PROCESS_POWER_THROTTLING_EXECUTION_SPEED,
    PROCESS_POWER_THROTTLING_IGNORE_TIMER_RESOLUTION, PROCESS_POWER_THROTTLING_STATE, PROCESS_QUERY_LIMITED_INFORMATION,
    PROCESS_SET_INFORMATION, PROCESS_SET_LIMITED_INFORMATION, PROCESS_TERMINATE, REALTIME_PRIORITY_CLASS,
};

#[link(name = "ntdll", kind = "raw-dylib")]
extern "system" {
    fn NtQuerySystemInformation(classe: u32, info: *mut c_void, tamanho: u32, retornado: *mut u32) -> i32;
    fn NtSetInformationProcess(processo: HANDLE, classe: u32, info: *const c_void, tamanho: u32) -> i32;
}

const SYSTEM_PROCESS_INFORMATION: u32 = 5;
const PROCESS_IO_PRIORITY: u32 = 33;
const STATUS_INFO_LENGTH_MISMATCH: i32 = 0xC000_0004_u32 as i32;
const STATUS_ACCESS_DENIED: i32 = 0xC000_0022_u32 as i32;

/// Erro ao mexer num processo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErroProcesso {
    /// O Windows ou um anti-cheat negou o acesso.
    AcessoNegado,
    /// O processo já fechou.
    NaoEncontrado,
    Falhou(String),
}

impl std::fmt::Display for ErroProcesso {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErroProcesso::AcessoNegado => write!(f, "acesso negado pelo Windows ou pelo anti-cheat"),
            ErroProcesso::NaoEncontrado => write!(f, "o processo já foi fechado"),
            ErroProcesso::Falhou(m) => write!(f, "{m}"),
        }
    }
}

impl std::error::Error for ErroProcesso {}

fn ultimo_erro() -> ErroProcesso {
    match unsafe { GetLastError() } {
        ERROR_ACCESS_DENIED => ErroProcesso::AcessoNegado,
        ERROR_INVALID_PARAMETER => ErroProcesso::NaoEncontrado,
        c => ErroProcesso::Falhou(format!("erro do Windows {c}")),
    }
}

/// Alça de processo que fecha sozinha.
struct Alca(HANDLE);

impl Drop for Alca {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.0);
        }
    }
}

fn abrir(pid: u32, acesso: PROCESS_ACCESS_RIGHTS) -> Result<Alca, ErroProcesso> {
    if pid == 0 || pid == 4 {
        return Err(ErroProcesso::AcessoNegado);
    }
    let h = unsafe { OpenProcess(acesso, 0, pid) };
    if h.is_null() {
        Err(ultimo_erro())
    } else {
        Ok(Alca(h))
    }
}

/// Amostra de um processo, obtida de uma vez para todos (NtQuerySystemInformation),
/// sem abrir nenhum processo — seguro até com jogos protegidos por anti-cheat.
#[derive(Debug, Clone, Serialize)]
pub struct AmostraProcesso {
    pub pid: u32,
    pub ppid: u32,
    pub nome: String,
    pub threads: u32,
    pub handles: u32,
    pub sessao: u32,
    /// Tempo de CPU acumulado (usuário + núcleo), em unidades de 100 ns.
    pub tempo_cpu: u64,
    /// Memória privada (commit), em bytes.
    pub memoria_privada: u64,
    /// Memória física em uso (working set), em bytes.
    pub memoria_em_uso: u64,
    pub leitura_bytes: u64,
    pub escrita_bytes: u64,
    pub outros_bytes: u64,
    /// Momento de criação (FILETIME), usado para não confundir PIDs reaproveitados.
    pub criado: u64,
    pub prioridade_base: i32,
}

#[repr(C)]
struct UnicodeString {
    tamanho: u16,
    maximo: u16,
    buffer: *mut u16,
}

/// Layout público de SYSTEM_PROCESS_INFORMATION (x64), igual ao usado por ferramentas como o System Informer.
#[repr(C)]
struct InfoProcessoSistema {
    proximo: u32,
    threads: u32,
    privado_ws: i64,
    falhas_duras: u32,
    threads_pico: u32,
    ciclos: u64,
    criado: i64,
    tempo_usuario: i64,
    tempo_nucleo: i64,
    nome: UnicodeString,
    prioridade_base: i32,
    pid: usize,
    ppid: usize,
    handles: u32,
    sessao: u32,
    chave: usize,
    virtual_pico: usize,
    virtual_tam: usize,
    falhas_pagina: u32,
    ws_pico: usize,
    ws: usize,
    quota_paginado_pico: usize,
    quota_paginado: usize,
    quota_nao_paginado_pico: usize,
    quota_nao_paginado: usize,
    pagefile: usize,
    pagefile_pico: usize,
    paginas_privadas: usize,
    op_leitura: i64,
    op_escrita: i64,
    op_outras: i64,
    bytes_leitura: i64,
    bytes_escrita: i64,
    bytes_outros: i64,
}

/// Fotografia de todos os processos agora.
pub fn instantaneo() -> Vec<AmostraProcesso> {
    let mut tamanho: usize = 512 * 1024;
    for _ in 0..8 {
        let mut buf: Vec<u64> = vec![0; tamanho.div_ceil(8)];
        let bytes = buf.len() * 8;
        let mut retornado: u32 = 0;
        let st = unsafe {
            NtQuerySystemInformation(SYSTEM_PROCESS_INFORMATION, buf.as_mut_ptr() as *mut c_void, bytes as u32, &mut retornado)
        };
        if st == STATUS_INFO_LENGTH_MISMATCH {
            tamanho = (retornado as usize).max(tamanho) + 256 * 1024;
            continue;
        }
        if st < 0 {
            return Vec::new();
        }
        return unsafe { interpretar(buf.as_ptr() as *const u8, bytes) };
    }
    Vec::new()
}

unsafe fn interpretar(base: *const u8, limite: usize) -> Vec<AmostraProcesso> {
    let mut v = Vec::with_capacity(256);
    let mut deslocamento = 0usize;
    loop {
        if deslocamento + std::mem::size_of::<InfoProcessoSistema>() > limite {
            break;
        }
        let p = &*(base.add(deslocamento) as *const InfoProcessoSistema);
        let nome = if p.nome.buffer.is_null() || p.nome.tamanho == 0 {
            if p.pid == 0 { "Ocioso do sistema".to_string() } else { "System".to_string() }
        } else {
            String::from_utf16_lossy(std::slice::from_raw_parts(p.nome.buffer, usize::from(p.nome.tamanho / 2)))
        };
        v.push(AmostraProcesso {
            pid: p.pid as u32,
            ppid: p.ppid as u32,
            nome,
            threads: p.threads,
            handles: p.handles,
            sessao: p.sessao,
            tempo_cpu: (p.tempo_usuario.max(0) + p.tempo_nucleo.max(0)) as u64,
            memoria_privada: p.pagefile as u64,
            memoria_em_uso: p.ws as u64,
            leitura_bytes: p.bytes_leitura.max(0) as u64,
            escrita_bytes: p.bytes_escrita.max(0) as u64,
            outros_bytes: p.bytes_outros.max(0) as u64,
            criado: p.criado.max(0) as u64,
            prioridade_base: p.prioridade_base,
        });
        if p.proximo == 0 {
            break;
        }
        deslocamento += p.proximo as usize;
    }
    v
}

/// PIDs dos processos com este nome de executável.
pub fn pids_por_nome(todos: &[AmostraProcesso], exe: &str) -> Vec<u32> {
    todos.iter().filter(|p| p.nome.eq_ignore_ascii_case(exe)).map(|p| p.pid).collect()
}

/// Processos filhos (e netos...) de `raiz`, ignorando PIDs reaproveitados.
pub fn descendentes(raiz: u32, todos: &[AmostraProcesso]) -> Vec<u32> {
    let criado_raiz = todos.iter().find(|p| p.pid == raiz).map(|p| p.criado).unwrap_or(0);
    let mut resultado: Vec<u32> = Vec::new();
    let mut fila = vec![(raiz, criado_raiz)];
    while let Some((pai, criado_pai)) = fila.pop() {
        for p in todos {
            if p.ppid == pai && p.pid != pai && p.pid != raiz && p.criado >= criado_pai && !resultado.contains(&p.pid) {
                resultado.push(p.pid);
                fila.push((p.pid, p.criado));
                if resultado.len() >= 1024 {
                    return resultado;
                }
            }
        }
    }
    resultado
}

/// Caminho completo do executável de um processo.
pub fn caminho_executavel(pid: u32) -> Option<String> {
    let h = abrir(pid, PROCESS_QUERY_LIMITED_INFORMATION).ok()?;
    let mut buf = vec![0u16; 2048];
    let mut tam = buf.len() as u32;
    let ok = unsafe { QueryFullProcessImageNameW(h.0, 0, buf.as_mut_ptr(), &mut tam) };
    (ok != 0).then(|| String::from_utf16_lossy(&buf[..tam as usize]))
}

fn classe(p: Prioridade) -> u32 {
    match p {
        Prioridade::Ociosa => IDLE_PRIORITY_CLASS,
        Prioridade::AbaixoDoNormal => BELOW_NORMAL_PRIORITY_CLASS,
        Prioridade::Normal => NORMAL_PRIORITY_CLASS,
        Prioridade::AcimaDoNormal => ABOVE_NORMAL_PRIORITY_CLASS,
        Prioridade::Alta => HIGH_PRIORITY_CLASS,
    }
}

/// Define a prioridade (nunca Tempo Real: a enum nem tem essa opção).
pub fn definir_prioridade(pid: u32, p: Prioridade) -> Result<(), ErroProcesso> {
    let h = abrir(pid, PROCESS_SET_INFORMATION)?;
    if unsafe { SetPriorityClass(h.0, classe(p)) } == 0 {
        Err(ultimo_erro())
    } else {
        Ok(())
    }
}

/// Prioridade atual.
pub fn ler_prioridade(pid: u32) -> Result<Prioridade, ErroProcesso> {
    let h = abrir(pid, PROCESS_QUERY_LIMITED_INFORMATION)?;
    let c = unsafe { GetPriorityClass(h.0) };
    Ok(match c {
        0 => return Err(ultimo_erro()),
        IDLE_PRIORITY_CLASS => Prioridade::Ociosa,
        BELOW_NORMAL_PRIORITY_CLASS => Prioridade::AbaixoDoNormal,
        ABOVE_NORMAL_PRIORITY_CLASS => Prioridade::AcimaDoNormal,
        HIGH_PRIORITY_CLASS | REALTIME_PRIORITY_CLASS => Prioridade::Alta,
        _ => Prioridade::Normal,
    })
}

fn estado_eficiencia(controle: u32, estado: u32) -> PROCESS_POWER_THROTTLING_STATE {
    PROCESS_POWER_THROTTLING_STATE { Version: 1, ControlMask: controle, StateMask: estado }
}

/// O Modo de Eficiência (EcoQoS, a "plantinha" do Gerenciador de Tarefas) está ligado?
/// `None` = o Windows não informa (Windows 10 antigo) ou sem acesso.
pub fn eficiencia_ativa(pid: u32) -> Option<bool> {
    let h = abrir(pid, PROCESS_QUERY_LIMITED_INFORMATION).ok()?;
    let mut st = estado_eficiencia(0, 0);
    let ok = unsafe {
        GetProcessInformation(
            h.0,
            ProcessPowerThrottling,
            &mut st as *mut _ as *mut c_void,
            std::mem::size_of::<PROCESS_POWER_THROTTLING_STATE>() as u32,
        )
    };
    if ok == 0 {
        return None;
    }
    let bit = PROCESS_POWER_THROTTLING_EXECUTION_SPEED;
    Some(st.ControlMask & bit != 0 && st.StateMask & bit != 0)
}

/// Desliga o Modo de Eficiência do processo e pede ao Windows para não religar
/// (e para respeitar a resolução de timer mesmo minimizado, no Windows 11).
pub fn remover_eficiencia(pid: u32) -> Result<(), ErroProcesso> {
    let h = abrir(pid, PROCESS_SET_INFORMATION)?;
    let tentar = |controle: u32| -> bool {
        let st = estado_eficiencia(controle, 0);
        unsafe {
            SetProcessInformation(
                h.0,
                ProcessPowerThrottling,
                &st as *const _ as *const c_void,
                std::mem::size_of::<PROCESS_POWER_THROTTLING_STATE>() as u32,
            ) != 0
        }
    };
    let completo = PROCESS_POWER_THROTTLING_EXECUTION_SPEED | PROCESS_POWER_THROTTLING_IGNORE_TIMER_RESOLUTION;
    if tentar(completo) || tentar(PROCESS_POWER_THROTTLING_EXECUTION_SPEED) {
        Ok(())
    } else {
        Err(ultimo_erro())
    }
}

/// Devolve ao Windows a decisão sobre o Modo de Eficiência (desfaz `remover_eficiencia`).
pub fn devolver_eficiencia_ao_windows(pid: u32) -> Result<(), ErroProcesso> {
    let h = abrir(pid, PROCESS_SET_INFORMATION)?;
    let st = estado_eficiencia(0, 0);
    let ok = unsafe {
        SetProcessInformation(
            h.0,
            ProcessPowerThrottling,
            &st as *const _ as *const c_void,
            std::mem::size_of::<PROCESS_POWER_THROTTLING_STATE>() as u32,
        )
    };
    if ok == 0 {
        Err(ultimo_erro())
    } else {
        Ok(())
    }
}

/// Prioridade de memória: 1 (muito baixa) a 5 (normal).
pub fn definir_prioridade_memoria(pid: u32, nivel: u8) -> Result<(), ErroProcesso> {
    let h = abrir(pid, PROCESS_SET_INFORMATION)?;
    let info = MEMORY_PRIORITY_INFORMATION { MemoryPriority: u32::from(nivel.clamp(1, 5)) };
    let ok = unsafe {
        SetProcessInformation(
            h.0,
            ProcessMemoryPriority,
            &info as *const _ as *const c_void,
            std::mem::size_of::<MEMORY_PRIORITY_INFORMATION>() as u32,
        )
    };
    if ok == 0 {
        Err(ultimo_erro())
    } else {
        Ok(())
    }
}

/// Prioridade de E/S (disco): muito baixa, baixa ou normal. Nunca "alta".
pub fn definir_prioridade_es(pid: u32, nivel: PrioridadeEs) -> Result<(), ErroProcesso> {
    let h = abrir(pid, PROCESS_SET_INFORMATION)?;
    let valor: u32 = match nivel {
        PrioridadeEs::MuitoBaixa => 0,
        PrioridadeEs::Baixa => 1,
        PrioridadeEs::Normal => 2,
    };
    let st = unsafe { NtSetInformationProcess(h.0, PROCESS_IO_PRIORITY, &valor as *const u32 as *const c_void, 4) };
    match st {
        s if s >= 0 => Ok(()),
        STATUS_ACCESS_DENIED => Err(ErroProcesso::AcessoNegado),
        s => Err(ErroProcesso::Falhou(format!("NTSTATUS 0x{:08X}", s as u32))),
    }
}

/// CPU Sets padrão do processo (o Windows prefere esses núcleos sem proibir os outros).
/// Lista vazia = volta ao normal (todos os núcleos).
pub fn definir_cpu_sets(pid: u32, ids: &[u32]) -> Result<(), ErroProcesso> {
    let h = abrir(pid, PROCESS_SET_LIMITED_INFORMATION)?;
    let ok = unsafe {
        if ids.is_empty() {
            SetProcessDefaultCpuSets(h.0, std::ptr::null(), 0)
        } else {
            SetProcessDefaultCpuSets(h.0, ids.as_ptr(), ids.len() as u32)
        }
    };
    if ok == 0 {
        Err(ultimo_erro())
    } else {
        Ok(())
    }
}

/// Afinidade atual: (máscara do processo, máscara do sistema).
pub fn ler_afinidade(pid: u32) -> Result<(usize, usize), ErroProcesso> {
    let h = abrir(pid, PROCESS_QUERY_LIMITED_INFORMATION)?;
    let (mut processo, mut sistema) = (0usize, 0usize);
    if unsafe { GetProcessAffinityMask(h.0, &mut processo, &mut sistema) } == 0 {
        Err(ultimo_erro())
    } else {
        Ok((processo, sistema))
    }
}

/// Afinidade rígida (opção avançada): o processo só roda nos núcleos da máscara.
pub fn definir_afinidade(pid: u32, mascara: usize) -> Result<(), ErroProcesso> {
    if mascara == 0 {
        return Err(ErroProcesso::Falhou("máscara de núcleos vazia".into()));
    }
    let h = abrir(pid, PROCESS_SET_INFORMATION | PROCESS_QUERY_LIMITED_INFORMATION)?;
    if unsafe { SetProcessAffinityMask(h.0, mascara) } == 0 {
        Err(ultimo_erro())
    } else {
        Ok(())
    }
}

/// Encerra um processo. Só usado pelo Watchdog quando o usuário marcou e confirmou essa ação.
pub fn encerrar(pid: u32) -> Result<(), ErroProcesso> {
    let h = abrir(pid, PROCESS_TERMINATE)?;
    if unsafe { TerminateProcess(h.0, 1) } == 0 {
        Err(ultimo_erro())
    } else {
        Ok(())
    }
}
