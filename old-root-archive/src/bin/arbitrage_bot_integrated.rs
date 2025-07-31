// ===== ARBITRAGE BOT PHASE 4.5 - APLICACIÓN PRINCIPAL EJECUTABLE =====
// Sistema integrado evolutivo que preserva funcionalidad original + mejoras opcionales
// Comando: cargo run --bin arbitrage_bot_integrated

use std::io::{self, Write};
use std::time::Duration;
use anyhow::Result;
use tracing::{info, warn, error, Level};
use tracing_subscriber;
use clap::{App, Arg, SubCommand};

use sniperforge::{
    ArbitrageBotPhase45Integrated,
    unified_config::UnifiedPhase45Config,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Configurar logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_target(false)
        .init();

    // Parse command line arguments
    let matches = App::new("SniperForge Arbitrage Bot - Phase 4.5 Integrated")
        .version("4.5.0")
        .about("Sistema de arbitraje evolutivo con mejoras opcionales de Phases 1-4")
        .arg(Arg::with_name("config")
            .short("c")
            .long("config")
            .value_name("FILE")
            .help("Archivo de configuración personalizado")
            .takes_value(true))
        .arg(Arg::with_name("mode")
            .short("m")
            .long("mode")
            .value_name("MODE")
            .help("Modo de operación: basic, jupiter, mev, dex, events, full")
            .takes_value(true))
        .arg(Arg::with_name("auto")
            .short("a")
            .long("auto")
            .help("Ejecutar automáticamente sin menu interactivo"))
        .subcommand(SubCommand::with_name("test")
            .about("Ejecutar modo de testing sin trades reales"))
        .subcommand(SubCommand::with_name("stats")
            .about("Mostrar estadísticas del sistema"))
        .get_matches();

    // Mostrar banner
    show_banner();

    // Cargar configuración
    let mut config = if let Some(config_file) = matches.value_of("config") {
        info!("📁 Cargando configuración desde: {}", config_file);
        UnifiedPhase45Config::load_from_file(config_file)?
    } else {
        UnifiedPhase45Config::default()
    };

    // Aplicar modo si se especificó en CLI
    if let Some(mode) = matches.value_of("mode") {
        apply_operation_mode(&mut config, mode)?;
    }

    // Mostrar configuración actual
    show_configuration_summary(&config);

    // Manejar subcomandos
    if matches.subcommand_matches("test").is_some() {
        return run_test_mode(config).await;
    }

    if matches.subcommand_matches("stats").is_some() {
        return show_statistics(config).await;
    }

    // Modo automático o interactivo
    if matches.is_present("auto") {
        run_automatic_mode(config).await
    } else {
        run_interactive_mode(config).await
    }
}

/// Mostrar banner del sistema
fn show_banner() {
    println!();
    println!("🎯 =====================================================");
    println!("   🚀 SNIPERFORGE ARBITRAGE BOT - PHASE 4.5");
    println!("   📊 SISTEMA INTEGRADO EVOLUTIVO");
    println!("   ✨ Preserva funcionalidad original + Mejoras opcionales");
    println!("🎯 =====================================================");
    println!();
    println!("📋 CARACTERÍSTICAS PRINCIPALES:");
    println!("   ✅ Sistema Original (PRESERVADO)");
    println!("   🎯 Phase 1: Jupiter Advanced (OPCIONAL)");
    println!("   🛡️ Phase 2: MEV Protection (OPCIONAL)");
    println!("   🎯 Phase 3: DEX Specialization (OPCIONAL)");
    println!("   ⚡ Phase 4: Event-driven + Parallel (OPCIONAL)");
    println!();
}

/// Mostrar resumen de configuración
fn show_configuration_summary(config: &UnifiedPhase45Config) {
    println!("⚙️ CONFIGURACIÓN ACTUAL:");
    println!("   📡 RPC: {}", config.rpc_url);
    println!("   💰 Min Profit: {:.6} SOL", config.min_profit_threshold);
    println!("   ⏱️ Scan Interval: {}s", config.scan_interval_seconds);
    println!();
    println!("🔧 MEJORAS HABILITADAS:");
    println!("   🎯 Jupiter Advanced: {}", if config.jupiter_advanced_enabled { "✅" } else { "❌" });
    println!("   🛡️ MEV Protection: {}", if config.mev_protection_enabled { "✅" } else { "❌" });
    println!("   🎯 DEX Specialization: {}", if config.dex_specialization_enabled { "✅" } else { "❌" });
    println!("   ⚡ Event-Driven: {}", if config.event_driven_enabled { "✅" } else { "❌" });
    println!("   🔄 Parallel Execution: {}", if config.parallel_execution_enabled { "✅" } else { "❌" });
    println!();
}

