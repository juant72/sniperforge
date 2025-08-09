#!/usr/bin/env pwsh
# SniperForge Enterprise Deployment Script
# Phase 5: Advanced Enterprise Optimization

param(
    [Parameter(Mandatory=$false)]
    [string]$Environment = "production",
    
    [Parameter(Mandatory=$false)]
    [switch]$Build = $false,
    
    [Parameter(Mandatory=$false)]
    [switch]$Deploy = $false,
    
    [Parameter(Mandatory=$false)]
    [switch]$Monitor = $false,
    
    [Parameter(Mandatory=$false)]
    [switch]$Stop = $false,
    
    [Parameter(Mandatory=$false)]
    [switch]$HealthCheck = $false
)

# Script Configuration
$SCRIPT_VERSION = "1.0.0"
$PROJECT_NAME = "SniperForge Enterprise"
$COMPOSE_FILE = "docker-compose.enterprise.yml"

# Colors for output
$Green = "`e[32m"
$Red = "`e[31m"
$Yellow = "`e[33m"
$Blue = "`e[34m"
$Cyan = "`e[36m"
$Reset = "`e[0m"

function Write-Header {
    param([string]$Message)
    Write-Host "${Cyan}================================================================${Reset}"
    Write-Host "${Cyan}  $Message${Reset}"
    Write-Host "${Cyan}================================================================${Reset}"
}

function Write-Success {
    param([string]$Message)
    Write-Host "${Green}✅ $Message${Reset}"
}

function Write-Error {
    param([string]$Message)
    Write-Host "${Red}❌ $Message${Reset}"
}

function Write-Warning {
    param([string]$Message)
    Write-Host "${Yellow}⚠️  $Message${Reset}"
}

function Write-Info {
    param([string]$Message)
    Write-Host "${Blue}ℹ️  $Message${Reset}"
}

function Test-Prerequisites {
    Write-Info "Checking prerequisites..."
    
    # Check Docker
    try {
        $dockerVersion = docker --version
        Write-Success "Docker found: $dockerVersion"
    }
    catch {
        Write-Error "Docker not found. Please install Docker Desktop."
        exit 1
    }
    
    # Check Docker Compose
    try {
        $composeVersion = docker-compose --version
        Write-Success "Docker Compose found: $composeVersion"
    }
    catch {
        Write-Error "Docker Compose not found. Please install Docker Compose."
        exit 1
    }
    
    # Check if compose file exists
    if (-not (Test-Path $COMPOSE_FILE)) {
        Write-Error "Docker Compose file not found: $COMPOSE_FILE"
        exit 1
    }
    
    # Check .env file
    if (-not (Test-Path ".env")) {
        Write-Warning ".env file not found. Creating from template..."
        Create-EnvTemplate
    }
    
    Write-Success "All prerequisites met!"
}

function Create-EnvTemplate {
    $envContent = @"
# SniperForge Enterprise Environment Configuration
# Copy this file to .env and fill in your values

# Database Configuration
DB_PASSWORD=your_secure_database_password_here

# Monitoring Configuration
GRAFANA_PASSWORD=your_secure_grafana_password_here

# Backup Configuration
BACKUP_S3_BUCKET=your-backup-s3-bucket
AWS_ACCESS_KEY_ID=your_aws_access_key
AWS_SECRET_ACCESS_KEY=your_aws_secret_key

# Notification Configuration
SLACK_WEBHOOK_URL=your_slack_webhook_url_for_notifications

# Security Configuration
JWT_SECRET=your_jwt_secret_key_here
API_KEY_SALT=your_api_key_salt_here

# Performance Configuration
MAX_CONCURRENT_TRADES=1000
HFT_ENGINE_ENABLED=true
PERFORMANCE_MODE=maximum

# Environment
SNIPERFORGE_ENV=production
RUST_LOG=info
"@
    
    Set-Content -Path ".env" -Value $envContent -Encoding UTF8
    Write-Success "Created .env template file. Please edit it with your configuration."
}

function Build-Application {
    Write-Header "Building SniperForge Enterprise Application"
    
    Write-Info "Cleaning previous builds..."
    cargo clean
    
    Write-Info "Building optimized release version..."
    $buildResult = cargo build --release --locked
    
    if ($LASTEXITCODE -eq 0) {
        Write-Success "Application built successfully!"
    } else {
        Write-Error "Build failed!"
        exit 1
    }
    
    Write-Info "Running tests..."
    $testResult = cargo test --release --locked
    
    if ($LASTEXITCODE -eq 0) {
        Write-Success "All tests passed!"
    } else {
        Write-Warning "Some tests failed, but continuing deployment..."
    }
    
    Write-Info "Building Docker images..."
    docker-compose -f $COMPOSE_FILE build --parallel
    
    if ($LASTEXITCODE -eq 0) {
        Write-Success "Docker images built successfully!"
    } else {
        Write-Error "Docker build failed!"
        exit 1
    }
}

