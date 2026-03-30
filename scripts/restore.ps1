param(
    [Parameter(Mandatory = $true)]
    [string]$BackupFile,
    [string]$Database = "realty_agency",
    [string]$DbUser = "postgres",
    [string]$DbHost = "localhost"
)

if (-not (Test-Path $BackupFile)) {
    throw "Файл резервной копии не найден: $BackupFile"
}

if (-not (Get-Command pg_restore -ErrorAction SilentlyContinue)) {
    throw "Команда pg_restore не найдена. Установите PostgreSQL client tools и добавьте их в PATH."
}

Write-Host "Восстановление базы $Database из $BackupFile"
pg_restore -c -h $DbHost -U $DbUser -d $Database $BackupFile

if ($LASTEXITCODE -ne 0) {
    throw "Не удалось восстановить базу данных из резервной копии."
}

Write-Host "Восстановление завершено."