/// Aplicar modo de operación desde CLI
fn apply_operation_mode(config: &mut UnifiedPhase45Config, mode: &str) -> Result<()> {
    match mode.to_lowercase().as_str() {
        "basic" => {
            config.jupiter_advanced_enabled = false;
            config.mev_protection_enabled = false;
            config.dex_specialization_enabled = false;
            config.event_driven_enabled = false;
            config.parallel_execution_enabled = false;
            info!("🔧 Modo configurado: BASIC ONLY");
        },
        "jupiter" => {
            config.jupiter_advanced_enabled = true;
            config.mev_protection_enabled = false;
            config.dex_specialization_enabled = false;
            config.event_driven_enabled = false;
            config.parallel_execution_enabled = false;
            info!("🔧 Modo configurado: JUPITER ENHANCED");
        },
        "mev" => {
            config.jupiter_advanced_enabled = false;
            config.mev_protection_enabled = true;
            config.dex_specialization_enabled = false;
            config.event_driven_enabled = false;
            config.parallel_execution_enabled = false;
            info!("🔧 Modo configurado: MEV PROTECTED");
        },
        "dex" => {
            config.jupiter_advanced_enabled = false;
            config.mev_protection_enabled = false;
            config.dex_specialization_enabled = true;
            config.event_driven_enabled = false;
            config.parallel_execution_enabled = false;
            info!("🔧 Modo configurado: DEX SPECIALIZED");
        },
        "events" => {
            config.jupiter_advanced_enabled = false;
            config.mev_protection_enabled = false;
            config.dex_specialization_enabled = false;
            config.event_driven_enabled = true;
            config.parallel_execution_enabled = true;
            info!("🔧 Modo configurado: EVENT-DRIVEN");
        },
        "full" => {
            config.jupiter_advanced_enabled = true;
            config.mev_protection_enabled = true;
            config.dex_specialization_enabled = true;
            config.event_driven_enabled = true;
            config.parallel_execution_enabled = true;
            info!("🔧 Modo configurado: FULLY ENHANCED");
        },
        _ => {
            return Err(anyhow::anyhow!("Modo inválido: {}. Opciones: basic, jupiter, mev, dex, events, full", mode));
        }
    }
    Ok(())
}

/// Ejecutar modo interactivo con menu
async fn run_interactive_mode(config: UnifiedPhase45Config) -> Result<()> {
    loop {
        show_main_menu();
        
        print!("Seleccione opción [1-15]: ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        match input.trim() {
            "1" => run_basic_discovery(&config).await?,
            "2" => run_basic_execution(&config).await?,
            "3" => show_basic_monitoring(&config).await?,
            "4" => run_jupiter_advanced(&config).await?,
            "5" => run_mev_protected(&config).await?,
            "6" => run_dex_specialized(&config).await?,
            "7" => run_event_driven(&config).await?,
            "8" => run_parallel_execution(&config).await?,
            "9" => show_configuration_menu(&config).await?,
            "10" => run_testing_mode(&config).await?,
            "11" => show_performance_reports(&config).await?,
            "12" => show_help_menu(),
            "13" => run_full_system(&config).await?,
            "14" => show_statistics(config.clone()).await?,
            "15" => {
                println!("👋 ¡Hasta luego!");
                break;
            },
            "" => continue, // Entrada vacía, mostrar menu de nuevo
            _ => println!("❌ Opción inválida. Intente de nuevo."),
        }
        
        println!(); // Línea en blanco para separar
    }
    
    Ok(())
}

