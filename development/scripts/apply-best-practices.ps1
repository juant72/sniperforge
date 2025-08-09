# SniperForge - Aplicación de Mejores Prácticas
# Script para configurar el entorno de desarrollo profesional

Write-Host ""
Write-Host "🚀 SniperForge - Aplicando Mejores Prácticas" -ForegroundColor Cyan
Write-Host "=============================================" -ForegroundColor Cyan
Write-Host ""

# 1. Configurar Rust para mejor rendimiento
function Set-RustOptimizations {
    Write-Host "⚡ Configurando optimizaciones de Rust..." -ForegroundColor Yellow
    
    # Crear .cargo/config.toml si no existe
    if (-not (Test-Path ".cargo")) {
        New-Item -ItemType Directory -Path ".cargo" -Force | Out-Null
    }
    
    $cargoConfig = @"
[build]
# Usar lld linker para builds más rápidos
rustflags = ["-C", "link-arg=-fuse-ld=lld"]

[target.x86_64-pc-windows-msvc]
# Optimizaciones específicas para Windows
rustflags = ["-C", "target-cpu=native"]

[profile.dev]
# Desarrollo más rápido
opt-level = 1
debug = true
incremental = true

[profile.release]
# Máximo rendimiento en producción
opt-level = 3
lto = "fat"
codegen-units = 1
panic = "abort"
strip = true

[net]
# Usar índice sparse para crates.io (más rápido)
git-fetch-with-cli = true
"@
    
    $cargoConfig | Out-File -FilePath ".cargo\config.toml" -Encoding utf8
    Write-Host "   ✅ Configuración de Rust optimizada" -ForegroundColor Green
}

# 2. Configurar herramientas de desarrollo
function Install-DevTools {
    Write-Host "🔧 Instalando herramientas de desarrollo..." -ForegroundColor Yellow
    
    # Verificar si las herramientas están instaladas
    $tools = @(
        @{name="rustfmt"; cmd="cargo fmt --version"},
        @{name="clippy"; cmd="cargo clippy --version"},
        @{name="cargo-watch"; cmd="cargo watch --version"},
        @{name="cargo-audit"; cmd="cargo audit --version"}
    )
    
    foreach ($tool in $tools) {
        try {
            Invoke-Expression $tool.cmd | Out-Null
            Write-Host "   ✅ $($tool.name) ya instalado" -ForegroundColor Green
        }
        catch {
            Write-Host "   📦 Instalando $($tool.name)..." -ForegroundColor Cyan
            if ($tool.name -eq "rustfmt" -or $tool.name -eq "clippy") {
                rustup component add $tool.name
            } else {
                cargo install $tool.name
            }
        }
    }
}

# 3. Configurar linting y formateo
function Set-LintingConfiguration {
    Write-Host "📋 Configurando linting y formateo..." -ForegroundColor Yellow
    
    # Crear rustfmt.toml
    $rustfmtConfig = @"
# SniperForge Rust Formatting Configuration
hard_tabs = false
tab_spaces = 4
max_width = 100
comment_width = 80
wrap_comments = true
format_code_in_doc_comments = true
normalize_comments = true
license_template_path = ""
format_strings = true
format_macro_matchers = true
format_macro_bodies = true
empty_item_single_line = true
struct_lit_single_line = true
fn_single_line = false
where_single_line = false
imports_indent = "Block"
imports_layout = "Mixed"
merge_imports = true
reorder_imports = true
reorder_modules = true
reorder_impl_items = false
type_punctuation_density = "Wide"
space_before_colon = false
space_after_colon = true
spaces_around_ranges = false
binop_separator = "Front"
remove_nested_parens = true
combine_control_expr = true
overflow_delimited_expr = false
struct_field_align_threshold = 0
enum_discrim_align_threshold = 0
match_arm_blocks = true
force_multiline_blocks = false
fn_args_layout = "Tall"
brace_style = "SameLineWhere"
control_brace_style = "AlwaysSameLine"
trailing_semicolon = true
trailing_comma = "Vertical"
match_block_trailing_comma = false
blank_lines_upper_bound = 1
blank_lines_lower_bound = 0
edition = "2021"
version = "Two"
inline_attribute_width = 0
merge_derives = true
use_try_shorthand = false
use_field_init_shorthand = false
force_explicit_abi = true
condense_wildcard_suffixes = false
color = "Auto"
required_version = "1.5.1"
unstable_features = false
disable_all_formatting = false
skip_children = false
hide_parse_errors = false
error_on_line_overflow = false
error_on_unformatted = false
report_todo = "Never"
report_fixme = "Never"
ignore = []
emit_mode = "Files"
make_backup = false
"@
    
    $rustfmtConfig | Out-File -FilePath "rustfmt.toml" -Encoding utf8
    
    # Crear clippy.toml
    $clippyConfig = @"
# SniperForge Clippy Configuration
cognitive-complexity-threshold = 30
type-complexity-threshold = 250
too-many-arguments-threshold = 8
trivial-copy-size-limit = 64
pass-by-value-size-limit = 256
semicolon-if-nothing-returned = true
doc-markdown = true
manual-let-else = true
manual-ok-or = true
manual-string-new = true
redundant-clone = true
redundant-closure-for-method-calls = true
single-match-else = true
unnecessary-wraps = true
used-underscore-binding = true
wildcard-imports = true
"@
    
    $clippyConfig | Out-File -FilePath "clippy.toml" -Encoding utf8
    
    Write-Host "   ✅ Configuración de linting aplicada" -ForegroundColor Green
}