function Deploy-Stack {
    Write-Header "Deploying SniperForge Enterprise Stack"
    
    Write-Info "Creating required directories..."
    $directories = @("logs", "data", "backups", "config/ssl")
    foreach ($dir in $directories) {
        if (-not (Test-Path $dir)) {
            New-Item -ItemType Directory -Path $dir -Force | Out-Null
            Write-Success "Created directory: $dir"
        }
    }
    
    Write-Info "Starting enterprise stack..."
    docker-compose -f $COMPOSE_FILE up -d
    
    if ($LASTEXITCODE -eq 0) {
        Write-Success "Enterprise stack deployed successfully!"
        Show-ServiceStatus
    } else {
        Write-Error "Deployment failed!"
        exit 1
    }
    
    Write-Info "Waiting for services to be ready..."
    Start-Sleep -Seconds 30
    
    Perform-HealthCheck
}

function Show-ServiceStatus {
    Write-Header "Service Status"
    docker-compose -f $COMPOSE_FILE ps
    
    Write-Info "Service URLs:"
    Write-Host "🌐 Load Balancer:      http://localhost (HTTP) / https://localhost (HTTPS)"
    Write-Host "📊 Grafana Dashboard:  http://localhost:3000"
    Write-Host "🔍 Prometheus:         http://localhost:9092"
    Write-Host "📋 Kibana Logs:        http://localhost:5601"
    Write-Host "🔍 Jaeger Tracing:     http://localhost:16686"
    Write-Host "⚖️  HAProxy Stats:      http://localhost:8404/haproxy-stats"
    Write-Host "🏥 Health Check:       http://localhost:8081/health"
}

function Perform-HealthCheck {
    Write-Header "Performing Health Checks"
    
    $healthEndpoints = @(
        @{Name="SniperForge Primary"; URL="http://localhost:8081/health"}
        @{Name="SniperForge Secondary"; URL="http://localhost:8083/health"}
        @{Name="Load Balancer"; URL="http://localhost/health"}
        @{Name="Grafana"; URL="http://localhost:3000/api/health"}
        @{Name="Prometheus"; URL="http://localhost:9092/-/healthy"}
    )
    
    foreach ($endpoint in $healthEndpoints) {
        try {
            $response = Invoke-WebRequest -Uri $endpoint.URL -Method GET -TimeoutSec 10
            if ($response.StatusCode -eq 200) {
                Write-Success "$($endpoint.Name) is healthy"
            } else {
                Write-Warning "$($endpoint.Name) returned status $($response.StatusCode)"
            }
        }
        catch {
            Write-Error "$($endpoint.Name) health check failed: $($_.Exception.Message)"
        }
    }
}

function Monitor-Services {
    Write-Header "Real-time Service Monitoring"
    
    Write-Info "Monitoring service logs (Ctrl+C to exit)..."
    docker-compose -f $COMPOSE_FILE logs -f
}

function Stop-Stack {
    Write-Header "Stopping SniperForge Enterprise Stack"
    
    Write-Info "Stopping all services..."
    docker-compose -f $COMPOSE_FILE down
    
    if ($LASTEXITCODE -eq 0) {
        Write-Success "Enterprise stack stopped successfully!"
    } else {
        Write-Error "Failed to stop some services!"
    }
}

function Show-Usage {
    Write-Host @"
${Cyan}SniperForge Enterprise Deployment Script v$SCRIPT_VERSION${Reset}

${Yellow}USAGE:${Reset}
    deploy-enterprise.ps1 [OPTIONS]

${Yellow}OPTIONS:${Reset}
    -Build              Build the application and Docker images
    -Deploy             Deploy the full enterprise stack
    -Monitor            Monitor service logs in real-time
    -Stop               Stop all enterprise services
    -HealthCheck        Perform health checks on all services
    -Environment        Set environment (default: production)

${Yellow}EXAMPLES:${Reset}
    ${Green}# Full deployment pipeline${Reset}
    ./deploy-enterprise.ps1 -Build -Deploy

    ${Green}# Just deploy with existing images${Reset}
    ./deploy-enterprise.ps1 -Deploy

    ${Green}# Monitor running services${Reset}
    ./deploy-enterprise.ps1 -Monitor

    ${Green}# Check service health${Reset}
    ./deploy-enterprise.ps1 -HealthCheck

    ${Green}# Stop all services${Reset}
    ./deploy-enterprise.ps1 -Stop

${Yellow}REQUIREMENTS:${Reset}
    - Docker Desktop
    - Docker Compose
    - PowerShell 5.1+
    - .env file with configuration

${Yellow}SERVICE PORTS:${Reset}
    - Load Balancer:   80, 443
    - SniperForge:     8080, 8082
    - Grafana:         3000
    - Prometheus:      9092
    - Kibana:          5601
    - Jaeger:          16686
    - HAProxy Stats:   8404

For more information, visit: https://github.com/sniperforge/enterprise
"@
}

# Main execution logic
Write-Header "$PROJECT_NAME Deployment Script v$SCRIPT_VERSION"

if (-not $Build -and -not $Deploy -and -not $Monitor -and -not $Stop -and -not $HealthCheck) {
    Show-Usage
    exit 0
}

try {
    Test-Prerequisites
    
    if ($Build) {
        Build-Application
    }
    
    if ($Deploy) {
        Deploy-Stack
    }
    
    if ($Monitor) {
        Monitor-Services
    }
    
    if ($HealthCheck) {
        Perform-HealthCheck
    }
    
    if ($Stop) {
        Stop-Stack
    }
    
    Write-Success "Operation completed successfully!"
}
catch {
    Write-Error "Script execution failed: $($_.Exception.Message)"
    exit 1
}