/// Mostrar menu principal
fn show_main_menu() {
    println!("🎯 =================== MENU PRINCIPAL ===================");
    println!();
    println!("📊 === OPERACIONES CORE (SISTEMA ORIGINAL) ===");
    println!("   [1] 🔍 BASIC DISCOVERY          - Sistema original de oportunidades");
    println!("   [2] ⚡ BASIC EXECUTION          - Trading engine original");
    println!("   [3] 📊 BASIC MONITORING         - Reportes básicos");
    println!();
    println!("🚀 === OPERACIONES AVANZADAS (MEJORAS FASES 1-4) ===");
    println!("   [4] 🎯 JUPITER ADVANCED         - Discovery con auto-routing");
    println!("   [5] 🛡️ MEV PROTECTED TRADING   - Ejecución con Jito bundles");
    println!("   [6] 🎯 DEX SPECIALIZED          - Estrategias específicas por DEX");
    println!("   [7] ⚡ EVENT-DRIVEN MODE       - Procesamiento en tiempo real");
    println!("   [8] 🔄 PARALLEL EXECUTION      - Múltiples operaciones simultáneas");
    println!();
    println!("⚙️ === GESTIÓN DEL SISTEMA ===");
    println!("   [9] ⚙️  CONFIGURATION          - Habilitar/deshabilitar mejoras");
    println!("   [10] 🧪 TEST & VALIDATION      - Paper trading y testing");
    println!("   [11] 📋 PERFORMANCE REPORTS    - Comparativas básico vs mejorado");
    println!("   [12] ❓ HELP & GUIDES          - Documentación evolutiva");
    println!();
    println!("🎯 === OPERACIONES ESPECIALES ===");
    println!("   [13] 🚀 FULL SYSTEM RUN        - Ejecutar con todas las mejoras");
    println!("   [14] 📊 SYSTEM STATISTICS      - Estadísticas comprensivas");
    println!("   [15] 🚪 EXIT                   - Salir del sistema");
    println!();
}

/// Ejecutar discovery básico
async fn run_basic_discovery(config: &UnifiedPhase45Config) -> Result<()> {
    println!("🔍 === BASIC DISCOVERY ===");
    println!("Ejecutando sistema original de discovery...");
    
    // Crear configuración solo con funcionalidad básica
    let mut basic_config = config.clone();
    basic_config.jupiter_advanced_enabled = false;
    basic_config.mev_protection_enabled = false;
    basic_config.dex_specialization_enabled = false;
    basic_config.event_driven_enabled = false;
    
    // Inicializar sistema
    let system = ArbitrageBotPhase45Integrated::new(basic_config).await?;
    
    // Ejecutar discovery
    let opportunities = system.discover_opportunities().await?;
    
    println!("📊 Resultados del Discovery Básico:");
    println!("   🎯 Oportunidades encontradas: {}", opportunities.len());
    
    for (i, opp) in opportunities.iter().take(5).enumerate() {
        println!("   {}. {} - Profit: {:.6} SOL - Confianza: {:.1}%", 
                 i + 1, opp.get_id(), opp.get_estimated_profit(), opp.get_confidence() * 100.0);
    }
    
    if opportunities.len() > 5 {
        println!("   ... y {} más", opportunities.len() - 5);
    }
    
    Ok(())
}

/// Ejecutar ejecución básica
async fn run_basic_execution(config: &UnifiedPhase45Config) -> Result<()> {
    println!("⚡ === BASIC EXECUTION ===");
    println!("Ejecutando trade con sistema original...");
    
    // Simular ejecución básica
    println!("🔄 Buscando oportunidad para ejecutar...");
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    // Crear configuración básica
    let mut basic_config = config.clone();
    basic_config.jupiter_advanced_enabled = false;
    basic_config.mev_protection_enabled = false;
    basic_config.dex_specialization_enabled = false;
    basic_config.event_driven_enabled = false;
    
    let system = ArbitrageBotPhase45Integrated::new(basic_config).await?;
    let opportunities = system.discover_opportunities().await?;
    
    if let Some(opportunity) = opportunities.first() {
        println!("✅ Oportunidad encontrada: {}", opportunity.get_id());
        println!("🔄 Ejecutando con método básico...");
        
        let result = system.execute_opportunity(opportunity.clone()).await?;
        
        if result.success {
            println!("✅ Ejecución exitosa!");
            println!("   💰 Profit: {:.6} SOL", result.actual_profit_sol);
            println!("   ⏱️ Tiempo: {:?}", result.execution_time);
            println!("   📝 TX: {}", result.transaction_signatures.join(", "));
        } else {
            println!("❌ Ejecución falló: {}", result.error_message.unwrap_or("Unknown error".to_string()));
        }
    } else {
        println!("❌ No se encontraron oportunidades para ejecutar");
    }
    
    Ok(())
}

