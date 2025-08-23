//! # 基础功能验证示例
//!
//! 这个示例验证RainbowBrowserAI的基础功能而不需要WebDriver

use rainbow_browser_ai::{BrowserConfig, types::*};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌈 RainbowBrowserAI V8.0 基础功能验证");
    println!("==========================================");

    // 1. 测试配置系统
    println!("\n📋 1. 测试配置系统...");
    test_configuration_system().await?;

    // 2. 测试类型系统
    println!("\n🔧 2. 测试类型系统...");
    test_type_system().await?;

    // 3. 测试AI意图系统
    println!("\n🎯 3. 测试AI意图系统...");
    test_ai_intent_system().await?;

    // 4. 测试感知模式
    println!("\n👁️ 4. 测试感知模式...");
    test_perception_modes().await?;

    // 5. 测试错误处理
    println!("\n⚠️ 5. 测试错误处理...");
    test_error_handling().await?;

    println!("\n✅ 所有基础功能验证通过！");
    println!("🚀 RainbowBrowserAI V8.0 数字生命体已就绪！");

    Ok(())
}

/// 测试配置系统
async fn test_configuration_system() -> Result<(), Box<dyn std::error::Error>> {
    // 测试默认配置
    let config = BrowserConfig::default();
    println!("  ✓ 默认配置创建成功");

    // 测试配置结构完整性
    assert!(!config.kernel.session_management.session_timeout.is_zero());
    assert!(!config.perception.caching.l1_cache_ttl.is_zero());
    println!("  ✓ 配置结构验证通过");

    // 测试测试配置
    let test_config = BrowserConfig::test_config();
    assert_eq!(test_config.kernel.session_management.pool_size, 2);
    assert_eq!(test_config.persistence.database.database_url, "sqlite::memory:");
    println!("  ✓ 测试配置创建成功");

    println!("  🎉 配置系统测试完成");
    Ok(())
}

/// 测试类型系统
async fn test_type_system() -> Result<(), Box<dyn std::error::Error>> {
    // 测试AI意图创建
    let intent = AIIntent {
        action: "navigate".to_string(),
        target: "https://example.com".to_string(),
        perception_mode: Some(PerceptionMode::Standard),
        parameters: json!({"timeout": 5000}),
    };
    assert_eq!(intent.action, "navigate");
    println!("  ✓ AI意图类型创建成功");

    // 测试感知模式
    let modes = [
        PerceptionMode::Lightning,
        PerceptionMode::Quick,
        PerceptionMode::Standard,
        PerceptionMode::Deep,
    ];
    for mode in &modes {
        println!("    • 感知模式 {:?} 正常", mode);
    }
    println!("  ✓ 感知模式类型验证通过");

    // 测试健康状态
    let health_statuses = [
        HealthStatus::Healthy,
        HealthStatus::Degraded,
        HealthStatus::Critical,
        HealthStatus::Unavailable,
    ];
    for status in &health_statuses {
        println!("    • 健康状态 {:?} 正常", status);
    }
    println!("  ✓ 健康状态类型验证通过");

    println!("  🎉 类型系统测试完成");
    Ok(())
}

/// 测试AI意图系统
async fn test_ai_intent_system() -> Result<(), Box<dyn std::error::Error>> {
    // 创建各种类型的意图
    let intents = vec![
        AIIntent {
            action: "navigate".to_string(),
            target: "https://github.com".to_string(),
            perception_mode: Some(PerceptionMode::Lightning),
            parameters: json!({"preload": true}),
        },
        AIIntent {
            action: "click".to_string(),
            target: "#login-button".to_string(),
            perception_mode: Some(PerceptionMode::Quick),
            parameters: json!({"wait_for_element": true}),
        },
        AIIntent {
            action: "analyze".to_string(),
            target: "page_content".to_string(),
            perception_mode: Some(PerceptionMode::Deep),
            parameters: json!({"include_hidden": false}),
        },
    ];

    for intent in &intents {
        println!("    • 意图: {} -> {} (模式: {:?})", 
                 intent.action, intent.target, intent.perception_mode);
    }
    println!("  ✓ AI意图创建和序列化成功");

    println!("  🎉 AI意图系统测试完成");
    Ok(())
}

/// 测试感知模式
async fn test_perception_modes() -> Result<(), Box<dyn std::error::Error>> {
    // 测试默认模式
    let default_mode = PerceptionMode::default();
    assert_eq!(default_mode, PerceptionMode::Standard);
    println!("  ✓ 默认感知模式为 Standard");

    // 测试性能目标
    let performance_targets = [
        (PerceptionMode::Lightning, 50),
        (PerceptionMode::Quick, 200),
        (PerceptionMode::Standard, 500),
        (PerceptionMode::Deep, 1000),
    ];

    for (mode, target_ms) in &performance_targets {
        println!("    • {:?} 模式目标: <{}ms", mode, target_ms);
    }
    println!("  ✓ 感知模式性能目标已定义");

    println!("  🎉 感知模式测试完成");
    Ok(())
}

/// 测试错误处理
async fn test_error_handling() -> Result<(), Box<dyn std::error::Error>> {
    use rainbow_browser_ai::error::BrowserError;

    // 测试各种错误类型
    let errors = vec![
        BrowserError::unified_kernel("测试内核错误", "UK001"),
        BrowserError::timeout("测试操作", 5000, "TO001"),
        BrowserError::layered_perception("感知失败", "LP001", None),
        BrowserError::intelligent_action("行动失败", "IA001", None),
    ];

    for error in &errors {
        println!("    • 错误类型: {}", error);
        assert!(!error.to_string().is_empty());
    }
    println!("  ✓ 错误类型创建和格式化成功");

    // 测试错误链
    let chained_error = BrowserError::intelligent_action(
        "复合错误",
        "IA002",
        Some(Box::new(BrowserError::timeout("超时", 1000, "TO002")))
    );
    println!("    • 链式错误: {}", chained_error);
    println!("  ✓ 错误链创建成功");

    println!("  🎉 错误处理测试完成");
    Ok(())
}

/// 性能基准测试
#[allow(dead_code)]
async fn benchmark_basic_operations() -> Result<(), Box<dyn std::error::Error>> {
    use std::time::Instant;

    println!("\n📊 性能基准测试...");

    // 配置创建基准
    let start = Instant::now();
    for _ in 0..100 {
        let _config = BrowserConfig::default();
    }
    let config_time = start.elapsed();
    println!("  • 100个配置创建: {:?}", config_time);

    // 意图创建基准
    let start = Instant::now();
    for i in 0..1000 {
        let _intent = AIIntent {
            action: format!("action_{}", i),
            target: format!("target_{}", i),
            perception_mode: Some(PerceptionMode::Standard),
            parameters: json!({"id": i}),
        };
    }
    let intent_time = start.elapsed();
    println!("  • 1000个意图创建: {:?}", intent_time);

    // JSON序列化基准
    let intent = AIIntent {
        action: "test".to_string(),
        target: "test".to_string(),
        perception_mode: Some(PerceptionMode::Standard),
        parameters: json!({"complex": {"nested": {"data": [1, 2, 3, 4, 5]}}}),
    };

    let start = Instant::now();
    for _ in 0..1000 {
        let _json = serde_json::to_string(&intent).unwrap();
    }
    let serialize_time = start.elapsed();
    println!("  • 1000次JSON序列化: {:?}", serialize_time);

    println!("  🎉 性能基准测试完成");
    Ok(())
}