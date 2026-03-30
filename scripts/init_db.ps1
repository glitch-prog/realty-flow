param(
    [string]$Database = "realty_agency",
    [string]$DbUser = "postgres",
    [string]$DbPassword = "",
    [string]$DbHost = "localhost",
    [int]$DbPort = 5432
)

$ErrorActionPreference = "Stop"

if (-not (Get-Command psql -ErrorAction SilentlyContinue)) {
    throw "Команда psql не найдена. Установите PostgreSQL client tools и добавьте их в PATH."
}

if ($DbPassword) {
    $env:PGPASSWORD = $DbPassword
}

$previousClientEncoding = $env:PGCLIENTENCODING
$env:PGCLIENTENCODING = "UTF8"

try {
    $dbExists = & psql -h $DbHost -p $DbPort -U $DbUser -tAc "SELECT 1 FROM pg_database WHERE datname = '$Database'" postgres
    if ($LASTEXITCODE -ne 0) {
        throw "Не удалось проверить наличие базы данных $Database."
    }

    if (($dbExists | Out-String).Trim() -ne "1") {
        Write-Host "Создание базы данных $Database"
        & psql -h $DbHost -p $DbPort -U $DbUser -c "CREATE DATABASE $Database" postgres
        if ($LASTEXITCODE -ne 0) {
            throw "Не удалось создать базу данных $Database."
        }
    }
    else {
        Write-Host "База данных $Database уже существует"
    }

    $sqlFiles = @(
        "sql/01_schema.sql",
        "sql/02_seed.sql",
        "sql/03_views_and_automation.sql",
        "sql/04_roles.sql"
    )

    $projectRoot = Split-Path -Parent $PSScriptRoot
    foreach ($sqlFile in $sqlFiles) {
        $fullPath = Join-Path $projectRoot $sqlFile
        Write-Host "Применение $sqlFile"
        & psql -h $DbHost -p $DbPort -U $DbUser -d $Database -f $fullPath
        if ($LASTEXITCODE -ne 0) {
            throw "Ошибка выполнения $sqlFile"
        }
    }

    Write-Host "База данных инициализирована" -ForegroundColor Green
}
finally {
    if ($DbPassword) {
        Remove-Item Env:PGPASSWORD -ErrorAction SilentlyContinue
    }

    if ($null -eq $previousClientEncoding) {
        Remove-Item Env:PGCLIENTENCODING -ErrorAction SilentlyContinue
    }
    else {
        $env:PGCLIENTENCODING = $previousClientEncoding
    }
}