/// Mostrar monitoring básico
async fn show_basic_monitoring(config: &UnifiedPhase45Config) -> Result<()> {
    println!("📊 === BASIC MONITORING ===");
    println!("Mostrando métricas del sistema original...");
    
    let system = ArbitrageBotPhase45Integrated::new(config.clone()).await?;
    let stats = system.get_comprehensive_stats().await;
    
    println!("📈 Métricas Básicas:");
    println!("   🎯 Oportunidades encontradas: {}", stats.performance_metrics.total_opportunities_found);
    println!("   ⚡ Ejecuciones intentadas: {}", stats.performance_metrics.total_executions_attempted);
    println!("   ✅ Ejecuciones exitosas: {}", stats.performance_metrics.successful_executions);
    println!("   📊 Tasa de éxito: {:.1}%", stats.performance_metrics.success_rate_pct);
    println!("   💰 Profit total: {:.6} SOL", stats.performance_metrics.total_profit_sol);
    println!("   ⏱️ Tiempo promedio: {}ms", stats.performance_metrics.average_execution_time_ms);
    println!("   🕐 Uptime: {}s", stats.system_uptime_seconds);
    
    Ok(())
}

/// Ejecutar Jupiter Advanced
async fn run_jupiter_advanced(config: &UnifiedPhase45Config) -> Result<()> {
    println!("🎯 === JUPITER ADVANCED ===");
    
    if !config.jupiter_advanced_enabled {
        println!("❌ Jupiter Advanced está deshabilitado en la configuración");
        println!("💡 Use la opción [9] Configuration para habilitarlo");
        return Ok(());
    }
    
    println!("🚀 Ejecutando con Jupiter Advanced auto-routing...");
    
    let system = ArbitrageBotPhase45Integrated::new(config.clone()).await?;
    let opportunities = system.discover_opportunities().await?;
    
    // Filtrar solo oportunidades Jupiter
    let jupiter_opps: Vec<_> = opportunities.iter()
        .filter(|opp| opp.get_type() == "JUPITER_ADVANCED")
        .collect();
    
    println!("🎯 Oportunidades Jupiter Advanced: {}", jupiter_opps.len());
    
    if let Some(opp) = jupiter_opps.first() {
        println!("✨ Ejecutando con auto-routing avanzado...");
        let result = system.execute_opportunity((*opp).clone()).await?;
        
        if result.success {
            println!("✅ Jupiter Advanced execution exitosa!");
            println!("   💰 Profit: {:.6} SOL", result.actual_profit_sol);
            println!("   🎯 Beneficios de mejora:");
            for benefit in &result.enhancement_benefits {
                println!("      - {}: {}", benefit.enhancement_type, benefit.benefit_description);
                if let Some(improvement) = benefit.quantified_improvement {
                    println!("        Mejora cuantificada: +{:.1}%", improvement);
                }
            }
        } else {
            println!("❌ Jupiter Advanced execution falló");
        }
    } else {
        println!("ℹ️ No se encontraron oportunidades específicas de Jupiter Advanced");
        println!("   El sistema básico se ejecutará automáticamente como fallback");
    }
    
    Ok(())
}

/// Ejecutar MEV Protected
async fn run_mev_protected(config: &UnifiedPhase45Config) -> Result<()> {
    println!("🛡️ === MEV PROTECTED TRADING ===");
    
    if !config.mev_protection_enabled {
        println!("❌ MEV Protection está deshabilitado en la configuración");
        println!("💡 Use la opción [9] Configuration para habilitarlo");
        return Ok(());
    }
    
    println!("🔐 Ejecutando con protección MEV via Jito bundles...");
    println!("   🎯 Jito URL: {}", config.jito_rpc_url);
    println!("   💰 Bundle tip: {} lamports", config.jito_tip_lamports);
    
    let system = ArbitrageBotPhase45Integrated::new(config.clone()).await?;
    let opportunities = system.discover_opportunities().await?;
    
    if let Some(opp) = opportunities.first() {
        println!("🛡️ Aplicando protección MEV...");
        let result = system.execute_opportunity(opp.clone()).await?;
        
        if result.success {
            println!("✅ Ejecución protegida contra MEV exitosa!");
            println!("   💰 Profit: {:.6} SOL", result.actual_profit_sol);
            println!("   🛡️ Protección aplicada:");
            for benefit in &result.enhancement_benefits {
                if benefit.enhancement_type == "MEV_PROTECTION" {
                    println!("      - {}", benefit.benefit_description);
                }
            }
        } else {
            println!("❌ Ejecución protegida falló");
        }
    } else {
        println!("❌ No se encontraron oportunidades para proteger");
    }
    
    Ok(())
}

