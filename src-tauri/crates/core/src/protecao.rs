//! Lista de proteção: o que o Xablau Optimizer NUNCA remove, desativa ou "otimiza".
//!
//! Microsoft Store, App Installer (winget), Visual C++, .NET, WebView2, Windows Update, áudio,
//! rede, instalador do Windows, criptografia e o antivírus instalado — além dos processos
//! essenciais do sistema. Toda comparação ignora maiúsculas/minúsculas.

/// Serviços que nunca têm o tipo de início alterado.
const SERVICOS: &[&str] = &[
    // Windows Update, Store e instalação
    "wuauserv", "UsoSvc", "WaaSMedicSvc", "BITS", "DoSvc", "TrustedInstaller", "msiserver", "InstallService",
    "AppXSvc", "ClipSVC", "LicenseManager", "StateRepository", "TokenBroker", "camsvc", "AppReadiness",
    // Criptografia, contas e segurança
    "CryptSvc", "KeyIso", "VaultSvc", "SamSs", "EFS", "BDESVC", "Appinfo", "wlidsvc", "NgcSvc", "NgcCtnrSvc",
    "WbioSrvc", "SecurityHealthService", "wscsvc", "WinDefend", "WdNisSvc", "Sense", "MDCoreSvc", "mpssvc", "BFE",
    "SgrmBroker", "seclogon", "Netlogon",
    // Áudio
    "Audiosrv", "AudioEndpointBuilder",
    // Rede
    "Dhcp", "Dnscache", "NlaSvc", "netprofm", "nsi", "LanmanWorkstation", "WlanSvc", "Wcmsvc", "WinHttpAutoProxySvc",
    "iphlpsvc", "Netman", "NcbService", "RasMan", "WwanSvc", "dot3svc", "EapHost", "IKEEXT", "PolicyAgent", "lmhosts",
    "DusmSvc", "NetSetupSvc",
    // Núcleo do sistema
    "RpcSs", "RpcEptMapper", "DcomLaunch", "EventLog", "EventSystem", "PlugPlay", "Power", "ProfSvc", "Schedule",
    "SENS", "ShellHWDetection", "Themes", "UserManager", "Winmgmt", "gpsvc", "LSM", "BrokerInfrastructure",
    "SystemEventsBroker", "TimeBrokerSvc", "CoreMessagingRegistrar", "DeviceInstall", "DsmSvc", "FontCache",
    "TextInputManagementService", "W32Time", "WpnService", "WpnUserService", "StorSvc", "VSS", "swprv", "SDRSVC",
    "DispBrokerDesktopSvc", "TabletInputService", "hidserv", "DPS", "WdiServiceHost", "WdiSystemHost", "cbdhsvc",
    "KeyboardFilter",
];

/// Famílias de serviços protegidas pelo começo do nome (antivírus e drivers de vídeo).
const SERVICOS_PREFIXOS: &[&str] = &[
    "AVP", "Kaspersky", "klnagent", "ekrn", "ESET", "avast", "AvastWsc", "AVG", "bdredline", "VSSERV", "Bitdefender",
    "McAfee", "mfe", "Norton", "Sophos", "MBAMService", "WRSVC", "PSUAService", "SentinelAgent", "CSFalcon", "TmCCSF",
    "TMBMServer", "ZoneAlarm", "NVDisplay", "AMD External Events", "igfxCUIService", "WinDefend", "WdNisSvc",
];

