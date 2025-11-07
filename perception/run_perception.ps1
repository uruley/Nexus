param(
    [string]$Host = "127.0.0.1",
    [int]$Port = 5055
)

$venvActivate = Join-Path -Path (Resolve-Path .) -ChildPath ".venv\Scripts\Activate.ps1"
if (Test-Path $venvActivate) {
    . $venvActivate
}

uvicorn server:app --host $Host --port $Port --reload