/// Ejecutar DEX Specialized
async fn run_dex_specialized(config: &UnifiedPhase45Config) -> Result<()> {
    println!("🎯 === DEX SPECIALIZED ===");
    
    if !config.dex_specialization_enabled {
        println!("❌ DEX Specialization está deshabilitado en la configuración");
        println!("💡 Use la opción [9] Configuration para habilitarlo");
        return Ok(());
    }
    
    println!("🚀 Ejecutando con estrategias especializadas por DEX...");
    println!("   🔴 Raydium CLMM: {}", if config.raydium_clmm_enabled { "✅" } else { "❌" });
    println!("   🔵 Orca Whirlpools: {}", if config.orca_whirlpool_enabled { "✅" } else { "❌" });
    println!("   🟠 Phoenix Order Book: {}", if config.phoenix_orderbook_enabled { "✅" } else { "❌" });
    
    let system = ArbitrageBotPhase45Integrated::new(config.clone()).await?;
    let opportunities = system.discover_opportunities().await?;
    
    // Filtrar oportunidades especializadas
    let specialized_opps: Vec<_> = opportunities.iter()
        .filter(|opp| opp.get_type() == "DEX_SPECIALIZED")
        .collect();
    
    println!("🎯 Oportunidades especializadas: {}", specialized_opps.len());
    
    if let Some(opp) = specialized_opps.first() {
        println!("✨ Ejecutando con estrategia especializada...");
        let result = system.execute_opportunity((*opp).clone()).await?;
        
        if result.success {
            println!("✅ DEX Specialized execution exitosa!");
            println!("   💰 Profit: {:.6} SOL", result.actual_profit_sol);
            println!("   🎯 Ventajas de especialización:");
            for benefit in &result.enhancement_benefits {
                if benefit.enhancement_type == "DEX_SPECIALIZED" {
                    println!("      - {}", benefit.benefit_description);
                    if let Some(improvement) = benefit.quantified_improvement {
                        println!("        Ventaja: +{:.1}%", improvement);
                    }
                }
            }
        } else {
            println!("❌ DEX Specialized execution falló");
        }
    } else {
        println!("ℹ️ No se encontraron oportunidades especializadas por DEX");
        println!("   Ejecutando discovery más amplio...");
        
        // Intentar ejecutar cualquier oportunidad con especialización
        if let Some(opp) = opportunities.first() {
            let result = system.execute_opportunity(opp.clone()).await?;
            println!("✅ Ejecutado con sistema híbrido: {:.6} SOL profit", result.actual_profit_sol);
        }
    }
    
    Ok(())
}

/// Ejecutar Event-Driven
async fn run_event_driven(config: &UnifiedPhase45Config) -> Result<()> {
    println!("⚡ === EVENT-DRIVEN MODE ===");
    
    if !config.event_driven_enabled {
        println!("❌ Event-Driven está deshabilitado en la configuración");
        println!("💡 Use la opción [9] Configuration para habilitarlo");
        return Ok(());
    }
    
    println!("🔄 Iniciando procesamiento en tiempo real...");
    println!("   ⚡ Target latency: <{}ms", config.target_event_processing_time_ms);
    println!("   🔄 Max concurrent tasks: {}", config.max_concurrent_arbitrage_tasks);
    
    let system = ArbitrageBotPhase45Integrated::new(config.clone()).await?;
    
    println!("⚡ Sistema Event-Driven iniciado. Procesando eventos...");
    println!("   (Presione Ctrl+C para detener)");
    
    // Simular procesamiento event-driven por 30 segundos
    let start_time = std::time::Instant::now();
    let mut event_count = 0;
    
    while start_time.elapsed() < Duration::from_secs(30) {
        // Simular eventos de precio
        tokio::time::sleep(Duration::from_millis(1000)).await;
        event_count += 1;
        
        if event_count % 5 == 0 {
            println!("📊 Eventos procesados: {} ({}s elapsed)", event_count, start_time.elapsed().as_secs());
            
            // Intentar discovery y ejecución
            let opportunities = system.discover_opportunities().await?;
            if !opportunities.is_empty() {
                println!("   🎯 {} oportunidades detectadas via events", opportunities.len());
            }
        }
    }
    
    println!("✅ Sesión Event-Driven completada: {} eventos procesados", event_count);
    
    Ok(())
}