/// Apps da Store (pacotes AppX) que nunca são removidos — comparação pelo começo do nome.
const APPX_PREFIXOS: &[&str] = &[
    "Microsoft.WindowsStore", "Microsoft.StorePurchaseApp", "Microsoft.Services.Store.Engagement",
    "Microsoft.DesktopAppInstaller", "Microsoft.VCLibs", "Microsoft.NET.", "Microsoft.UI.Xaml",
    "Microsoft.WindowsAppRuntime", "Microsoft.WinAppRuntime", "Microsoft.SecHealthUI", "Microsoft.Windows.SecHealthUI",
    "Microsoft.Windows.Apprep.ChxApp", "Microsoft.Windows.ShellExperienceHost", "Microsoft.Windows.StartMenuExperienceHost",
    "MicrosoftWindows.Client.", "MicrosoftWindows.UndockedDevKit", "Microsoft.Windows.Search", "Microsoft.Windows.CloudExperienceHost",
    "Microsoft.Windows.ContentDeliveryManager", "Microsoft.AAD.BrokerPlugin", "Microsoft.AccountsControl", "Microsoft.CredDialogHost",
    "Microsoft.LockApp", "Microsoft.ECApp", "Microsoft.BioEnrollment", "Microsoft.Win32WebViewHost", "Microsoft.Windows.OOBENetwork",
    "Microsoft.Windows.PeopleExperienceHost", "Microsoft.Windows.PinningConfirmationDialog", "Microsoft.Windows.XGpuEjectDialog",
    "Microsoft.Windows.AssignedAccessLockApp", "Microsoft.Windows.CapturePicker", "Microsoft.Windows.NarratorQuickStart",
    "Microsoft.Windows.ParentalControls", "Microsoft.Windows.SecureAssessmentBrowser", "Microsoft.Windows.PrintQueueActionCenter",
    "Microsoft.MicrosoftEdge", "Microsoft.MicrosoftEdgeDevToolsClient", "NcsiUwpApp", "windows.immersivecontrolpanel",
    "Windows.PrintDialog", "Windows.CBSPreview", "1527c705-839a-4832-9118-54d4Bd6a0c89", "c5e2524a-ea46-4f67-841f-6a9465d9d515",
    "E2A4F912-2574-4A75-9BB0-0D023378592B", "F46D4000-FD22-4DB4-AC8E-4E1DDDE828FE", "Microsoft.GamingServices",
    "Microsoft.XboxIdentityProvider", "Microsoft.Xbox.TCUI", "Microsoft.XboxGameCallableUI", "Microsoft.HEIFImageExtension",
    "Microsoft.HEVCVideoExtension", "Microsoft.AV1VideoExtension", "Microsoft.VP9VideoExtensions", "Microsoft.WebMediaExtensions",
    "Microsoft.WebpImageExtension", "Microsoft.RawImageExtension", "Microsoft.MPEG2VideoExtension", "Microsoft.AVCEncoderVideoExtension",
    "Microsoft.DolbyAudioExtensions", "NVIDIACorp.NVIDIAControlPanel", "AdvancedMicroDevicesInc-2.AMDRadeonSoftware",
    "AppUp.IntelGraphicsExperience", "RealtekSemiconductorCorp.RealtekAudioControl",
];

/// Processos do sistema e do próprio Xablau: nunca recebem prioridade, CPU Set ou regra do Watchdog.
const PROCESSOS: &[&str] = &[
    "system", "registry", "idle", "secure system", "memory compression", "smss.exe", "csrss.exe", "wininit.exe",
    "winlogon.exe", "services.exe", "lsass.exe", "lsaiso.exe", "svchost.exe", "dwm.exe", "explorer.exe", "fontdrvhost.exe",
    "audiodg.exe", "sihost.exe", "ctfmon.exe", "taskhostw.exe", "runtimebroker.exe", "searchhost.exe", "searchindexer.exe",
    "startmenuexperiencehost.exe", "shellexperiencehost.exe", "textinputhost.exe", "securityhealthservice.exe",
    "securityhealthsystray.exe", "smartscreen.exe", "spoolsv.exe", "wudfhost.exe", "conhost.exe", "dllhost.exe",
    "wmiprvse.exe", "trustedinstaller.exe", "tiworker.exe", "msiexec.exe", "wuauclt.exe", "mousocoreworker.exe",
    "usoclient.exe", "vmmem", "vmmemws", "lockapp.exe", "logonui.exe", "consent.exe", "userinit.exe", "werfault.exe",
    "xablau-optimizer.exe", "xablau-vigia.exe", "xablau-restaurar.exe", "presentmon.exe",
];

