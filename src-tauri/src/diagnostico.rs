//! Coletas de diagnóstico (só leem, não alteram nada): hardware, verificações de BIOS,
//! saúde dos SSDs, monitor de culpados, memória e energia em tempo real.
//! O que é rápido vem por API nativa; o que exige WMI/CIM vem por PowerShell embutido.

use serde::Serialize;
use std::time::Duration;
use xablau_core::comando;
use xablau_core::sistema::{self, VersaoWindows};

/// Resumo de hardware para o Painel e a Ficha do PC.
#[derive(Debug, Clone, Serialize, Default)]
pub struct Hardware {
    pub cpu: String,
    pub nucleos_fisicos: u32,
    pub nucleos_logicos: u32,
    pub ccds: usize,
    pub gpu: Vec<String>,
    pub placa_mae: String,
    pub bios: String,
    pub ram_total_gb: f64,
    pub ram_velocidade_mhz: u32,
    pub ram_modo: String,
    pub discos: Vec<String>,
    pub windows: String,
    pub notebook: bool,
}

/// Lê o hardware (uma chamada de PowerShell com CIM para o que a API não dá fácil).
pub fn hardware(topologia: &xablau_core::topologia::Topologia, versao: &VersaoWindows) -> Hardware {
    let mem = sistema::memoria();
    let energia = sistema::status_energia();
    let mut hw = Hardware {
        nucleos_fisicos: topologia.nucleos as u32,
        nucleos_logicos: topologia.logicos as u32,
        ccds: topologia.grupos.len(),
        ram_total_gb: mem.total as f64 / 1_073_741_824.0,
        notebook: energia.tem_bateria,
        windows: if versao.windows11 {
            format!("Windows 11 (build {}.{})", versao.build, versao.ubr)
        } else {
            format!("Windows 10 (build {}.{})", versao.build, versao.ubr)
        },
        ..Default::default()
    };

    // CIM em uma chamada só, saída em JSON (campos simples).
    let script = "\
$ErrorActionPreference='SilentlyContinue';\
$cpu=Get-CimInstance Win32_Processor | Select-Object -First 1;\
$bb=Get-CimInstance Win32_BaseBoard | Select-Object -First 1;\
$bios=Get-CimInstance Win32_BIOS | Select-Object -First 1;\
$gpu=Get-CimInstance Win32_VideoController | ForEach-Object { $_.Name };\
$mem=Get-CimInstance Win32_PhysicalMemory;\
$vel=($mem | Measure-Object -Property Speed -Maximum).Maximum;\
$disk=Get-CimInstance Win32_DiskDrive | ForEach-Object { $_.Model };\
$o=[ordered]@{cpu=$cpu.Name;placa=($bb.Manufacturer+' '+$bb.Product);bios=($bios.SMBIOSBIOSVersion+' ('+$bios.ReleaseDate+')');gpu=@($gpu);ram_mhz=[int]$vel;discos=@($disk)};\
$o | ConvertTo-Json -Compress -Depth 4";
    if let Ok(s) = comando::powershell(script, &[], Duration::from_secs(60)) {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(s.stdout.trim()) {
            hw.cpu = v.get("cpu").and_then(|x| x.as_str()).unwrap_or("").trim().to_string();
            hw.placa_mae = v.get("placa").and_then(|x| x.as_str()).unwrap_or("").trim().to_string();
            hw.bios = v.get("bios").and_then(|x| x.as_str()).unwrap_or("").trim().to_string();
            hw.ram_velocidade_mhz = v.get("ram_mhz").and_then(serde_json::Value::as_u64).unwrap_or(0) as u32;
            hw.gpu = valor_para_lista(v.get("gpu"));
            hw.discos = valor_para_lista(v.get("discos"));
        }
    }
    hw.ram_modo = if hw.ram_velocidade_mhz >= 2667 { "XMP/DOCP provavelmente ativo".into() } else { "Verificar XMP/DOCP na BIOS".into() };
    hw
}

fn valor_para_lista(v: Option<&serde_json::Value>) -> Vec<String> {
    match v {
        Some(serde_json::Value::Array(a)) => a.iter().filter_map(|x| x.as_str()).map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect(),
        Some(serde_json::Value::String(s)) => vec![s.trim().to_string()],
        _ => Vec::new(),
    }
}