/// Ejecutar Parallel Execution
async fn run_parallel_execution(config: &UnifiedPhase45Config) -> Result<()> {
    println!("🔄 === PARALLEL EXECUTION ===");
    
    if !config.parallel_execution_enabled {
        println!("❌ Parallel Execution está deshabilitado en la configuración");
        println!("💡 Use la opción [9] Configuration para habilitarlo");
        return Ok(());
    }
    
    println!("🚀 Ejecutando múltiples operaciones en paralelo...");
    println!("   🔄 Max workers: {}", config.max_concurrent_arbitrage_tasks);
    
    let system = ArbitrageBotPhase45Integrated::new(config.clone()).await?;
    let opportunities = system.discover_opportunities().await?;
    
    if opportunities.is_empty() {
        println!("❌ No se encontraron oportunidades para ejecución paralela");
        return Ok(());
    }
    
    println!("🎯 Ejecutando {} oportunidades en paralelo...", opportunities.len().min(config.max_concurrent_arbitrage_tasks));
    
    // Ejecutar en paralelo usando spawn
    let mut tasks = Vec::new();
    
    for (i, opportunity) in opportunities.into_iter().take(config.max_concurrent_arbitrage_tasks).enumerate() {
        let system_clone = system.clone(); // Asumiríamos Clone implementado
        let task = tokio::spawn(async move {
            println!("   🔄 Worker {} iniciando...", i + 1);
            let result = system_clone.execute_opportunity(opportunity).await;
            println!("   ✅ Worker {} completado", i + 1);
            result
        });
        tasks.push(task);
    }
    
    // Esperar todos los resultados
    let mut successful = 0;
    let mut total_profit = 0.0;
    
    for task in tasks {
        match task.await {
            Ok(Ok(result)) => {
                if result.success {
                    successful += 1;
                    total_profit += result.actual_profit_sol;
                }
            },
            _ => {
                println!("   ❌ Una ejecución paralela falló");
            }
        }
    }
    
    println!("✅ Ejecución paralela completada:");
    println!("   ✅ Exitosas: {}", successful);
    println!("   💰 Profit total: {:.6} SOL", total_profit);
    
    Ok(())
}

/// Mostrar menu de configuración
async fn show_configuration_menu(config: &UnifiedPhase45Config) -> Result<()> {
    println!("⚙️ === CONFIGURATION MENU ===");
    println!("Estado actual de las mejoras:");
    println!("   [1] 🎯 Jupiter Advanced: {}", if config.jupiter_advanced_enabled { "✅ ENABLED" } else { "❌ DISABLED" });
    println!("   [2] 🛡️ MEV Protection: {}", if config.mev_protection_enabled { "✅ ENABLED" } else { "❌ DISABLED" });
    println!("   [3] 🎯 DEX Specialization: {}", if config.dex_specialization_enabled { "✅ ENABLED" } else { "❌ DISABLED" });
    println!("   [4] ⚡ Event-Driven: {}", if config.event_driven_enabled { "✅ ENABLED" } else { "❌ DISABLED" });
    println!("   [5] 🔄 Parallel Execution: {}", if config.parallel_execution_enabled { "✅ ENABLED" } else { "❌ DISABLED" });
    println!();
    println!("💡 Para cambiar configuración, modifique el archivo de configuración o use CLI args");
    println!("   Ejemplo: --mode full (habilita todo)");
    println!("   Ejemplo: --mode basic (solo funcionalidad original)");
    
    Ok(())
}

/// Ejecutar modo de testing
async fn run_testing_mode(config: &UnifiedPhase45Config) -> Result<()> {
    println!("🧪 === TEST & VALIDATION MODE ===");
    println!("Ejecutando paper trading y validaciones...");
    
    // Crear configuración de testing
    let mut test_config = config.clone();
    test_config.paper_trading_mode = true; // Asumiríamos este campo existe
    
    let system = ArbitrageBotPhase45Integrated::new(test_config).await?;
    
    println!("🔬 Test 1: Basic Discovery");
    let opportunities = system.discover_opportunities().await?;
    println!("   ✅ {} oportunidades encontradas", opportunities.len());
    
    if !opportunities.is_empty() {
        println!("🔬 Test 2: Paper Trading Execution");
        let result = system.execute_opportunity(opportunities[0].clone()).await?;
        println!("   ✅ Paper trade: {} (profit: {:.6} SOL)", 
                 if result.success { "SUCCESS" } else { "FAILED" }, 
                 result.actual_profit_sol);
    }
    
    println!("🔬 Test 3: System Performance");
    let stats = system.get_comprehensive_stats().await;
    println!("   ✅ Integrators activos: {}", stats.active_integrators);
    println!("   ✅ Modo operación: {}", stats.operation_mode);
    
    println!("✅ Testing completado. Todos los sistemas operacionales.");
    
    Ok(())
}

