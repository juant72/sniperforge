// SniperForge Enterprise - Ejemplo Completo TypeScript
// Demostración de todas las capacidades empresariales

import { TCPClient } from '@sniperforge/sdk';

class SniperForgeEnterpriseManager {
    private client: TCPClient;
    private activeBots: Map<string, any> = new Map();
    private monitoring: boolean = false;

    constructor(host = 'localhost', port = 8888) {
        this.client = new TCPClient(host, port);
    }

    async initialize(): Promise<void> {
        try {
            await this.client.connect();
            console.log('✅ Conectado a SniperForge Enterprise');
            
            // Verificar conectividad
            await this.client.sendCommand({ Ping: null });
            console.log('🏓 Servidor respondiendo correctamente');
            
        } catch (error) {
            throw new Error(`Error de conexión: ${error}`);
        }
    }

    async createLiquiditySniperBot(pairs: string[], maxPosition: number): Promise<string> {
        const config = {
            bot_type: 'LiquiditySniper',
            trading_pairs: pairs,
            max_position_size: maxPosition,
            risk_level: 'Conservative',
            enable_ai_analysis: true,
            slippage_tolerance: 0.1,
            min_liquidity_threshold: 10000.0
        };

        const response = await this.client.sendCommand({
            CreateBot: {
                bot_type: 'LiquiditySniper',
                config: config
            }
        });

        const botId = response.BotCreated.bot_id;
        console.log(`🤖 LiquiditySniper creado: ${botId}`);

        // Iniciar bot
        await this.client.sendCommand({
            StartBot: {
                bot_id: botId,
                config: config
            }
        });

        this.activeBots.set(botId, {
            type: 'LiquiditySniper',
            config: config,
            startTime: new Date()
        });

        console.log(`🚀 Bot ${botId} iniciado y cazando liquidez`);
        return botId;
    }

    async createArbitrageBot(exchanges: string[]): Promise<string> {
        const config = {
            bot_type: 'EnhancedArbitrage',
            exchanges: exchanges,
            max_position_size: 500.0,
            min_profit_threshold: 0.3,
            enable_flash_loans: true,
            risk_level: 'Moderate'
        };

        const response = await this.client.sendCommand({
            CreateBot: {
                bot_type: 'EnhancedArbitrage',
                config: config
            }
        });

        const botId = response.BotCreated.bot_id;
        console.log(`📈 ArbitrageBot creado: ${botId}`);

        await this.client.sendCommand({
            StartBot: {
                bot_id: botId,
                config: config
            }
        });

        this.activeBots.set(botId, {
            type: 'EnhancedArbitrage',
            config: config,
            startTime: new Date()
        });

        console.log(`💰 Bot de arbitraje ${botId} buscando oportunidades`);
        return botId;
    }

    async getSystemOverview(): Promise<any> {
        const [systemMetrics, resourceStatus, systemState] = await Promise.all([
            this.client.sendCommand({ GetSystemMetrics: null }),
            this.client.sendCommand({ GetResourceStatus: null }),
            this.client.sendCommand({ GetSystemState: null })
        ]);

        return {
            metrics: systemMetrics.SystemMetrics,
            resources: resourceStatus.ResourceStatus,
            state: systemState.SystemState
        };
    }

    async getBotPerformance(botId: string): Promise<any> {
        const [status, metrics] = await Promise.all([
            this.client.sendCommand({ GetBotStatus: { bot_id: botId } }),
            this.client.sendCommand({ GetBotMetrics: { bot_id: botId } })
        ]);

        return {
            status: status.BotStatus,
            metrics: metrics.BotMetrics,
            botInfo: this.activeBots.get(botId)
        };
    }