/// Processos de antivírus conhecidos (nunca mexer neles, e usados para detectar antivírus ativo).
pub const ANTIVIRUS_PROCESSOS: &[&str] = &[
    "msmpeng.exe", "nissrv.exe", "mpdefendercoreservice.exe", "avp.exe", "avpui.exe", "kavtray.exe", "klnagent.exe",
    "ekrn.exe", "egui.exe", "avastsvc.exe", "avastui.exe", "avgsvc.exe", "avgui.exe", "bdagent.exe", "vsserv.exe",
    "bdservicehost.exe", "mcshield.exe", "mfemms.exe", "mfevtps.exe", "nortonsecurity.exe", "navapsvc.exe",
    "sophoshealth.exe", "savservice.exe", "mbamservice.exe", "mbamtray.exe", "wrsa.exe", "psanhost.exe",
    "sentinelagent.exe", "csfalconservice.exe", "ntrtscan.exe", "tmbmsrv.exe",
];

/// Programas que o desinstalador completo nunca remove (comparação por trecho do nome).
const PROGRAMAS: &[&str] = &[
    "microsoft edge webview2", "webview2 runtime", "microsoft edge", "microsoft visual c++", ".net", "microsoft windows desktop runtime",
    "microsoft asp.net", "microsoft update health tools", "update for windows", "microsoft gameinput", "nvidia graphics driver",
    "nvidia hd audio", "nvidia physx", "amd software", "amd chipset", "amd ryzen", "intel(r) graphics", "intel® graphics",
    "intel graphics", "intel(r) management engine", "intel chipset", "realtek", "kaspersky", "eset", "avast", "avg antivirus",
    "bitdefender", "norton", "mcafee", "malwarebytes", "windows defender", "microsoft defender", "xablau optimizer",
];

fn igual(a: &str, b: &str) -> bool {
    a.eq_ignore_ascii_case(b)
}

fn comeca_com(texto: &str, prefixo: &str) -> bool {
    texto.len() >= prefixo.len() && texto.as_bytes()[..prefixo.len()].eq_ignore_ascii_case(prefixo.as_bytes())
}

/// Serviço protegido? Serviços por usuário ("WpnUserService_1a2b3") contam pelo nome-base.
pub fn servico_protegido(nome: &str) -> bool {
    let base = match nome.rsplit_once('_') {
        Some((b, sufixo)) if sufixo.chars().all(|c| c.is_ascii_hexdigit()) && !b.is_empty() => b,
        _ => nome,
    };
    SERVICOS.iter().any(|s| igual(s, nome) || igual(s, base)) || SERVICOS_PREFIXOS.iter().any(|p| comeca_com(nome, p))
}

/// Pacote AppX protegido?
pub fn appx_protegido(nome: &str) -> bool {
    APPX_PREFIXOS.iter().any(|p| comeca_com(nome, p))
}

/// Processo protegido (sistema, antivírus ou o próprio Xablau)?
pub fn processo_protegido(nome_exe: &str) -> bool {
    PROCESSOS.iter().chain(ANTIVIRUS_PROCESSOS.iter()).any(|p| igual(p, nome_exe))
}

/// Processo de antivírus?
pub fn processo_antivirus(nome_exe: &str) -> bool {
    ANTIVIRUS_PROCESSOS.iter().any(|p| igual(p, nome_exe))
}

/// Programa instalado protegido contra o desinstalador completo?
pub fn programa_protegido(nome_exibicao: &str) -> bool {
    let n = nome_exibicao.to_lowercase();
    PROGRAMAS.iter().any(|p| n.contains(p))
}

/// Nome de pacote AppX válido (sem curingas nem caracteres especiais).
pub fn nome_appx_valido(nome: &str) -> bool {
    !nome.is_empty() && nome.len() <= 128 && nome.chars().all(|c| c.is_ascii_alphanumeric() || "._-".contains(c))
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn protecoes_basicas() {
        assert!(servico_protegido("wuauserv"));
        assert!(servico_protegido("WpnUserService_3f2a1"));
        assert!(servico_protegido("AVP21.3"));
        assert!(!servico_protegido("Fax"));
        assert!(appx_protegido("Microsoft.WindowsStore"));
        assert!(appx_protegido("Microsoft.VCLibs.140.00"));
        assert!(!appx_protegido("Microsoft.BingNews"));
        assert!(processo_protegido("AVP.EXE"));
        assert!(!processo_protegido("CapCut.exe"));
        assert!(programa_protegido("Microsoft Edge WebView2 Runtime"));
    }
}