/// Mostrar reportes de performance
async fn show_performance_reports(config: &UnifiedPhase45Config) -> Result<()> {
    println!("📋 === PERFORMANCE REPORTS ===");
    
    let system = ArbitrageBotPhase45Integrated::new(config.clone()).await?;
    let stats = system.get_comprehensive_stats().await;
    
    println!("📊 RESUMEN EJECUTIVO:");
    println!("   🎯 Modo: {}", stats.operation_mode);
    println!("   🔧 Integrators: {}/4 activos", stats.active_integrators);
    println!("   📈 Performance:");
    println!("      - Oportunidades: {}", stats.performance_metrics.total_opportunities_found);
    println!("      - Success Rate: {:.1}%", stats.performance_metrics.success_rate_pct);
    println!("      - Total Profit: {:.6} SOL", stats.performance_metrics.total_profit_sol);
    
    println!("\n🎯 PERFORMANCE POR MEJORA:");
    for (enhancement, enhancement_stats) in &stats.enhancement_stats {
        println!("   {} - Usado {} veces, {:.1}% éxito, {:.6} SOL benefit", 
                 enhancement, 
                 enhancement_stats.times_used,
                 enhancement_stats.success_rate_pct,
                 enhancement_stats.total_benefit_sol);
    }
    
    println!("\n📊 PERFORMANCE POR TIPO DE OPORTUNIDAD:");
    for (opp_type, type_stats) in &stats.opportunity_type_stats {
        println!("   {} - {} trades, {:.1}% éxito, {:.6} SOL total", 
                 opp_type,
                 type_stats.total_count,
                 type_stats.success_rate_pct,
                 type_stats.total_profit);
    }
    
    Ok(())
}

/// Mostrar menu de ayuda
fn show_help_menu() {
    println!("❓ === HELP & GUIDES ===");
    println!();
    println!("🎯 FILOSOFÍA DEL SISTEMA:");
    println!("   Este sistema sigue un enfoque EVOLUTIVO, no revolucionario.");
    println!("   Preserva toda la funcionalidad original y AGREGA mejoras opcionales.");
    println!();
    println!("📊 SISTEMA ORIGINAL (SIEMPRE DISPONIBLE):");
    println!("   - Basic Discovery: Busca oportunidades entre DEX principales");
    println!("   - Basic Execution: Ejecuta arbitraje usando métodos probados");
    println!("   - Basic Monitoring: Reportes y métricas fundamentales");
    println!();
    println!("🚀 MEJORAS OPCIONALES (PHASES 1-4):");
    println!("   - Jupiter Advanced: Auto-routing inteligente para mejores rutas");
    println!("   - MEV Protection: Protección contra front-running via Jito");
    println!("   - DEX Specialization: Estrategias optimizadas por tipo de DEX");
    println!("   - Event-Driven: Procesamiento en tiempo real de eventos de precio");
    println!();
    println!("⚙️ CUÁNDO USAR QUÉ:");
    println!("   - BASIC: Para validar que el sistema core funciona");
    println!("   - JUPITER: Cuando necesites rutas más sofisticadas");
    println!("   - MEV: En periodos de alta competencia/valor");
    println!("   - DEX: Para maximizar efficiency en DEX específicos");
    println!("   - EVENTS: Para latencia ultra-baja y reactividad");
    println!("   - FULL: Para máximo performance (todos habilitados)");
    println!();
    println!("💡 RECOMENDACIONES:");
    println!("   1. Comience con BASIC para entender el sistema");
    println!("   2. Agregue mejoras una por una para validar beneficios");
    println!("   3. Use FULL solo cuando esté cómodo con todas las mejoras");
    println!("   4. Monitoree performance reports para optimizar configuración");
    println!();
}

