param(
    [string]$Database = "realty_agency",
    [string]$DbUser = "postgres",
    [string]$DbHost = "localhost",
    [string]$BackupDir = "backups"
)

if (-not (Get-Command pg_dump -ErrorAction SilentlyContinue)) {
    throw "Команда pg_dump не найдена. Установите PostgreSQL client tools и добавьте их в PATH."
}

New-Item -ItemType Directory -Force -Path $BackupDir | Out-Null

$timestamp = Get-Date -Format "yyyyMMdd_HHmmss"
$backupPath = Join-Path $BackupDir ("realty_agency_{0}.dump" -f $timestamp)

Write-Host "Создание резервной копии в $backupPath"
pg_dump -Fc -h $DbHost -U $DbUser -d $Database -f $backupPath

if ($LASTEXITCODE -ne 0) {
    throw "Не удалось создать резервную копию базы данных."
}

Write-Host "Резервная копия создана."
