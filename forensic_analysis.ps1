#!/usr/bin/env pwsh
# 🔍 ANÁLISIS FORENSE COMPLETO DEL SISTEMA SNIPERFORGE
# Fecha: Agosto 1, 2025
# Propósito: Detectar duplicaciones, warnings, errores y inconsistencias

Write-Host "🔥 ANÁLISIS FORENSE SISTEMA SNIPERFORGE" -ForegroundColor Red
Write-Host "=======================================" -ForegroundColor Red

# 1. VERIFICAR COMPILACIÓN COMPLETA
Write-Host "`n1️⃣ VERIFICACIÓN DE COMPILACIÓN" -ForegroundColor Yellow
cargo check --all-targets 2>&1 | Tee-Object -FilePath "forensic_analysis_compilation.log"
$compilation_status = $LASTEXITCODE

# 2. ANÁLISIS CLIPPY (WARNINGS Y MEJORES PRÁCTICAS)
Write-Host "`n2️⃣ ANÁLISIS CLIPPY (WARNINGS Y CODE QUALITY)" -ForegroundColor Yellow
cargo clippy --all-targets 2>&1 | Tee-Object -FilePath "forensic_analysis_clippy.log"
$clippy_status = $LASTEXITCODE

# 3. VERIFICACIÓN DE TESTS
Write-Host "`n3️⃣ VERIFICACIÓN DE TESTS" -ForegroundColor Yellow
cargo test --no-run 2>&1 | Tee-Object -FilePath "forensic_analysis_tests.log"
$tests_status = $LASTEXITCODE

# 4. BÚSQUEDA DE DUPLICACIONES
Write-Host "`n4️⃣ BÚSQUEDA DE DUPLICACIONES" -ForegroundColor Yellow

Write-Host "   🔍 Buscando structs duplicadas..."
Get-ChildItem -Path "src" -Recurse -Filter "*.rs" | ForEach-Object {
    $content = Get-Content $_.FullName
    $content | Select-String "pub struct.*Executor|pub struct.*Engine|pub struct.*Manager" | ForEach-Object {
        Write-Host "     📁 $($_.Filename):$($_.LineNumber) - $($_.Line.Trim())" -ForegroundColor Cyan
    }
}

Write-Host "   🔍 Buscando funciones duplicadas..."
Get-ChildItem -Path "src" -Recurse -Filter "*.rs" | ForEach-Object {
    $content = Get-Content $_.FullName
    $content | Select-String "pub async fn execute_.*trade|pub fn get_.*status" | ForEach-Object {
        Write-Host "     📁 $($_.Filename):$($_.LineNumber) - $($_.Line.Trim())" -ForegroundColor Cyan
    }
}

# 5. VERIFICAR REFERENCES A OLD-ROOT-ARCHIVE
Write-Host "`n5️⃣ BÚSQUEDA DE REFERENCIAS A OLD-ROOT-ARCHIVE" -ForegroundColor Yellow
$old_refs = Get-ChildItem -Path "src" -Recurse -Filter "*.rs" | Select-String "old.root.archive|old_root_archive"
if ($old_refs) {
    Write-Host "   ❌ ENCONTRADAS REFERENCIAS:" -ForegroundColor Red
    $old_refs | ForEach-Object { Write-Host "     📁 $($_.Filename):$($_.LineNumber) - $($_.Line.Trim())" -ForegroundColor Red }
} else {
    Write-Host "   ✅ No se encontraron referencias a old-root-archive" -ForegroundColor Green
}

# 6. ANÁLISIS DE TODOs Y FIXMEs
Write-Host "`n6️⃣ ANÁLISIS DE TODOs Y AREAS PENDIENTES" -ForegroundColor Yellow
$todos = Get-ChildItem -Path "src" -Recurse -Filter "*.rs" | Select-String "TODO|FIXME|HACK|XXX"
Write-Host "   📊 Total TODOs encontrados: $($todos.Count)" -ForegroundColor Cyan
$todos | Group-Object { $_.Filename } | ForEach-Object {
    Write-Host "     📁 $($_.Name): $($_.Count) TODOs" -ForegroundColor Cyan
}