# 4. Crear scripts de desarrollo
function Create-DevScripts {
    Write-Host "📜 Creando scripts de desarrollo..." -ForegroundColor Yellow
    
    # Script de desarrollo rápido
    $devScript = @"
# Script de desarrollo rápido para SniperForge
Write-Host "🔄 Iniciando desarrollo en modo watch..." -ForegroundColor Cyan
cargo watch -x "check --all-targets" -x "test" -x "run --bin arbitrage-basic"
"@
    $devScript | Out-File -FilePath "dev.ps1" -Encoding utf8
    
    # Script de testing
    $testScript = @"
# Script de testing completo para SniperForge
Write-Host "🧪 Ejecutando suite de tests completa..." -ForegroundColor Cyan
cargo test --all-targets --all-features
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt -- --check
cargo audit
"@
    $testScript | Out-File -FilePath "test.ps1" -Encoding utf8
    
    # Script de build de producción
    $buildScript = @"
# Script de build de producción para SniperForge
Write-Host "🏗️ Construyendo versión de producción..." -ForegroundColor Cyan
cargo clean
cargo build --release
Write-Host "✅ Build de producción completado" -ForegroundColor Green
Write-Host "📦 Binario disponible en: target\release\arbitrage-basic.exe" -ForegroundColor Cyan
"@
    $buildScript | Out-File -FilePath "build-production.ps1" -Encoding utf8
    
    Write-Host "   ✅ Scripts de desarrollo creados" -ForegroundColor Green
}

# 5. Configurar estructura de directorios profesional
function Set-ProjectStructure {
    Write-Host "📁 Configurando estructura de proyecto..." -ForegroundColor Yellow
    
    $directories = @(
        "docs\api",
        "docs\guides", 
        "docs\architecture",
        "examples\basic",
        "examples\advanced",
        "benchmarks",
        "tools"
    )
    
    foreach ($dir in $directories) {
        if (-not (Test-Path $dir)) {
            New-Item -ItemType Directory -Path $dir -Force | Out-Null
            Write-Host "   📁 Creado: $dir" -ForegroundColor Gray
        }
    }
    
    # Crear .gitignore mejorado
    $gitignore = @"
# Rust
/target/
Cargo.lock
**/*.rs.bk
*.pdb

# IDE
.vscode/
.idea/
*.swp
*.swo
*~

# OS
.DS_Store
Thumbs.db

# Logs
*.log
logs/

# Environment variables
.env.local
.env.*.local

# Wallet files (IMPORTANTE: no commitear wallets reales)
**/wallet-real.json
**/keypair.json
**/*private*
**/*secret*

# Trading data
trading_data/
performance_logs/
backtest_results/

# Temporary files
temp/
tmp/
*.tmp

# Coverage reports
tarpaulin-report.html
lcov.info

# Benchmark results
criterion/

# Documentation build
book/
"@
    $gitignore | Out-File -FilePath ".gitignore" -Encoding utf8
    
    Write-Host "   ✅ Estructura de proyecto configurada" -ForegroundColor Green
}

# 6. Crear documentación básica
function Create-Documentation {
    Write-Host "📚 Creando documentación básica..." -ForegroundColor Yellow
    
    # README mejorado
    $readme = @"
# SniperForge 🎯

Professional Solana DeFi Trading Bot Suite

## 🚀 Quick Start

\`\`\`bash
# Desarrollo
./dev.ps1

# Testing
./test.ps1

# Producción
./build-production.ps1
cargo run --release --bin arbitrage-basic
\`\`\`

## 🏗️ Architecture

- **Core**: Shared trading engine and utilities
- **Bots**: Individual trading strategies
- **Tools**: Development and configuration utilities

## 🔧 Development

- **Language**: Rust 2021 Edition
- **Runtime**: Tokio async runtime
- **Blockchain**: Solana mainnet/devnet
- **APIs**: Jupiter v6, DexScreener, CoinGecko

## 📊 Features

- ✅ Real-time arbitrage detection
- ✅ Multi-DEX support (Jupiter, Orca, Raydium)
- ✅ Flash loan integration
- ✅ Cross-chain capabilities
- ✅ Enterprise-grade risk management
- ✅ AI/ML optimization
- ✅ Real-time performance analytics

## ⚠️ Security

- Never commit private keys or wallet files
- Use environment variables for sensitive data
- Test on devnet before mainnet deployment
- Monitor positions and set stop-losses

## 📄 License

MIT License - see LICENSE file for details
"@
    $readme | Out-File -FilePath "README.md" -Encoding utf8
    
    Write-Host "   ✅ Documentación básica creada" -ForegroundColor Green
}

# Ejecutar todas las configuraciones
try {
    Set-RustOptimizations
    Install-DevTools
    Set-LintingConfiguration
    Create-DevScripts
    Set-ProjectStructure
    Create-Documentation
    
    Write-Host ""
    Write-Host "🎉 ¡Mejores prácticas aplicadas exitosamente!" -ForegroundColor Green
    Write-Host ""
    Write-Host "📋 Próximos pasos:" -ForegroundColor Cyan
    Write-Host "   1. ./dev.ps1          - Desarrollo en modo watch" -ForegroundColor Gray
    Write-Host "   2. ./test.ps1         - Ejecutar tests completos" -ForegroundColor Gray
    Write-Host "   3. ./build-production.ps1 - Build de producción" -ForegroundColor Gray
    Write-Host ""
    Write-Host "🚀 ¡Listo para trading profesional!" -ForegroundColor Green
    Write-Host ""
    
} catch {
    Write-Host ""
    Write-Host "❌ Error aplicando mejores prácticas: $($_.Exception.Message)" -ForegroundColor Red
    Write-Host ""
}