    async startSystemMonitoring(): Promise<void> {
        if (this.monitoring) return;
        
        this.monitoring = true;
        console.log('📊 Iniciando monitoreo del sistema...');

        setInterval(async () => {
            try {
                const overview = await this.getSystemOverview();
                
                console.log('\n=== ESTADO DEL SISTEMA ===');
                console.log(`🤖 Bots: ${overview.metrics.running_bots}/${overview.metrics.total_bots} activos`);
                console.log(`💰 Ganancia Total: $${overview.metrics.total_profit.toFixed(2)}`);
                console.log(`📈 Trades Ejecutados: ${overview.metrics.total_trades}`);
                console.log(`💾 Memoria: ${overview.resources.memory_usage_mb.toFixed(1)}MB`);
                console.log(`⏱️ Uptime: ${Math.floor(overview.metrics.uptime_seconds / 3600)}h ${Math.floor((overview.metrics.uptime_seconds % 3600) / 60)}m`);
                
                if (overview.resources.resource_warning) {
                    console.log(`⚠️ Advertencia: ${overview.resources.resource_warning}`);
                }

                // Monitorear cada bot individualmente
                for (const [botId, botInfo] of this.activeBots.entries()) {
                    try {
                        const performance = await this.getBotPerformance(botId);
                        console.log(`\n🤖 Bot ${botId.substring(0, 8)}... (${botInfo.type})`);
                        console.log(`   Estado: ${performance.status}`);
                        console.log(`   PnL: $${performance.metrics.trading.total_pnl_usd.toFixed(2)}`);
                        console.log(`   Trades: ${performance.metrics.trading.trades_executed}`);
                        console.log(`   Win Rate: ${performance.metrics.trading.win_rate.toFixed(1)}%`);
                        console.log(`   Uptime: ${Math.floor(performance.metrics.performance.uptime_seconds / 3600)}h`);
                    } catch (error) {
                        console.log(`❌ Error monitoreando bot ${botId}: ${error}`);
                    }
                }

            } catch (error) {
                console.error('❌ Error en monitoreo del sistema:', error);
            }
        }, 15000); // Cada 15 segundos
    }

    async createSystemBackup(): Promise<string> {
        const response = await this.client.sendCommand({ CreateBackup: null });
        const backupPath = response.BackupCreated;
        console.log(`💾 Backup creado: ${backupPath}`);
        return backupPath;
    }

    async emergencyStopAll(): Promise<void> {
        console.log('🚨 PARADA DE EMERGENCIA - Deteniendo todos los bots...');
        
        const response = await this.client.sendCommand({ StopAllBots: null });
        const result = response.MassControlResult;
        
        console.log(`✅ Bots detenidos exitosamente: ${result.successful.length}`);
        console.log(`❌ Bots con error: ${result.failed.length}`);
        
        if (result.failed.length > 0) {
            console.log('Errores:');
            result.failed.forEach(([botId, error]: [string, string]) => {
                console.log(`  - ${botId}: ${error}`);
            });
        }

        // Crear backup antes del shutdown
        await this.createSystemBackup();
    }

    async gracefulShutdown(): Promise<void> {
        console.log('🛑 Iniciando shutdown graceful...');
        
        // Detener monitoreo
        this.monitoring = false;
        
        // Detener todos los bots
        await this.emergencyStopAll();
        
        // Forzar guardado del estado
        await this.client.sendCommand({ ForceSave: null });
        console.log('💾 Estado del sistema guardado');
        
        // Cerrar conexión
        this.client.disconnect();
        console.log('✅ Shutdown completado');
    }
}

// Ejemplo de uso empresarial
async function enterpriseExample() {
    const manager = new SniperForgeEnterpriseManager();
    
    try {
        // Inicializar sistema
        await manager.initialize();
        
        // Crear bots diversificados
        const liquidityBot1 = await manager.createLiquiditySniperBot(['BTC/USDT', 'ETH/USDT'], 1000);
        const liquidityBot2 = await manager.createLiquiditySniperBot(['ADA/USDT', 'SOL/USDT'], 500);
        const arbitrageBot = await manager.createArbitrageBot(['binance', 'coinbase', 'kraken']);
        
        console.log('\n🎯 Portfolio de bots creado exitosamente');
        console.log(`🤖 LiquiditySniper #1: ${liquidityBot1}`);
        console.log(`🤖 LiquiditySniper #2: ${liquidityBot2}`);
        console.log(`📈 ArbitrageBot: ${arbitrageBot}`);
        
        // Iniciar monitoreo empresarial
        await manager.startSystemMonitoring();
        
        // Configurar manejo de señales para shutdown graceful
        process.on('SIGINT', async () => {
            console.log('\n🛑 Recibida señal de interrupción...');
            await manager.gracefulShutdown();
            process.exit(0);
        });
        
        // Mantener el programa corriendo
        console.log('\n✅ Sistema empresarial funcionando. Presiona Ctrl+C para shutdown graceful.');
        console.log('📊 Monitoreo cada 15 segundos iniciado...');
        
        // Demo: Crear backup cada hora
        setInterval(async () => {
            try {
                await manager.createSystemBackup();
            } catch (error) {
                console.error('Error creando backup automático:', error);
            }
        }, 3600000); // 1 hora
        
    } catch (error) {
        console.error('❌ Error en ejemplo empresarial:', error);
        process.exit(1);
    }
}

// Ejecutar ejemplo si es llamado directamente
if (require.main === module) {
    enterpriseExample().catch(console.error);
}

export { SniperForgeEnterpriseManager };
