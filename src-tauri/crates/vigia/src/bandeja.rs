//! Ícone na bandeja do vigia, com menu de atalhos (Modo Turbo, perfis, abrir o programa, pausar).
//! Win32 puro: sem WebView2, para o vigia ficar leve.

use std::sync::mpsc::Sender;
use windows_sys::core::PCWSTR;
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, WPARAM};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::Shell::{
    Shell_NotifyIconW, NIF_ICON, NIF_INFO, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NIM_MODIFY, NOTIFYICONDATAW,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyMenu, DispatchMessageW, GetCursorPos,
    LoadImageW, PostQuitMessage, RegisterClassExW, SetForegroundWindow, TrackPopupMenu, TranslateMessage, CheckMenuItem,
    HICON, HMENU, IMAGE_ICON, LR_DEFAULTSIZE, LR_LOADFROMFILE, MF_CHECKED, MF_SEPARATOR, MF_STRING, MF_UNCHECKED, MSG,
    TPM_RIGHTBUTTON, WM_APP, WM_COMMAND, WM_DESTROY, WNDCLASSEXW, WS_OVERLAPPED,
};

use crate::acoes::Acao;

const MSG_BANDEJA: u32 = WM_APP + 1;
const ID_ABRIR: usize = 1;
const ID_TURBO: usize = 2;
const ID_PAUSAR: usize = 3;
const ID_RESTAURAR: usize = 10;
const ID_SAIR: usize = 99;
const ID_PERFIL_BASE: usize = 1000;

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// Itens de perfil mostrados no menu (preenchidos a cada abertura).
pub struct ItemMenuPerfil {
    pub id: String,
    pub nome: String,
    pub ativo: bool,
}

/// Dados que o menu precisa para se desenhar (atualizados pelo loop principal).
#[derive(Default)]
pub struct EstadoMenu {
    pub turbo: bool,
    pub pausado: bool,
    pub perfis: Vec<ItemMenuPerfil>,
}

static mut PONTEIRO_BANDEJA: *mut Bandeja = std::ptr::null_mut();

pub struct Bandeja {
    hwnd: HWND,
    icone: HICON,
    tx: Sender<Acao>,
    estado: EstadoMenu,
}

impl Bandeja {
    /// Cria a janela oculta e o ícone da bandeja. `caminho_icone` = vigia.ico ao lado do executável.
    pub fn criar(tx: Sender<Acao>, caminho_icone: &str) -> Result<Box<Bandeja>, String> {
        unsafe {
            let hinst = GetModuleHandleW(std::ptr::null());
            let classe = wide("XablauVigiaBandeja");
            let wc = WNDCLASSEXW {
                cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                style: 0,
                lpfnWndProc: Some(proc_janela),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: hinst,
                hIcon: std::ptr::null_mut(),
                hCursor: std::ptr::null_mut(),
                hbrBackground: std::ptr::null_mut(),
                lpszMenuName: std::ptr::null(),
                lpszClassName: classe.as_ptr(),
                hIconSm: std::ptr::null_mut(),
            };
            RegisterClassExW(&wc);
            let titulo = wide("Xablau Vigia");
            let hwnd = CreateWindowExW(
                0,
                classe.as_ptr(),
                titulo.as_ptr(),
                WS_OVERLAPPED,
                0,
                0,
                0,
                0,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                hinst,
                std::ptr::null(),
            );
            if hwnd.is_null() {
                return Err("Não foi possível criar a janela do vigia.".into());
            }
            let caminho = wide(caminho_icone);
            let icone = LoadImageW(std::ptr::null_mut(), caminho.as_ptr() as PCWSTR, IMAGE_ICON, 0, 0, LR_LOADFROMFILE | LR_DEFAULTSIZE) as HICON;

            let mut bandeja = Box::new(Bandeja { hwnd, icone, tx, estado: EstadoMenu::default() });
            PONTEIRO_BANDEJA = bandeja.as_mut() as *mut Bandeja;

            let mut nid = montar_nid(hwnd, icone);
            nid.uFlags = NIF_MESSAGE | NIF_ICON | NIF_TIP;
            Shell_NotifyIconW(NIM_ADD, &nid);
            Ok(bandeja)
        }
    }

    /// Atualiza a dica e o estado do menu.
    pub fn atualizar(&mut self, estado: EstadoMenu, dica: &str) {
        self.estado = estado;
        unsafe {
            let mut nid = montar_nid(self.hwnd, self.icone);
            nid.uFlags = NIF_TIP;
            let d = wide(dica);
            let n = d.len().min(nid.szTip.len());
            nid.szTip[..n].copy_from_slice(&d[..n]);
            Shell_NotifyIconW(NIM_MODIFY, &nid);
        }
    }

    /// Mostra uma notificação (balão) discreta.
    pub fn notificar(&self, titulo: &str, texto: &str) {
        unsafe {
            let mut nid = montar_nid(self.hwnd, self.icone);
            nid.uFlags = NIF_INFO;
            let t = wide(titulo);
            let n = t.len().min(nid.szInfoTitle.len());
            nid.szInfoTitle[..n].copy_from_slice(&t[..n]);
            let c = wide(texto);
            let m = c.len().min(nid.szInfo.len());
            nid.szInfo[..m].copy_from_slice(&c[..m]);
            Shell_NotifyIconW(NIM_MODIFY, &nid);
        }
    }

