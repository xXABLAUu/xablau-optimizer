//! Informações rápidas do sistema: versão do Windows, Modo de Segurança, energia, ociosidade,
//! uso de CPU e memória. Tudo por APIs nativas (sem PowerShell), para o vigia continuar leve.

use serde::Serialize;
use std::path::PathBuf;
use std::sync::OnceLock;
use windows_sys::Win32::Foundation::FILETIME;
use windows_sys::Win32::System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};
use windows_sys::Win32::System::SystemInformation::{GetSystemDirectoryW, GetTickCount64, GlobalMemoryStatusEx, MEMORYSTATUSEX};
use windows_sys::Win32::System::Threading::GetSystemTimes;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO};
use windows_sys::Win32::UI::Shell::IsUserAnAdmin;
use windows_sys::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CLEANBOOT};

/// Pasta `System32` real (via API, nunca pela variável de ambiente que pode ser forjada).
pub fn dir_sistema32() -> PathBuf {
    static DIR: OnceLock<PathBuf> = OnceLock::new();
    DIR.get_or_init(|| {
        let mut buf = [0u16; 512];
        let n = unsafe { GetSystemDirectoryW(buf.as_mut_ptr(), buf.len() as u32) } as usize;
        if n == 0 || n >= buf.len() {
            PathBuf::from(r"C:\Windows\System32")
        } else {
            PathBuf::from(String::from_utf16_lossy(&buf[..n]))
        }
    })
    .clone()
}

/// `true` quando o Windows foi iniciado no Modo de Segurança.
pub fn modo_seguranca() -> bool {
    unsafe { GetSystemMetrics(SM_CLEANBOOT) != 0 }
}

/// `true` quando o processo atual roda como administrador.
pub fn e_administrador() -> bool {
    unsafe { IsUserAnAdmin() != 0 }
}

/// Versão real do Windows (sem a "mentira de compatibilidade" do GetVersionEx).
#[derive(Debug, Clone, Copy, Serialize)]
pub struct VersaoWindows {
    pub major: u32,
    pub minor: u32,
    pub build: u32,
    pub ubr: u32,
    pub windows11: bool,
}

#[repr(C)]
struct OsVersionInfoW {
    tamanho: u32,
    major: u32,
    minor: u32,
    build: u32,
    plataforma: u32,
    csd: [u16; 128],
}

#[link(name = "ntdll", kind = "raw-dylib")]
extern "system" {
    fn RtlGetVersion(info: *mut OsVersionInfoW) -> i32;
}

pub fn versao_windows() -> VersaoWindows {
    let mut info = OsVersionInfoW {
        tamanho: std::mem::size_of::<OsVersionInfoW>() as u32,
        major: 0,
        minor: 0,
        build: 0,
        plataforma: 0,
        csd: [0; 128],
    };
    unsafe {
        RtlGetVersion(&mut info);
    }
    let ubr = crate::registro::ler_dword(r"HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion", "UBR").unwrap_or(0);
    VersaoWindows { major: info.major, minor: info.minor, build: info.build, ubr, windows11: info.major == 10 && info.build >= 22000 }
}

/// Situação de energia (tomada/bateria).
#[derive(Debug, Clone, Copy, Serialize)]
pub struct StatusEnergia {
    pub na_tomada: bool,
    pub tem_bateria: bool,
    pub bateria_pct: Option<u8>,
    pub economia_de_bateria: bool,
}

impl StatusEnergia {
    /// Notebook funcionando na bateria: nunca aplicar Desempenho Máximo.
    pub fn na_bateria(&self) -> bool {
        self.tem_bateria && !self.na_tomada
    }
}

pub fn status_energia() -> StatusEnergia {
    let mut s: SYSTEM_POWER_STATUS = unsafe { std::mem::zeroed() };
    if unsafe { GetSystemPowerStatus(&mut s) } == 0 {
        return StatusEnergia { na_tomada: true, tem_bateria: false, bateria_pct: None, economia_de_bateria: false };
    }
    // BatteryFlag 128 = sem bateria; 255 = desconhecido. ACLineStatus 0 = desligado da tomada.
    let tem_bateria = s.BatteryFlag != 128 && s.BatteryFlag != 255;
    StatusEnergia {
        na_tomada: s.ACLineStatus != 0,
        tem_bateria,
        bateria_pct: if tem_bateria && s.BatteryLifePercent <= 100 { Some(s.BatteryLifePercent) } else { None },
        economia_de_bateria: s.SystemStatusFlag == 1,
    }
}

/// Segundos desde a última vez que o teclado ou o mouse foram usados (sessão atual).
pub fn segundos_sem_uso() -> u64 {
    let mut lii = LASTINPUTINFO { cbSize: std::mem::size_of::<LASTINPUTINFO>() as u32, dwTime: 0 };
    unsafe {
        if GetLastInputInfo(&mut lii) == 0 {
            return 0;
        }
        let agora = GetTickCount64() as u32;
        u64::from(agora.wrapping_sub(lii.dwTime) / 1000)
    }
}

/// Uso de memória física.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct Memoria {
    pub total: u64,
    pub disponivel: u64,
    pub uso_pct: u32,
}

pub fn memoria() -> Memoria {
    let mut m: MEMORYSTATUSEX = unsafe { std::mem::zeroed() };
    m.dwLength = std::mem::size_of::<MEMORYSTATUSEX>() as u32;
    if unsafe { GlobalMemoryStatusEx(&mut m) } == 0 {
        return Memoria { total: 0, disponivel: 0, uso_pct: 0 };
    }
    Memoria { total: m.ullTotalPhys, disponivel: m.ullAvailPhys, uso_pct: m.dwMemoryLoad }
}

fn ft(f: FILETIME) -> u64 {
    (u64::from(f.dwHighDateTime) << 32) | u64::from(f.dwLowDateTime)
}

/// Mede o uso total de CPU entre duas chamadas.
#[derive(Debug, Default)]
pub struct AmostradorCpu {
    anterior: Option<(u64, u64, u64)>,
}

impl AmostradorCpu {
    pub fn novo() -> Self {
        let mut a = Self::default();
        let _ = a.amostrar();
        a
    }

    /// Porcentagem de uso desde a última amostra (0 a 100).
    pub fn amostrar(&mut self) -> f32 {
        let mut ocioso: FILETIME = unsafe { std::mem::zeroed() };
        let mut nucleo: FILETIME = unsafe { std::mem::zeroed() };
        let mut usuario: FILETIME = unsafe { std::mem::zeroed() };
        if unsafe { GetSystemTimes(&mut ocioso, &mut nucleo, &mut usuario) } == 0 {
            return 0.0;
        }
        let atual = (ft(ocioso), ft(nucleo), ft(usuario));
        let uso = match self.anterior {
            Some((i0, k0, u0)) => {
                let i = atual.0.saturating_sub(i0);
                let total = atual.1.saturating_sub(k0) + atual.2.saturating_sub(u0);
                if total == 0 { 0.0 } else { (total.saturating_sub(i) as f64 * 100.0 / total as f64) as f32 }
            }
            None => 0.0,
        };
        self.anterior = Some(atual);
        uso.clamp(0.0, 100.0)
    }
}
