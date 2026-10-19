\xEF\xBB\xBF<#
  Monta a versão PORTÁTIL do Xablau Optimizer (.zip), com o WebView2 embutido (fixed runtime),
  para funcionar em qualquer Windows 10/11 — inclusive sem WebView2 instalado.
  Roda no GitHub Actions depois do "tauri build". Também gera o SHA256SUMS.txt.
#>
param([string]$Saida = 'dist-portatil')
$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'

$raiz = Split-Path -Parent $PSScriptRoot
$versao = (Get-Content (Join-Path $raiz 'package.json') -Raw | ConvertFrom-Json).version
$tauri = Join-Path $raiz 'src-tauri'
$release = Join-Path $tauri 'target\release'
$binarios = Join-Path $tauri 'binaries'
$saidaAbs = Join-Path $raiz $Saida
$nome = "XablauOptimizer-$versao-portatil"
$pasta = Join-Path $saidaAbs $nome

if (Test-Path $pasta) { Remove-Item $pasta -Recurse -Force }
New-Item -ItemType Directory -Force -Path $pasta | Out-Null

# 1) Executáveis: interface + vigia + kit de emergência + PresentMon
$principal = Join-Path $release 'xablau-optimizer.exe'
if (-not (Test-Path $principal)) { throw "Não encontrei $principal. Rode o 'tauri build' antes." }
Copy-Item $principal $pasta
foreach ($b in 'xablau-vigia', 'xablau-restaurar', 'presentmon') {
  $arq = Get-ChildItem $binarios -Filter "$b-*.exe" | Select-Object -First 1
  if (-not $arq) { throw "Binário auxiliar ausente: $b (rode scripts/preparar-binarios.mjs)." }
  Copy-Item $arq.FullName (Join-Path $pasta "$b.exe")
}

# 2) Licenças
Copy-Item (Join-Path $tauri 'licencas') (Join-Path $pasta 'licencas') -Recurse

# 3) WebView2 (fixed runtime): copia a versão instalada no computador de compilação; se não houver,
#    instala o pacote oficial da Microsoft (assinatura conferida) e copia em seguida.
function Get-PastaWebView2 {
  $bases = @("${env:ProgramFiles(x86)}\Microsoft\EdgeWebView\Application", "$env:ProgramFiles\Microsoft\EdgeWebView\Application")
  foreach ($b in $bases) {
    if (Test-Path $b) {
      $v = Get-ChildItem $b -Directory |
        Where-Object { $_.Name -match '^\d+\.\d+\.\d+\.\d+$' -and (Test-Path (Join-Path $_.FullName 'msedgewebview2.exe')) } |
        Sort-Object { [version]$_.Name } -Descending | Select-Object -First 1
      if ($v) { return $v }
    }
  }
  return $null
}
$wv = Get-PastaWebView2
if (-not $wv) {
  Write-Host 'WebView2 não encontrado; instalando o pacote oficial (Evergreen Standalone x64)...'
  $temp = if ($env:RUNNER_TEMP) { $env:RUNNER_TEMP } else { $env:TEMP }
  $instalador = Join-Path $temp 'MicrosoftEdgeWebView2RuntimeInstallerX64.exe'
  Invoke-WebRequest 'https://go.microsoft.com/fwlink/?linkid=2124701' -OutFile $instalador -UseBasicParsing
  $assinatura = Get-AuthenticodeSignature $instalador
  if ($assinatura.Status -ne 'Valid' -or $assinatura.SignerCertificate.Subject -notmatch 'Microsoft Corporation') {
    throw 'A assinatura do instalador do WebView2 não é válida. Compilação interrompida por segurança.'
  }
  Start-Process $instalador -ArgumentList '/silent', '/install' -Wait
  $wv = Get-PastaWebView2
  if (-not $wv) { throw 'Não foi possível obter o WebView2.' }
}
Write-Host "WebView2 $($wv.Name) embutido na versão portátil."
Copy-Item $wv.FullName (Join-Path $pasta 'WebView2Runtime') -Recurse

# 4) Marcador do modo portátil + instruções
$utf8 = New-Object System.Text.UTF8Encoding $true
[IO.File]::WriteAllLines((Join-Path $pasta 'portatil.txt'), @(
  'Este arquivo liga o MODO PORTÁTIL do Xablau Optimizer. Não apague.',
  '',
  'No modo portátil o programa não inicia com o Windows e não se atualiza sozinho (só avisa e abre a página da nova versão).',
  'Os backups e o diário de alterações ficam no PC otimizado (C:\ProgramData\XablauOptimizer), nunca no pendrive.',
  'O kit de emergência também é copiado para o PC otimizado.'
), $utf8)
[IO.File]::WriteAllLines((Join-Path $pasta 'LEIA-ME.txt'), @(
  "Xablau Optimizer $versao — versão portátil",
  'Criado por Xablau Studios — Luiz',
  '',
  '1. Abra o arquivo xablau-optimizer.exe e aceite o pedido de administrador.',
  '2. Se aparecer "O Windows protegeu o computador", clique em "Mais informações" e depois em "Executar assim mesmo".',
  '3. Em caso de problema, abra xablau-restaurar.exe (kit de emergência) para desfazer as alterações.',
  '',
  'Pasta WebView2Runtime: motor da interface (Microsoft Edge WebView2), embutido para funcionar em qualquer Windows 10/11.'
), $utf8)

# 5) Compactar (a pasta vai inteira para dentro do .zip)
$zip = Join-Path $saidaAbs "$nome.zip"
if (Test-Path $zip) { Remove-Item $zip -Force }
Compress-Archive -Path $pasta -DestinationPath $zip -CompressionLevel Optimal

# 6) SHA-256 de tudo o que vai para a Release
$arquivos = @()
$arquivos += Get-ChildItem (Join-Path $release 'bundle\nsis') -Filter '*.exe' -ErrorAction SilentlyContinue
$arquivos += Get-ChildItem (Join-Path $release 'bundle\msi') -Filter '*.msi' -ErrorAction SilentlyContinue
$arquivos += Get-Item $zip
$linhas = $arquivos | ForEach-Object { '{0}  {1}' -f (Get-FileHash $_.FullName -Algorithm SHA256).Hash.ToLower(), $_.Name }
Set-Content -Path (Join-Path $saidaAbs 'SHA256SUMS.txt') -Value $linhas -Encoding ASCII
Write-Host "Versão portátil pronta: $zip"
