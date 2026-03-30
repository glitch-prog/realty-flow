param(
    [string]$Database = "realty_agency",
    [string]$DbUser = "postgres",
    [string]$DbPassword = "postgres",
    [string]$DbHost = "localhost",
    [int]$DbPort = 5432,
    [string]$AppName = "RealtyFlow",
    [string]$AppHost = "127.0.0.1",
    [int]$AppPort = 8080,
    [switch]$InitDatabase
)

$ErrorActionPreference = "Stop"

function Test-CommandExists([string]$Name) {
    return $null -ne (Get-Command $Name -ErrorAction SilentlyContinue)
}

function Write-Step([string]$Message) {
    Write-Host "[setup] $Message" -ForegroundColor Cyan
}

function Get-EnvValue([string]$Path, [string]$Key) {
    if (-not (Test-Path $Path)) {
        return $null
    }

    $line = Get-Content $Path | Where-Object { $_ -match "^${Key}=" } | Select-Object -First 1
    if ($null -eq $line) {
        return $null
    }

    return $line.Substring($Key.Length + 1).Trim().Trim('"')
}

$projectRoot = Split-Path -Parent $PSScriptRoot
$envExamplePath = Join-Path $projectRoot ".env.example"
$envPath = Join-Path $projectRoot ".env"

Write-Step "Проверка локального файла окружения"
if (-not (Test-Path $envPath)) {
    if (Test-Path $envExamplePath) {
        Copy-Item $envExamplePath $envPath
        Write-Host "Создан файл .env на основе .env.example" -ForegroundColor Green
    }
    else {
        @"
APP_NAME=$AppName
APP_HOST=$AppHost
APP_PORT=$AppPort
DATABASE_URL=postgres://${DbUser}:${DbPassword}@${DbHost}:${DbPort}/${Database}
"@ | Set-Content -Encoding UTF8 $envPath
        Write-Host "Создан файл .env со значениями по умолчанию" -ForegroundColor Green
    }
}
else {
    Write-Host "Файл .env уже существует" -ForegroundColor Green
}

Write-Step "Проверка внешних инструментов"
$hasCargo = Test-CommandExists cargo
$hasPsql = Test-CommandExists psql
$hasPgDump = Test-CommandExists pg_dump
$hasPgRestore = Test-CommandExists pg_restore

Write-Host ("cargo:     {0}" -f ($(if ($hasCargo) { 'OK' } else { 'missing' })))
Write-Host ("psql:      {0}" -f ($(if ($hasPsql) { 'OK' } else { 'missing' })))
Write-Host ("pg_dump:   {0}" -f ($(if ($hasPgDump) { 'OK' } else { 'missing' })))
Write-Host ("pg_restore:{0}" -f ($(if ($hasPgRestore) { 'OK' } else { 'missing' })))

Write-Step "Проверка доступности PostgreSQL"
try {
    $dbPortCheck = Test-NetConnection -ComputerName $DbHost -Port $DbPort -WarningAction SilentlyContinue
    if ($dbPortCheck.TcpTestSucceeded) {
        Write-Host ("PostgreSQL: TCP {0}:{1} доступен" -f $DbHost, $DbPort) -ForegroundColor Green
    }
    else {
        Write-Host ("PostgreSQL: TCP {0}:{1} недоступен" -f $DbHost, $DbPort) -ForegroundColor Yellow
        Write-Host "Проверьте, что служба PostgreSQL запущена и слушает нужный порт." -ForegroundColor Yellow
    }
}
catch {
    Write-Host "Не удалось выполнить сетевую проверку PostgreSQL." -ForegroundColor Yellow
}

$databaseUrl = Get-EnvValue -Path $envPath -Key "DATABASE_URL"
if ($hasPsql -and $databaseUrl) {
    Write-Step "Проверка аутентификации по DATABASE_URL"
    $stdoutFile = [System.IO.Path]::GetTempFileName()
    $stderrFile = [System.IO.Path]::GetTempFileName()

    try {
        $process = Start-Process -FilePath "psql" -ArgumentList @($databaseUrl, "-Atqc", "SELECT 1") -Wait -NoNewWindow -PassThru -RedirectStandardOutput $stdoutFile -RedirectStandardError $stderrFile
        if ($process.ExitCode -eq 0) {
            Write-Host "PostgreSQL: учетные данные из .env корректны" -ForegroundColor Green
        }
        else {
            Write-Host "PostgreSQL: не удалось войти по DATABASE_URL из .env" -ForegroundColor Yellow
            Write-Host "Проверьте логин, пароль и имя базы данных в .env." -ForegroundColor Yellow
        }
    }
    finally {
        Remove-Item $stdoutFile, $stderrFile -ErrorAction SilentlyContinue
    }
}

if ($InitDatabase) {
    if (-not $hasPsql) {
        throw "Невозможно инициализировать БД: команда psql не найдена в PATH."
    }

    $previousClientEncoding = $env:PGCLIENTENCODING
    $env:PGCLIENTENCODING = "UTF8"

    try {
        Write-Step "Создание базы данных при необходимости"
        $dbExists = & psql -h $DbHost -p $DbPort -U $DbUser -tAc "SELECT 1 FROM pg_database WHERE datname = '$Database'" postgres
        if ($LASTEXITCODE -ne 0) {
            throw "Не удалось проверить наличие базы данных $Database."
        }

        if (($dbExists | Out-String).Trim() -ne "1") {
            & psql -h $DbHost -p $DbPort -U $DbUser -c "CREATE DATABASE $Database" postgres
            if ($LASTEXITCODE -ne 0) {
                throw "Не удалось создать базу данных $Database."
            }
            Write-Host "База данных $Database создана" -ForegroundColor Green
        }
        else {
            Write-Host "База данных $Database уже существует" -ForegroundColor Green
        }

        Write-Step "Применение SQL-скриптов"
        $sqlFiles = @(
            "sql/01_schema.sql",
            "sql/02_seed.sql",
            "sql/03_views_and_automation.sql",
            "sql/04_roles.sql"
        )

        foreach ($sqlFile in $sqlFiles) {
            $fullPath = Join-Path $projectRoot $sqlFile
            Write-Host " -> $sqlFile"
            & psql -h $DbHost -p $DbPort -U $DbUser -d $Database -f $fullPath
            if ($LASTEXITCODE -ne 0) {
                throw "Ошибка выполнения $sqlFile"
            }
        }

        Write-Host "Инициализация базы данных завершена" -ForegroundColor Green
    }
    finally {
        if ($null -eq $previousClientEncoding) {
            Remove-Item Env:PGCLIENTENCODING -ErrorAction SilentlyContinue
        }
        else {
            $env:PGCLIENTENCODING = $previousClientEncoding
        }
    }
}

Write-Step "Что дальше"
if (-not $hasCargo) {
    Write-Host "1. Установить Rust / cargo и открыть новую сессию терминала." -ForegroundColor Yellow
}
else {
    Write-Host "1. Выполнить cargo run" -ForegroundColor Green
}

if (-not $hasPsql) {
    Write-Host "2. Установить PostgreSQL client tools или добавить psql в PATH." -ForegroundColor Yellow
}
else {
    Write-Host "2. Убедиться, что DATABASE_URL в .env содержит правильный пароль и имя базы." -ForegroundColor Green
}
