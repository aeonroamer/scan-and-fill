$ErrorActionPreference = "Stop"
Set-Location $PSScriptRoot

cargo build --workspace
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

$server = Start-Process -FilePath ".\target\debug\scanfill-server.exe" -NoNewWindow -PassThru
try {
    & ".\target\debug\scanfill.exe"
}
finally {
    Stop-Process -Id $server.Id -ErrorAction SilentlyContinue
}