/// Uso em tempo real (Painel).
#[derive(Debug, Clone, Serialize)]
pub struct UsoTempoReal {
    pub cpu_pct: f32,
    pub ram_pct: u32,
    pub ram_usada_gb: f64,
    pub ram_total_gb: f64,
    pub na_bateria: bool,
    pub bateria_pct: Option<u8>,
}

pub fn uso_tempo_real(cpu: &mut sistema::AmostradorCpu) -> UsoTempoReal {
    let mem = sistema::memoria();
    let energia = sistema::status_energia();
    UsoTempoReal {
        cpu_pct: cpu.amostrar(),
        ram_pct: mem.uso_pct,
        ram_usada_gb: (mem.total.saturating_sub(mem.disponivel)) as f64 / 1_073_741_824.0,
        ram_total_gb: mem.total as f64 / 1_073_741_824.0,
        na_bateria: energia.na_bateria(),
        bateria_pct: energia.bateria_pct,
    }
}

/// Um processo no "monitor de culpados".
#[derive(Debug, Clone, Serialize)]
pub struct Culpado {
    pub pid: u32,
    pub nome: String,
    pub memoria_mb: u64,
    pub es_mb: u64,
    pub eficiencia: Option<bool>,
    pub protegido: bool,
}

/// Top processos por memória e por E/S (para o monitor de culpados).
pub fn culpados(limite: usize) -> Vec<Culpado> {
    let mut proc = xablau_core::processos::instantaneo();
    proc.sort_by(|a, b| (b.memoria_privada + b.leitura_bytes + b.escrita_bytes).cmp(&(a.memoria_privada + a.leitura_bytes + a.escrita_bytes)));
    proc.into_iter()
        .filter(|p| p.pid > 4)
        .take(limite)
        .map(|p| Culpado {
            eficiencia: xablau_core::processos::eficiencia_ativa(p.pid),
            protegido: xablau_core::protecao::processo_protegido(&p.nome),
            pid: p.pid,
            nome: p.nome,
            memoria_mb: p.memoria_privada / (1024 * 1024),
            es_mb: (p.leitura_bytes + p.escrita_bytes) / (1024 * 1024),
        })
        .collect()
}

/// Saúde de um SSD, com validação de dados não confiáveis (seção 11.1).
#[derive(Debug, Clone, Serialize)]
pub struct SaudeSsd {
    pub modelo: String,
    pub tipo: String,
    pub tamanho_gb: f64,
    pub confiavel: bool,
    pub desgaste_pct: Option<u8>,
    pub temperatura_c: Option<u8>,
    pub horas_ligado: Option<u64>,
    pub erros: Option<u64>,
    pub aviso: Option<String>,
}

/// Lê a saúde dos discos físicos. Se os contadores vierem vazios/zerados/inconsistentes, marca
/// como "dados não confiáveis" e NÃO calcula nota — exatamente como pedido para SSDs genéricos.
pub fn saude_ssds() -> Vec<SaudeSsd> {
    let script = "\
$ErrorActionPreference='SilentlyContinue';\
$res=@();\
foreach($d in Get-PhysicalDisk){\
 $r=$d | Get-StorageReliabilityCounter;\
 $res+=[ordered]@{modelo=$d.FriendlyName;tipo=$d.MediaType;tam=[double]$d.Size;\
  desgaste=$r.Wear;temp=$r.Temperature;horas=$r.PowerOnHours;\
  erros=([int64]$r.ReadErrorsTotal + [int64]$r.WriteErrorsTotal)}\
};\
$res | ConvertTo-Json -Compress -Depth 4";
    let Ok(s) = comando::powershell(script, &[], Duration::from_secs(90)) else { return Vec::new() };
    let txt = s.stdout.trim();
    if txt.is_empty() {
        return Vec::new();
    }
    let valor: serde_json::Value = match serde_json::from_str(txt) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    let lista = match valor {
        serde_json::Value::Array(a) => a,
        v @ serde_json::Value::Object(_) => vec![v],
        _ => return Vec::new(),
    };
    lista.into_iter().map(ssd_de_json).collect()
}

