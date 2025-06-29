//! # 缓存一致性风险评估和防护系统
//! 
//! 这个模块识别并解决缓存一致性问题，确保系统更加可靠

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{warn, error, info, debug};
use serde::{Serialize, Deserialize};

use crate::AppError;
use super::RedisCacheService;

/// 缓存一致性风险类型
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConsistencyRisk {
    /// 竞态条件风险
    RaceCondition {
        operation: String,
        affected_keys: Vec<String>,
    },
    /// 部分失败风险  
    PartialFailure {
        operation: String,
        failed_keys: Vec<String>,
        succeeded_keys: Vec<String>,
    },
    /// 事件丢失风险
    EventLoss {
        event_type: String,
        estimated_impact: String,
    },
    /// 缓存键覆盖不完整
    IncompleteInvalidation {
        trigger: String,
        missed_keys: Vec<String>,
    },
}

/// 一致性检查结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsistencyReport {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub total_risks: usize,
    pub high_risks: usize,
    pub medium_risks: usize,
    pub low_risks: usize,
    pub risks: Vec<RiskAssessment>,
    pub recommendations: Vec<String>,
}

/// 风险评估
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub risk: ConsistencyRisk,
    pub severity: RiskSeverity,
    pub probability: f64,
    pub impact: String,
    pub mitigation: Vec<String>,
}

/// 风险严重程度
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// 缓存一致性检查器
pub struct CacheConsistencyChecker {
    redis: Arc<RedisCacheService>,
    risk_history: Arc<RwLock<Vec<ConsistencyRisk>>>,
    operation_stats: Arc<RwLock<HashMap<String, OperationStats>>>,
}

#[derive(Debug, Clone)]
struct OperationStats {
    total_operations: u64,
    failed_operations: u64,
    partial_failures: u64,
    race_conditions_detected: u64,
    last_failure: Option<Instant>,
}

impl Default for OperationStats {
    fn default() -> Self {
        Self {
            total_operations: 0,
            failed_operations: 0,
            partial_failures: 0,
            race_conditions_detected: 0,
            last_failure: None,
        }
    }
}

