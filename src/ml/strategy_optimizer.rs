//! Strategy Optimization using Genetic Algorithms
//!
//! This module implements:
//! - Genetic algorithm optimization for trading parameters
//! - Multi-objective fitness evaluation (Sharpe, drawdown, win rate)
//! - Automated backtesting across historical data
//! - Hyperparameter tuning for optimal performance

use crate::strategies::MarketData;
use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{FeatureVector, MLPrediction, StrategyOptimizerConfig};

/// Genetic Algorithm-based strategy optimizer
pub struct StrategyOptimizer {
    config: StrategyOptimizerConfig,
    population: Vec<Individual>,
    generation: u32,
    best_individual: Option<Individual>,
    fitness_history: Vec<FitnessGeneration>,
    last_optimization: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Individual {
    pub parameters: HashMap<String, f64>,
    pub fitness: Option<FitnessScore>,
    pub age: u32,
    pub mutations: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FitnessScore {
    pub sharpe_ratio: f64,
    pub max_drawdown: f64,
    pub win_rate: f64,
    pub total_return: f64,
    pub volatility: f64,
    pub combined_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FitnessGeneration {
    pub generation: u32,
    pub best_fitness: f64,
    pub average_fitness: f64,
    pub worst_fitness: f64,
    pub diversity_score: f64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationResult {
    pub best_parameters: HashMap<String, f64>,
    pub fitness_score: FitnessScore,
    pub generations_run: u32,
    pub convergence_achieved: bool,
    pub optimization_time_seconds: f64,
    pub fitness_history: Vec<FitnessGeneration>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestResult {
    pub total_trades: u32,
    pub winning_trades: u32,
    pub losing_trades: u32,
    pub total_return: f64,
    pub max_drawdown: f64,
    pub sharpe_ratio: f64,
    pub volatility: f64,
    pub trade_details: Vec<TradeResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeResult {
    pub entry_time: DateTime<Utc>,
    pub exit_time: DateTime<Utc>,
    pub entry_price: f64,
    pub exit_price: f64,
    pub quantity: f64,
    pub pnl: f64,
    pub return_pct: f64,
    pub trade_type: String,
}

impl StrategyOptimizer {
    pub async fn new(config: StrategyOptimizerConfig) -> Result<Self> {
        Ok(Self {
            config,
            population: Vec::new(),
            generation: 0,
            best_individual: None,
            fitness_history: Vec::new(),
            last_optimization: Utc::now(),
        })
    }
    /// Optimize trading strategy parameters using genetic algorithm
    pub async fn optimize_parameters(
        &mut self,
        historical_data: &[MarketData],
        initial_parameters: &HashMap<String, f64>,
    ) -> Result<HashMap<String, f64>> {
        let start_time = std::time::Instant::now();

        // Initialize population
        self.initialize_population(initial_parameters).await?;
        // Run genetic algorithm
        for generation in 0..self.config.generations {
            self.generation = generation as u32;

            // Evaluate fitness for all individuals
            self.evaluate_population(historical_data).await?;

            // Record generation statistics
            self.record_generation_stats().await?;

            // Check for convergence
            if self.check_convergence() {
                tracing::info!("Optimization converged at generation {}", generation);
                break;
            }

            // Create next generation
            self.evolve_population().await?;

            // Log progress every 10 generations
            if generation % 10 == 0 {
                if let Some(best) = &self.best_individual {
                    if let Some(fitness) = &best.fitness {
                        tracing::info!(
                            "Generation {}: Best fitness = {:.4}",
                            generation,
                            fitness.combined_score
                        );
                    }
                }
            }
        }

        let optimization_time = start_time.elapsed().as_secs_f64();
        self.last_optimization = Utc::now();

        // Return best parameters found
        if let Some(best) = &self.best_individual {
            tracing::info!(
                "Optimization completed in {:.2}s, {} generations",
                optimization_time,
                self.generation
            );
            Ok(best.parameters.clone())
        } else {
            Err(anyhow!("Optimization failed to find viable parameters"))
        }
    }

    async fn initialize_population(
        &mut self,
        initial_parameters: &HashMap<String, f64>,
    ) -> Result<()> {
        self.population.clear();
        let mut rng = thread_rng();

        // Add the initial parameters as the first individual
        let initial_individual = Individual {
            parameters: initial_parameters.clone(),
            fitness: None,
            age: 0,
            mutations: 0,
        };
        self.population.push(initial_individual);

        // Generate random variations for the rest of the population
        for _ in 1..self.config.population_size {
            let mut individual_params = HashMap::new();

            for (key, &base_value) in initial_parameters {
                // Add random variation (±50% of base value)
                let variation = rng.gen_range(-0.5..=0.5);
                let new_value = base_value * (1.0 + variation);

                // Ensure parameters stay within reasonable bounds
                let bounded_value = match key.as_str() {
                    "stop_loss" => new_value.clamp(0.01, 0.2),     // 1% to 20%
                    "take_profit" => new_value.clamp(0.02, 0.5),   // 2% to 50%
                    "position_size" => new_value.clamp(0.1, 1.0),  // 10% to 100%
                    "rsi_oversold" => new_value.clamp(20.0, 40.0), // RSI bounds
                    "rsi_overbought" => new_value.clamp(60.0, 80.0),
                    "volume_threshold" => new_value.clamp(0.5, 5.0), // Volume multiplier
                    _ => new_value.clamp(0.1, 10.0),                 // Default bounds
                };

                individual_params.insert(key.clone(), bounded_value);
            }

            let individual = Individual {
                parameters: individual_params,
                fitness: None,
                age: 0,
                mutations: 0,
            };
            self.population.push(individual);
        }

        tracing::info!(
            "Initialized population with {} individuals",
            self.population.len()
        );
        Ok(())
    }
    async fn evaluate_population(&mut self, historical_data: &[MarketData]) -> Result<()> {
        let mut fitness_values = Vec::new();
        for individual in &self.population {
            if individual.fitness.is_none() {
                let fitness = self
                    .evaluate_individual(individual, historical_data)
                    .await?;
                fitness_values.push(fitness);
            } else {
                fitness_values.push(individual.fitness.clone().unwrap());
            }
        }

        // Update fitness values
        for (i, fitness) in fitness_values.into_iter().enumerate() {
            self.population[i].fitness = Some(fitness);
        }

        // Update best individual
        if let Some(best_in_generation) = self.population.iter().max_by(|a, b| {
            let fitness_a = a.fitness.as_ref().map(|f| f.combined_score).unwrap_or(0.0);
            let fitness_b = b.fitness.as_ref().map(|f| f.combined_score).unwrap_or(0.0);
            fitness_a
                .partial_cmp(&fitness_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        }) {
            if self.best_individual.is_none()
                || best_in_generation.fitness.as_ref().unwrap().combined_score
                    > self
                        .best_individual
                        .as_ref()
                        .unwrap()
                        .fitness
                        .as_ref()
                        .unwrap()
                        .combined_score
            {
                self.best_individual = Some(best_in_generation.clone());
            }
        }

        Ok(())
    }

    async fn evaluate_individual(
        &self,
        individual: &Individual,
        historical_data: &[MarketData],
    ) -> Result<FitnessScore> {
        // Run backtest with individual's parameters
        let backtest_result = self
            .run_backtest(&individual.parameters, historical_data)
            .await?;

        // Calculate multi-objective fitness score
        let fitness = self.calculate_fitness(&backtest_result)?;

        Ok(fitness)
    }
    async fn run_backtest(
        &self,
        parameters: &HashMap<String, f64>,
        historical_data: &[MarketData],
    ) -> Result<BacktestResult> {
        let mut trades = Vec::new();
        let mut equity_curve = Vec::new();
        let mut current_equity = 10000.0; // Starting capital
        let mut position: Option<Position> = None;

        for (i, market_data) in historical_data.iter().enumerate() {
            let price = market_data.current_price;
            equity_curve.push(current_equity);

            // Simple strategy implementation for backtesting
            if let Some(pos) = &position {
                // Check exit conditions
                let should_exit = self.should_exit_position(pos, price, parameters);
                if should_exit {
                    let exit_time = Utc::now(); // Use current time since MarketData doesn't have timestamp
                    let pnl = (price - pos.entry_price) * pos.quantity;
                    let return_pct = pnl / (pos.entry_price * pos.quantity);

                    current_equity += pnl;

                    let trade = TradeResult {
                        entry_time: pos.entry_time,
                        exit_time,
                        entry_price: pos.entry_price,
                        exit_price: price,
                        quantity: pos.quantity,
                        pnl,
                        return_pct,
                        trade_type: pos.trade_type.clone(),
                    };

                    trades.push(trade);
                    position = None;
                }
            } else {
                // Check entry conditions
                if let Some(signal) =
                    self.should_enter_position(market_data, parameters, historical_data, i)
                {
                    let position_size = parameters.get("position_size").unwrap_or(&0.1);
                    let quantity = (current_equity * position_size) / price;
                    position = Some(Position {
                        entry_time: Utc::now(), // Use current time since MarketData doesn't have timestamp
                        entry_price: price,
                        quantity,
                        trade_type: signal,
                    });
                }
            }
        }

        // Calculate backtest metrics
        let total_trades = trades.len() as u32;
        let winning_trades = trades.iter().filter(|t| t.pnl > 0.0).count() as u32;
        let losing_trades = total_trades - winning_trades;

        let total_return = (current_equity - 10000.0) / 10000.0;
        let max_drawdown = self.calculate_max_drawdown(&equity_curve);
        let returns: Vec<f64> = trades.iter().map(|t| t.return_pct).collect();
        let (sharpe_ratio, volatility) = self.calculate_sharpe_ratio(&returns);

        Ok(BacktestResult {
            total_trades,
            winning_trades,
            losing_trades,
            total_return,
            max_drawdown,
            sharpe_ratio,
            volatility,
            trade_details: trades,
        })
    }
    fn should_enter_position(
        &self,
        market_data: &MarketData,
        parameters: &HashMap<String, f64>,
        historical_data: &[MarketData],
        current_index: usize,
    ) -> Option<String> {
        // Simple RSI-based entry strategy
        if current_index < 14 {
            return None;
        } // Need enough data for RSI

        let rsi_oversold = parameters.get("rsi_oversold").unwrap_or(&30.0);
        let rsi_overbought = parameters.get("rsi_overbought").unwrap_or(&70.0);
        let volume_threshold = parameters.get("volume_threshold").unwrap_or(&1.5);

        // Calculate simple RSI
        let recent_prices: Vec<f64> = historical_data[current_index - 14..=current_index]
            .iter()
            .map(|d| d.current_price)
            .collect();

        if let Ok(rsi) = self.calculate_simple_rsi(&recent_prices) {
            let avg_volume = historical_data[current_index - 10..current_index]
                .iter()
                .map(|d| d.volume_24h)
                .sum::<f64>()
                / 10.0;

            let volume_spike = market_data.volume_24h > avg_volume * volume_threshold;

            if rsi < *rsi_oversold && volume_spike {
                return Some("long".to_string());
            } else if rsi > *rsi_overbought && volume_spike {
                return Some("short".to_string());
            }
        }

        None
    }

    fn should_exit_position(
        &self,
        position: &Position,
        current_price: f64,
        parameters: &HashMap<String, f64>,
    ) -> bool {
        let stop_loss = parameters.get("stop_loss").unwrap_or(&0.05); // 5% default
        let take_profit = parameters.get("take_profit").unwrap_or(&0.1); // 10% default

        let price_change = (current_price - position.entry_price) / position.entry_price;

        match position.trade_type.as_str() {
            "long" => price_change <= -stop_loss || price_change >= *take_profit,
            "short" => price_change >= *stop_loss || price_change <= -*take_profit,
            _ => false,
        }
    }

    fn calculate_simple_rsi(&self, prices: &[f64]) -> Result<f64> {
        if prices.len() < 15 {
            return Err(anyhow!("Insufficient data for RSI"));
        }

        let mut gains = 0.0;
        let mut losses = 0.0;

        for i in 1..prices.len() {
            let change = prices[i] - prices[i - 1];
            if change > 0.0 {
                gains += change;
            } else {
                losses -= change;
            }
        }

        let avg_gain = gains / 14.0;
        let avg_loss = losses / 14.0;

        if avg_loss == 0.0 {
            return Ok(100.0);
        }

        let rs = avg_gain / avg_loss;
        Ok(100.0 - (100.0 / (1.0 + rs)))
    }

    fn calculate_fitness(&self, backtest_result: &BacktestResult) -> Result<FitnessScore> {
        // Multi-objective fitness function
        let sharpe_weight = 0.4;
        let drawdown_weight = 0.3;
        let winrate_weight = 0.2;
        let return_weight = 0.1;

        // Normalize metrics to 0-1 scale
        let sharpe_score = (backtest_result.sharpe_ratio.max(0.0) / 3.0).min(1.0); // Cap at 3.0 Sharpe
        let drawdown_score = (1.0 - backtest_result.max_drawdown.abs()).max(0.0); // Lower drawdown is better
        let winrate_score = if backtest_result.total_trades > 0 {
            backtest_result.winning_trades as f64 / backtest_result.total_trades as f64
        } else {
            0.0
        };
        let return_score = (backtest_result.total_return + 1.0).max(0.0).min(2.0) / 2.0; // Cap at 100% return

        let combined_score = sharpe_score * sharpe_weight
            + drawdown_score * drawdown_weight
            + winrate_score * winrate_weight
            + return_score * return_weight;

        Ok(FitnessScore {
            sharpe_ratio: backtest_result.sharpe_ratio,
            max_drawdown: backtest_result.max_drawdown,
            win_rate: winrate_score,
            total_return: backtest_result.total_return,
            volatility: backtest_result.volatility,
            combined_score,
        })
    }

    fn calculate_max_drawdown(&self, equity_curve: &[f64]) -> f64 {
        let mut max_drawdown = 0.0;
        let mut peak = equity_curve[0];

        for &equity in equity_curve {
            if equity > peak {
                peak = equity;
            }
            let drawdown = (peak - equity) / peak;
            if drawdown > max_drawdown {
                max_drawdown = drawdown;
            }
        }

        max_drawdown
    }

    fn calculate_sharpe_ratio(&self, returns: &[f64]) -> (f64, f64) {
        if returns.is_empty() {
            return (0.0, 0.0);
        }

        let mean_return = returns.iter().sum::<f64>() / returns.len() as f64;
        let variance = returns
            .iter()
            .map(|r| (r - mean_return).powi(2))
            .sum::<f64>()
            / returns.len() as f64;
        let volatility = variance.sqrt();

        let sharpe_ratio = if volatility > 0.0 {
            mean_return / volatility
        } else {
            0.0
        };

        (sharpe_ratio, volatility)
    }

    async fn record_generation_stats(&mut self) -> Result<()> {
        let fitness_scores: Vec<f64> = self
            .population
            .iter()
            .filter_map(|ind| ind.fitness.as_ref().map(|f| f.combined_score))
            .collect();

        if !fitness_scores.is_empty() {
            let best_fitness = fitness_scores
                .iter()
                .fold(f64::NEG_INFINITY, |a, &b| a.max(b));
            let worst_fitness = fitness_scores.iter().fold(f64::INFINITY, |a, &b| a.min(b));
            let average_fitness = fitness_scores.iter().sum::<f64>() / fitness_scores.len() as f64;

            // Calculate diversity score (standard deviation of fitness scores)
            let variance = fitness_scores
                .iter()
                .map(|score| (score - average_fitness).powi(2))
                .sum::<f64>()
                / fitness_scores.len() as f64;
            let diversity_score = variance.sqrt();

            let generation_stats = FitnessGeneration {
                generation: self.generation,
                best_fitness,
                average_fitness,
                worst_fitness,
                diversity_score,
                timestamp: Utc::now(),
            };

            self.fitness_history.push(generation_stats);
        }

        Ok(())
    }

    fn check_convergence(&self) -> bool {
        if self.fitness_history.len() < 10 {
            return false;
        }

        // Check if fitness has plateaued over last 10 generations
        let recent_best: Vec<f64> = self
            .fitness_history
            .iter()
            .rev()
            .take(10)
            .map(|gen| gen.best_fitness)
            .collect();

        let fitness_range = recent_best.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b))
            - recent_best.iter().fold(f64::INFINITY, |a, &b| a.min(b));

        fitness_range < 0.001 // Converged if improvement is less than 0.1%
    }

    async fn evolve_population(&mut self) -> Result<()> {
        let mut new_population = Vec::new();
        let mut rng = thread_rng();

        // Keep top 10% of population (elitism)
        let elite_count = (self.config.population_size as f64 * 0.1) as usize;
        let mut sorted_population = self.population.clone();
        sorted_population.sort_by(|a, b| {
            let fitness_a = a.fitness.as_ref().map(|f| f.combined_score).unwrap_or(0.0);
            let fitness_b = b.fitness.as_ref().map(|f| f.combined_score).unwrap_or(0.0);
            fitness_b
                .partial_cmp(&fitness_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        for i in 0..elite_count {
            let mut elite = sorted_population[i].clone();
            elite.age += 1;
            new_population.push(elite);
        }

        // Generate new individuals through crossover and mutation
        while new_population.len() < self.config.population_size {
            let parent1 = self.tournament_selection(&sorted_population);
            let parent2 = self.tournament_selection(&sorted_population);
            let mut offspring = if rng.gen::<f64>() < self.config.crossover_rate {
                self.crossover(&parent1, &parent2)?
            } else {
                parent1.clone()
            };

            if rng.gen::<f64>() < self.config.mutation_rate {
                self.mutate(&mut offspring)?;
            }

            offspring.fitness = None; // Reset fitness for re-evaluation
            offspring.age = 0;
            new_population.push(offspring);
        }

        self.population = new_population;
        Ok(())
    }
    fn tournament_selection(&self, population: &[Individual]) -> Individual {
        let mut rng = thread_rng();
        let tournament_size = 3;

        let mut best_individual = population[rng.gen_range(0..population.len())].clone();
        let mut best_fitness = best_individual
            .fitness
            .as_ref()
            .map(|f| f.combined_score)
            .unwrap_or(0.0);

        for _ in 1..tournament_size {
            let candidate = &population[rng.gen_range(0..population.len())];
            let candidate_fitness = candidate
                .fitness
                .as_ref()
                .map(|f| f.combined_score)
                .unwrap_or(0.0);

            if candidate_fitness > best_fitness {
                best_individual = candidate.clone();
                best_fitness = candidate_fitness;
            }
        }

        best_individual
    }

    fn crossover(&self, parent1: &Individual, parent2: &Individual) -> Result<Individual> {
        let mut rng = thread_rng();
        let mut offspring_params = HashMap::new();

        for key in parent1.parameters.keys() {
            let value = if rng.gen::<f64>() < 0.5 {
                parent1.parameters[key]
            } else {
                parent2.parameters[key]
            };
            offspring_params.insert(key.clone(), value);
        }

        Ok(Individual {
            parameters: offspring_params,
            fitness: None,
            age: 0,
            mutations: 0,
        })
    }

    fn mutate(&self, individual: &mut Individual) -> Result<()> {
        let mut rng = thread_rng();

        for (key, value) in &mut individual.parameters {
            if rng.gen::<f64>() < 0.3 {
                // 30% chance to mutate each parameter
                let mutation_strength = 0.1; // 10% mutation
                let mutation = rng.gen_range(-mutation_strength..=mutation_strength);
                let new_value = *value * (1.0 + mutation);

                // Apply bounds
                *value = match key.as_str() {
                    "stop_loss" => new_value.clamp(0.01, 0.2),
                    "take_profit" => new_value.clamp(0.02, 0.5),
                    "position_size" => new_value.clamp(0.1, 1.0),
                    "rsi_oversold" => new_value.clamp(20.0, 40.0),
                    "rsi_overbought" => new_value.clamp(60.0, 80.0),
                    "volume_threshold" => new_value.clamp(0.5, 5.0),
                    _ => new_value.clamp(0.1, 10.0),
                };
            }
        }

        individual.mutations += 1;
        Ok(())
    }

    /// Get optimization statistics and progress
    pub fn get_optimization_stats(&self) -> OptimizationStats {
        OptimizationStats {
            current_generation: self.generation,
            population_size: self.population.len(),
            best_fitness: self
                .best_individual
                .as_ref()
                .and_then(|ind| ind.fitness.as_ref())
                .map(|f| f.combined_score),
            convergence_progress: self.fitness_history.len() as f64
                / self.config.generations as f64,
            last_optimization: self.last_optimization,
        }
    }

    /// Update strategy optimizer configuration
    pub async fn update_configuration(
        &mut self,
        new_config: StrategyOptimizerConfig,
    ) -> Result<()> {
        self.config = new_config;
        tracing::info!("Strategy optimizer configuration updated");
        Ok(())
    }
}

#[derive(Debug, Clone)]
struct Position {
    entry_time: DateTime<Utc>,
    entry_price: f64,
    quantity: f64,
    trade_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OptimizationStats {
    pub current_generation: u32,
    pub population_size: usize,
    pub best_fitness: Option<f64>,
    pub convergence_progress: f64,
    pub last_optimization: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_strategy_optimizer_creation() {
        let config = StrategyOptimizerConfig {
            population_size: 20,
            generations: 10,
            mutation_rate: 0.1,
            crossover_rate: 0.8,
            fitness_metrics: vec!["sharpe_ratio".to_string()],
        };

        let optimizer = StrategyOptimizer::new(config).await;
        assert!(optimizer.is_ok());
    }

    #[tokio::test]
    async fn test_population_initialization() {
        let config = StrategyOptimizerConfig {
            population_size: 10,
            generations: 5,
            mutation_rate: 0.1,
            crossover_rate: 0.8,
            fitness_metrics: vec!["sharpe_ratio".to_string()],
        };

        let mut optimizer = StrategyOptimizer::new(config).await.unwrap();

        let mut initial_params = HashMap::new();
        initial_params.insert("stop_loss".to_string(), 0.05);
        initial_params.insert("take_profit".to_string(), 0.1);

        let result = optimizer.initialize_population(&initial_params).await;
        assert!(result.is_ok());
        assert_eq!(optimizer.population.len(), 10);
    }

    #[test]
    fn test_fitness_calculation() {
        let config = StrategyOptimizerConfig {
            population_size: 10,
            generations: 5,
            mutation_rate: 0.1,
            crossover_rate: 0.8,
            fitness_metrics: vec!["sharpe_ratio".to_string()],
        };

        let optimizer = StrategyOptimizer {
            config,
            population: Vec::new(),
            generation: 0,
            best_individual: None,
            fitness_history: Vec::new(),
            last_optimization: Utc::now(),
        };

        let backtest_result = BacktestResult {
            total_trades: 100,
            winning_trades: 60,
            losing_trades: 40,
            total_return: 0.15,
            max_drawdown: 0.08,
            sharpe_ratio: 1.5,
            volatility: 0.12,
            trade_details: Vec::new(),
        };

        let fitness = optimizer.calculate_fitness(&backtest_result);
        assert!(fitness.is_ok());

        let fitness_score = fitness.unwrap();
        assert!(fitness_score.combined_score > 0.0);
        assert!(fitness_score.combined_score <= 1.0);
    }
}