# 7. VERIFICAR IMPORTS PROBLEMÁTICOS
Write-Host "`n7️⃣ VERIFICACIÓN DE IMPORTS PROBLEMÁTICOS" -ForegroundColor Yellow
$problematic_imports = Get-ChildItem -Path "src" -Recurse -Filter "*.rs" | Select-String "use.*shared::|use.*crate::shared"
if ($problematic_imports) {
    Write-Host "   ❌ IMPORTS PROBLEMÁTICOS ENCONTRADOS:" -ForegroundColor Red
    $problematic_imports | ForEach-Object { Write-Host "     📁 $($_.Filename):$($_.LineNumber) - $($_.Line.Trim())" -ForegroundColor Red }
} else {
    Write-Host "   ✅ No se encontraron imports problemáticos" -ForegroundColor Green
}

# 8. VERIFICAR ALLOWS Y WARNINGS SUPRIMIDOS
Write-Host "`n8️⃣ ANÁLISIS DE WARNINGS SUPRIMIDOS" -ForegroundColor Yellow
$allows = Get-ChildItem -Path "src" -Recurse -Filter "*.rs" | Select-String "#\[allow\(.*\)\]"
Write-Host "   📊 Total allows encontrados: $($allows.Count)" -ForegroundColor Cyan
$allows | Group-Object { ($_ -split '#\[allow\(')[1] -split '\)\]')[0] } | Sort-Object Count -Descending | ForEach-Object {
    Write-Host "     🚫 $($_.Name): $($_.Count) ocurrencias" -ForegroundColor Yellow
}

# 9. ANÁLISIS DE ARCHIVOS GRANDES
Write-Host "`n9️⃣ ANÁLISIS DE ARCHIVOS GRANDES (>1000 líneas)" -ForegroundColor Yellow
Get-ChildItem -Path "src" -Recurse -Filter "*.rs" | ForEach-Object {
    $lines = (Get-Content $_.FullName).Count
    if ($lines -gt 1000) {
        Write-Host "     📄 $($_.Name): $lines líneas" -ForegroundColor Magenta
    }
}

# 10. RESUMEN FINAL
Write-Host "`n🎯 RESUMEN FINAL DEL ANÁLISIS FORENSE" -ForegroundColor Green
Write-Host "====================================" -ForegroundColor Green

if ($compilation_status -eq 0) {
    Write-Host "✅ COMPILACIÓN: EXITOSA" -ForegroundColor Green
} else {
    Write-Host "❌ COMPILACIÓN: FALLÓ" -ForegroundColor Red
}

if ($clippy_status -eq 0) {
    Write-Host "✅ CLIPPY: SIN PROBLEMAS CRÍTICOS" -ForegroundColor Green
} else {
    Write-Host "⚠️ CLIPPY: WARNINGS ENCONTRADOS" -ForegroundColor Yellow
}

if ($tests_status -eq 0) {
    Write-Host "✅ TESTS: COMPILACIÓN EXITOSA" -ForegroundColor Green
} else {
    Write-Host "❌ TESTS: PROBLEMAS DE COMPILACIÓN" -ForegroundColor Red
}

Write-Host "`n📊 ARCHIVOS DE LOG GENERADOS:" -ForegroundColor Cyan
Write-Host "   📄 forensic_analysis_compilation.log" -ForegroundColor Cyan
Write-Host "   📄 forensic_analysis_clippy.log" -ForegroundColor Cyan
Write-Host "   📄 forensic_analysis_tests.log" -ForegroundColor Cyan

Write-Host "`n🔍 ANÁLISIS FORENSE COMPLETADO" -ForegroundColor Green