    fn abrir_menu(&self) {
        unsafe {
            let menu = CreatePopupMenu();
            item(menu, ID_ABRIR, "Abrir o Xablau Optimizer");
            separador(menu);
            marcavel(menu, ID_TURBO, "Modo Turbo", self.estado.turbo);
            marcavel(menu, ID_PAUSAR, "Pausar as automações", self.estado.pausado);
            if !self.estado.perfis.is_empty() {
                separador(menu);
                for (i, p) in self.estado.perfis.iter().enumerate().take(50) {
                    marcavel(menu, ID_PERFIL_BASE + i, &format!("Perfil: {}", p.nome), p.ativo);
                }
            }
            separador(menu);
            item(menu, ID_RESTAURAR, "Restaurar tudo (kit de emergência)");
            item(menu, ID_SAIR, "Encerrar o vigia");

            let mut pt = POINT { x: 0, y: 0 };
            GetCursorPos(&mut pt);
            SetForegroundWindow(self.hwnd);
            TrackPopupMenu(menu, TPM_RIGHTBUTTON, pt.x, pt.y, 0, self.hwnd, std::ptr::null());
            DestroyMenu(menu);
        }
    }

    fn ao_comando(&self, id: usize) {
        let acao = match id {
            ID_ABRIR => Some(Acao::AbrirInterface),
            ID_TURBO => Some(Acao::AlternarTurbo),
            ID_PAUSAR => Some(Acao::AlternarPausa),
            ID_RESTAURAR => Some(Acao::AbrirKit),
            ID_SAIR => Some(Acao::Encerrar),
            n if (ID_PERFIL_BASE..ID_PERFIL_BASE + self.estado.perfis.len()).contains(&n) => {
                self.estado.perfis.get(n - ID_PERFIL_BASE).map(|p| Acao::AlternarPerfil(p.id.clone()))
            }
            _ => None,
        };
        if let Some(a) = acao {
            let _ = self.tx.send(a);
        }
    }
}

impl Drop for Bandeja {
    fn drop(&mut self) {
        unsafe {
            let nid = montar_nid(self.hwnd, self.icone);
            Shell_NotifyIconW(NIM_DELETE, &nid);
            PONTEIRO_BANDEJA = std::ptr::null_mut();
        }
    }
}

fn montar_nid(hwnd: HWND, icone: HICON) -> NOTIFYICONDATAW {
    let mut nid: NOTIFYICONDATAW = unsafe { std::mem::zeroed() };
    nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
    nid.hWnd = hwnd;
    nid.uID = 1;
    nid.uCallbackMessage = MSG_BANDEJA;
    nid.hIcon = icone;
    nid
}

unsafe fn item(menu: HMENU, id: usize, texto: &str) {
    let t = wide(texto);
    AppendMenuW(menu, MF_STRING, id, t.as_ptr());
}

unsafe fn marcavel(menu: HMENU, id: usize, texto: &str, marcado: bool) {
    let t = wide(texto);
    AppendMenuW(menu, MF_STRING | if marcado { MF_CHECKED } else { MF_UNCHECKED }, id, t.as_ptr());
    let _ = CheckMenuItem;
}

unsafe fn separador(menu: HMENU) {
    AppendMenuW(menu, MF_SEPARATOR, 0, std::ptr::null());
}

unsafe extern "system" fn proc_janela(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    const WM_RBUTTONUP: u32 = 0x0205;
    const WM_LBUTTONDBLCLK: u32 = 0x0203;
    const WM_CONTEXTMENU_TRAY: u32 = 0x007B;
    match msg {
        MSG_BANDEJA => {
            let evento = (lparam as u32) & 0xFFFF;
            if let Some(b) = PONTEIRO_BANDEJA.as_ref() {
                if evento == WM_RBUTTONUP || evento == WM_CONTEXTMENU_TRAY {
                    b.abrir_menu();
                } else if evento == WM_LBUTTONDBLCLK {
                    b.ao_comando(ID_ABRIR);
                }
            }
            0
        }
        WM_COMMAND => {
            let id = (wparam & 0xFFFF) as usize;
            if let Some(b) = PONTEIRO_BANDEJA.as_ref() {
                b.ao_comando(id);
            }
            0
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

/// Processa as mensagens pendentes da bandeja (chamado no loop principal). `false` = recebeu "sair".
pub fn bombear_mensagens() -> bool {
    unsafe {
        let mut msg: MSG = std::mem::zeroed();
        // PeekMessage não bloqueia; usamos GetMessage com timeout via PostMessage no loop.
        while peek(&mut msg) {
            if msg.message == 0x0012 {
                // WM_QUIT
                return false;
            }
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
    true
}

unsafe fn peek(msg: *mut MSG) -> bool {
    use windows_sys::Win32::UI::WindowsAndMessaging::{PeekMessageW, PM_REMOVE};
    PeekMessageW(msg, std::ptr::null_mut(), 0, 0, PM_REMOVE) != 0
}