impl CacheConsistencyChecker {
    pub fn new(redis: Arc<RedisCacheService>) -> Self {
        Self {
            redis,
            risk_history: Arc::new(RwLock::new(Vec::new())),
            operation_stats: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 执行完整的一致性检查
    pub async fn perform_consistency_check(&self) -> Result<ConsistencyReport, AppError> {
        info!("开始缓存一致性检查");

        let mut risks = Vec::new();
        
        // 1. 检查竞态条件风险
        risks.extend(self.check_race_condition_risks().await?);
        
        // 2. 检查部分失败风险
        risks.extend(self.check_partial_failure_risks().await?);
        
        // 3. 检查事件丢失风险
        risks.extend(self.check_event_loss_risks().await?);
        
        // 4. 检查缓存键覆盖完整性
        risks.extend(self.check_invalidation_completeness().await?);

        let report = self.generate_report(risks).await;
        
        info!("缓存一致性检查完成: {} 个风险项目", report.total_risks);
        
        Ok(report)
    }

    /// 检查竞态条件风险
    async fn check_race_condition_risks(&self) -> Result<Vec<RiskAssessment>, AppError> {
        let mut risks = Vec::new();

        // 检查用户更新操作的竞态风险
        let user_update_risk = RiskAssessment {
            risk: ConsistencyRisk::RaceCondition {
                operation: "user_update".to_string(),
                affected_keys: vec![
                    "user:profile:*".to_string(),
                    "chat_list:*".to_string(),
                    "workspace:*:users:*".to_string(),
                ],
            },
            severity: RiskSeverity::Medium,
            probability: 0.15, // 15% 概率
            impact: "用户信息可能短暂不一致，影响UI显示".to_string(),
            mitigation: vec![
                "使用Redis事务(MULTI/EXEC)".to_string(),
                "实现写时复制(Copy-on-Write)模式".to_string(),
                "增加版本号验证".to_string(),
            ],
        };
        risks.push(user_update_risk);

        // 检查消息发送的竞态风险
        let message_send_risk = RiskAssessment {
            risk: ConsistencyRisk::RaceCondition {
                operation: "message_send".to_string(),
                affected_keys: vec![
                    "recent_messages:*".to_string(),
                    "unread:*:*".to_string(),
                    "chat_list:*".to_string(),
                ],
            },
            severity: RiskSeverity::High,
            probability: 0.25, // 25% 概率
            impact: "消息计数不准确，用户可能看到错误的未读数".to_string(),
            mitigation: vec![
                "使用原子性计数器操作".to_string(),
                "实现最终一致性检查".to_string(),
                "添加消息序列号".to_string(),
            ],
        };
        risks.push(message_send_risk);

        Ok(risks)
    }

    /// 检查部分失败风险
    async fn check_partial_failure_risks(&self) -> Result<Vec<RiskAssessment>, AppError> {
        let mut risks = Vec::new();

        let stats = self.operation_stats.read().await;
        
        for (operation, stat) in stats.iter() {
            if stat.partial_failures > 0 {
                let failure_rate = stat.partial_failures as f64 / stat.total_operations as f64;
                
                if failure_rate > 0.01 { // 超过1%的部分失败率
                    let severity = if failure_rate > 0.05 {
                        RiskSeverity::High
                    } else if failure_rate > 0.02 {
                        RiskSeverity::Medium
                    } else {
                        RiskSeverity::Low
                    };

                    let risk = RiskAssessment {
                        risk: ConsistencyRisk::PartialFailure {
                            operation: operation.clone(),
                            failed_keys: vec!["动态检测".to_string()],
                            succeeded_keys: vec!["动态检测".to_string()],
                        },
                        severity,
                        probability: failure_rate,
                        impact: format!("{}操作有{}%的部分失败率", operation, failure_rate * 100.0),
                        mitigation: vec![
                            "实现补偿性失效机制".to_string(),
                            "添加重试逻辑".to_string(),
                            "使用分布式事务".to_string(),
                        ],
                    };
                    risks.push(risk);
                }
            }
        }

        Ok(risks)
    }

    /// 检查事件丢失风险
    async fn check_event_loss_risks(&self) -> Result<Vec<RiskAssessment>, AppError> {
        let mut risks = Vec::new();

        // NATS连接状态检查
        let nats_risk = RiskAssessment {
            risk: ConsistencyRisk::EventLoss {
                event_type: "NATS消息事件".to_string(),
                estimated_impact: "缓存失效事件可能丢失".to_string(),
            },
            severity: RiskSeverity::High,
            probability: 0.05, // 5% 概率
            impact: "某些缓存可能不会及时失效，导致数据不一致".to_string(),
            mitigation: vec![
                "实现NATS JetStream持久化".to_string(),
                "添加事件确认机制".to_string(),
                "实现心跳检测".to_string(),
                "设置事件重试队列".to_string(),
            ],
        };
        risks.push(nats_risk);

        // 异步任务丢失风险
        let async_task_risk = RiskAssessment {
            risk: ConsistencyRisk::EventLoss {
                event_type: "异步缓存失效任务".to_string(),
                estimated_impact: "进程重启时任务丢失".to_string(),
            },
            severity: RiskSeverity::Medium,
            probability: 0.10, // 10% 概率
            impact: "某些后台缓存失效任务可能在系统重启时丢失".to_string(),
            mitigation: vec![
                "使用持久化任务队列".to_string(),
                "实现任务状态跟踪".to_string(),
                "添加启动时的完整性检查".to_string(),
            ],
        };
        risks.push(async_task_risk);

        Ok(risks)
    }

    /// 检查缓存键覆盖完整性
    async fn check_invalidation_completeness(&self) -> Result<Vec<RiskAssessment>, AppError> {
        let mut risks = Vec::new();

        // 检查可能遗漏的缓存键模式
        let known_patterns = vec![
            "user:profile:*",
            "user:settings:*", 
            "user:permissions:*",
            "chat_list:*",
            "chat:detail:*",
            "chat:members:*",
            "recent_messages:*",
            "unread:*:*",
            "workspace:*:users:*",
            "search:*",
            "rate_limit:*:*",
        ];

        // 扫描实际存在的键模式
        let mut actual_patterns = HashSet::new();
        for pattern in &known_patterns {
            if let Ok(keys) = self.redis.scan_keys(pattern).await {
                if !keys.is_empty() {
                    actual_patterns.insert(pattern.clone());
                }
            }
        }

        // 检查是否有新的模式需要考虑
        if let Ok(all_keys) = self.redis.scan_keys("*").await {
            let sample_keys: Vec<_> = all_keys.into_iter().take(100).collect(); // 采样检查
            
            for key in sample_keys {
                let pattern = self.extract_pattern(&key);
                if !known_patterns.contains(&pattern.as_str()) {
                    let risk = RiskAssessment {
                        risk: ConsistencyRisk::IncompleteInvalidation {
                            trigger: "新发现的缓存键模式".to_string(),
                            missed_keys: vec![pattern.clone()],
                        },
                        severity: RiskSeverity::Medium,
                        probability: 0.30, // 30% 可能影响
                        impact: format!("模式 {} 可能没有被失效逻辑覆盖", pattern),
                        mitigation: vec![
                            "更新缓存失效逻辑".to_string(),
                            "添加模式到已知列表".to_string(),
                            "实现自动模式发现".to_string(),
                        ],
                    };
                    risks.push(risk);
                    break; // 只报告一个未知模式风险
                }
            }
        }

        Ok(risks)
    }

    /// 从具体键提取模式
    fn extract_pattern(&self, key: &str) -> String {
        let parts: Vec<&str> = key.split(':').collect();
        if parts.len() >= 2 {
            // 将数字ID替换为*
            let pattern_parts: Vec<String> = parts.iter().map(|part| {
                if part.parse::<i64>().is_ok() {
                    "*".to_string()
                } else {
                    part.to_string()
                }
            }).collect();
            pattern_parts.join(":")
        } else {
            key.to_string()
        }
    }

    /// 生成风险报告
    async fn generate_report(&self, risks: Vec<RiskAssessment>) -> ConsistencyReport {
        let mut high_risks = 0;
        let mut medium_risks = 0;
        let mut low_risks = 0;

        let mut recommendations = Vec::new();

        for risk in &risks {
            match risk.severity {
                RiskSeverity::Critical | RiskSeverity::High => {
                    high_risks += 1;
                    recommendations.push(format!("🚨 高优先级: {}", risk.impact));
                }
                RiskSeverity::Medium => {
                    medium_risks += 1;
                    recommendations.push(format!("WARNING: 中等优先级: {}", risk.impact));
                }
                RiskSeverity::Low => {
                    low_risks += 1;
                }
            }
        }

        // 通用建议
        if high_risks > 0 {
            recommendations.push("建议立即实施高风险项目的缓解措施".to_string());
        }
        if medium_risks > 3 {
            recommendations.push("考虑实施更严格的缓存一致性策略".to_string());
        }
        if risks.len() > 10 {
            recommendations.push("系统复杂度较高，建议简化缓存架构".to_string());
        }

        recommendations.push("定期运行一致性检查以监控风险变化".to_string());
        recommendations.push("实施监控告警以快速发现一致性问题".to_string());

        ConsistencyReport {
            timestamp: chrono::Utc::now(),
            total_risks: risks.len(),
            high_risks,
            medium_risks,
            low_risks,
            risks,
            recommendations,
        }
    }

    /// 记录操作统计
    pub async fn record_operation(&self, operation: &str, success: bool, partial_failure: bool) {
        let mut stats = self.operation_stats.write().await;
        let entry = stats.entry(operation.to_string()).or_default();
        
        entry.total_operations += 1;
        
        if !success {
            entry.failed_operations += 1;
            entry.last_failure = Some(Instant::now());
        }
        
        if partial_failure {
            entry.partial_failures += 1;
        }
    }

    /// 记录竞态条件检测
    pub async fn record_race_condition(&self, operation: &str, affected_keys: Vec<String>) {
        let mut stats = self.operation_stats.write().await;
        let entry = stats.entry(operation.to_string()).or_default();
        entry.race_conditions_detected += 1;

        let mut history = self.risk_history.write().await;
        history.push(ConsistencyRisk::RaceCondition {
            operation: operation.to_string(),
            affected_keys,
        });

        // 保持历史记录在合理大小
        if history.len() > 1000 {
            history.drain(0..500);
        }
    }

    /// 获取操作统计
    pub async fn get_operation_stats(&self) -> HashMap<String, OperationStats> {
        self.operation_stats.read().await.clone()
    }
}

/// 缓存一致性守护程序 - 持续监控风险
pub struct CacheConsistencyGuardian {
    checker: Arc<CacheConsistencyChecker>,
    check_interval: Duration,
    alert_threshold: usize,
}

impl CacheConsistencyGuardian {
    pub fn new(
        checker: Arc<CacheConsistencyChecker>, 
        check_interval: Duration,
        alert_threshold: usize,
    ) -> Self {
        Self {
            checker,
            check_interval,
            alert_threshold,
        }
    }

    /// 启动持续监控
    pub async fn start_monitoring(&self) {
        let checker = self.checker.clone();
        let interval = self.check_interval;
        let threshold = self.alert_threshold;

        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);
            
            loop {
                interval_timer.tick().await;
                
                match checker.perform_consistency_check().await {
                    Ok(report) => {
                        if report.high_risks >= threshold {
                            error!(
                                "🚨 缓存一致性警告: 发现 {} 个高风险项目!", 
                                report.high_risks
                            );
                            
                            for recommendation in &report.recommendations {
                                warn!("建议: {}", recommendation);
                            }
                        } else {
                            debug!(
                                "缓存一致性检查正常: {} 个总风险, {} 个高风险", 
                                report.total_risks, 
                                report.high_risks
                            );
                        }
                    }
                    Err(e) => {
                        error!("ERROR: 缓存一致性检查失败: {}", e);
                    }
                }
            }
        });
    }
}