fn ssd_de_json(v: serde_json::Value) -> SaudeSsd {
    let modelo = v.get("modelo").and_then(|x| x.as_str()).unwrap_or("Disco").trim().to_string();
    let tipo = match v.get("tipo").and_then(serde_json::Value::as_i64) {
        Some(3) => "HDD",
        Some(4) => "SSD",
        _ => "Desconhecido",
    }
    .to_string();
    let tamanho_gb = v.get("tam").and_then(serde_json::Value::as_f64).unwrap_or(0.0) / 1_073_741_824.0;
    let desgaste = v.get("desgaste").and_then(serde_json::Value::as_u64);
    let temp = v.get("temp").and_then(serde_json::Value::as_u64);
    let horas = v.get("horas").and_then(serde_json::Value::as_u64);
    let erros = v.get("erros").and_then(serde_json::Value::as_u64);

    // Validação: se desgaste, temperatura e horas vierem todos ausentes/zerados, os dados SMART
    // deste disco não são confiáveis (comum em SSDs Kingdian e genéricos).
    let tudo_vazio = desgaste.unwrap_or(0) == 0 && temp.unwrap_or(0) == 0 && horas.unwrap_or(0) == 0;
    let faltam = desgaste.is_none() && temp.is_none() && horas.is_none();
    let confiavel = !(tudo_vazio || faltam);

    SaudeSsd {
        modelo,
        tipo,
        tamanho_gb,
        confiavel,
        desgaste_pct: if confiavel { desgaste.map(|d| d.min(100) as u8) } else { None },
        temperatura_c: if confiavel { temp.map(|t| t.min(255) as u8) } else { None },
        horas_ligado: if confiavel { horas } else { None },
        erros: if confiavel { erros } else { None },
        aviso: (!confiavel).then(|| "Dados não confiáveis para este SSD (SMART incompleto). A nota de saúde não é calculada.".to_string()),
    }
}

/// Ficha do PC exportável, SEM dados pessoais (seção 11.12).
pub fn ficha_pc(hw: &Hardware, ssds: &[SaudeSsd], markdown: bool) -> String {
    let gpu = if hw.gpu.is_empty() { "—".to_string() } else { hw.gpu.join(", ") };
    let discos: Vec<String> = ssds
        .iter()
        .map(|d| {
            let saude = if d.confiavel {
                match (d.desgaste_pct, d.temperatura_c) {
                    (Some(w), Some(t)) => format!("desgaste {w}%, {t}°C"),
                    _ => "ok".to_string(),
                }
            } else {
                "saúde: dados não confiáveis".to_string()
            };
            format!("{} ({:.0} GB, {}, {})", d.modelo, d.tamanho_gb, d.tipo, saude)
        })
        .collect();
    let discos_txt = if discos.is_empty() { "—".to_string() } else { discos.join("; ") };

    if markdown {
        format!(
            "## Ficha do PC (Xablau Optimizer)\n\n\
            - **CPU:** {}\n- **Núcleos:** {} físicos / {} lógicos ({} CCD/grupos de cache)\n\
            - **GPU:** {}\n- **Placa-mãe:** {}\n- **BIOS:** {}\n\
            - **RAM:** {:.0} GB @ {} MHz ({})\n- **Discos:** {}\n- **Sistema:** {}\n\n\
            _Gerada sem dados pessoais (sem nome de usuário, PC, números de série, chave do Windows, IP ou MAC)._\n",
            vazio(&hw.cpu), hw.nucleos_fisicos, hw.nucleos_logicos, hw.ccds, gpu, vazio(&hw.placa_mae),
            vazio(&hw.bios), hw.ram_total_gb, hw.ram_velocidade_mhz, hw.ram_modo, discos_txt, hw.windows
        )
    } else {
        format!(
            "Ficha do PC (Xablau Optimizer)\n\
            CPU: {}\nNúcleos: {} físicos / {} lógicos ({} CCD/grupos de cache)\n\
            GPU: {}\nPlaca-mãe: {}\nBIOS: {}\n\
            RAM: {:.0} GB @ {} MHz ({})\nDiscos: {}\nSistema: {}\n\
            (Sem dados pessoais.)",
            vazio(&hw.cpu), hw.nucleos_fisicos, hw.nucleos_logicos, hw.ccds, gpu, vazio(&hw.placa_mae),
            vazio(&hw.bios), hw.ram_total_gb, hw.ram_velocidade_mhz, hw.ram_modo, discos_txt, hw.windows
        )
    }
}

fn vazio(s: &str) -> &str {
    if s.trim().is_empty() { "—" } else { s }
}