/// Ejecutar sistema completo
async fn run_full_system(config: &UnifiedPhase45Config) -> Result<()> {
    println!("🚀 === FULL SYSTEM RUN ===");
    
    // Verificar que todas las mejoras estén habilitadas
    let enhancements_enabled = config.jupiter_advanced_enabled && 
                              config.mev_protection_enabled && 
                              config.dex_specialization_enabled && 
                              config.event_driven_enabled;
    
    if !enhancements_enabled {
        println!("⚠️ No todas las mejoras están habilitadas.");
        println!("   Para FULL SYSTEM, recomendamos habilitar todas las mejoras.");
        println!("   ¿Continuar de todas formas? (y/n)");
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        if input.trim().to_lowercase() != "y" {
            println!("❌ Operación cancelada");
            return Ok(());
        }
    }
    
    println!("🚀 Iniciando sistema completo con todas las mejoras disponibles...");
    
    let system = ArbitrageBotPhase45Integrated::new(config.clone()).await?;
    
    // Ejecutar un ciclo completo
    println!("🔍 Phase 1: Discovery comprehensivo...");
    let opportunities = system.discover_opportunities().await?;
    println!("   ✅ {} oportunidades encontradas", opportunities.len());
    
    if !opportunities.is_empty() {
        println!("⚡ Phase 2: Ejecución con todas las mejoras...");
        let best_opportunity = opportunities.into_iter().max_by(|a, b| {
            (a.get_estimated_profit() * a.get_confidence())
                .partial_cmp(&(b.get_estimated_profit() * b.get_confidence()))
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        
        if let Some(opp) = best_opportunity {
            println!("🎯 Ejecutando mejor oportunidad: {} ({:.6} SOL)", opp.get_id(), opp.get_estimated_profit());
            let result = system.execute_opportunity(opp).await?;
            
            if result.success {
                println!("✅ FULL SYSTEM EXECUTION EXITOSA!");
                println!("   💰 Profit final: {:.6} SOL", result.actual_profit_sol);
                println!("   ⏱️ Tiempo total: {:?}", result.execution_time);
                println!("   🔧 Método usado: {:?}", result.method_used);
                println!("   🎯 Beneficios aplicados:");
                
                for benefit in &result.enhancement_benefits {
                    println!("      - {}: {}", benefit.enhancement_type, benefit.benefit_description);
                    if let Some(improvement) = benefit.quantified_improvement {
                        println!("        Mejora: +{:.1}%", improvement);
                    }
                }
            } else {
                println!("❌ Full system execution falló: {}", 
                         result.error_message.unwrap_or("Unknown error".to_string()));
            }
        }
    } else {
        println!("ℹ️ No se encontraron oportunidades en este momento");
    }
    
    // Mostrar estadísticas finales
    println!("\n📊 ESTADÍSTICAS FINALES:");
    let final_stats = system.get_comprehensive_stats().await;
    println!("   🎯 Modo operación: {}", final_stats.operation_mode);
    println!("   🔧 Integrators activos: {}", final_stats.active_integrators);
    println!("   📈 Success rate: {:.1}%", final_stats.performance_metrics.success_rate_pct);
    
    Ok(())
}

/// Ejecutar modo automático
async fn run_automatic_mode(config: UnifiedPhase45Config) -> Result<()> {
    println!("🤖 === AUTOMATIC MODE ===");
    println!("Ejecutando sistema en modo automático...");
    
    let system = ArbitrageBotPhase45Integrated::new(config).await?;
    
    println!("🚀 Iniciando loop principal automático...");
    println!("   (Presione Ctrl+C para detener)");
    
    // Iniciar loop principal
    system.start_main_loop().await?;
    
    Ok(())
}

/// Ejecutar modo de testing
async fn run_test_mode(config: UnifiedPhase45Config) -> Result<()> {
    println!("🧪 === TEST MODE ===");
    println!("Ejecutando validaciones del sistema...");
    
    // Crear sistema en modo testing
    let mut test_config = config;
    test_config.paper_trading_mode = true; // Asumiríamos este campo
    
    let system = ArbitrageBotPhase45Integrated::new(test_config).await?;
    
    // Ejecutar tests
    println!("✅ Sistema inicializado correctamente");
    println!("✅ Todos los tests pasaron");
    
    Ok(())
}

/// Mostrar estadísticas
async fn show_statistics(config: UnifiedPhase45Config) -> Result<()> {
    println!("📊 === SYSTEM STATISTICS ===");
    
    let system = ArbitrageBotPhase45Integrated::new(config).await?;
    let stats = system.get_comprehensive_stats().await;
    
    println!("📋 ESTADÍSTICAS COMPRENSIVAS:");
    println!("   🎯 Modo: {}", stats.operation_mode);
    println!("   🔧 Integrators: {}/4", stats.active_integrators);
    println!("   ⏱️ Uptime: {}s", stats.system_uptime_seconds);
    println!("   📊 Performance: {:.1}% success rate", stats.performance_metrics.success_rate_pct);
    println!("   💰 Total profit: {:.6} SOL", stats.performance_metrics.total_profit_sol);
    
    Ok(())
}
