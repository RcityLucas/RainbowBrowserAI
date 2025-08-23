use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use rainbow_poc::{
    SimpleBrowser, BrowserPool, WorkflowEngine, Workflow,
    LLMCache, WorkflowCache, MetricsCollector, CostTracker,
};
use std::time::Duration;
use tokio::runtime::Runtime;

fn benchmark_browser_navigation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    c.bench_function("browser_navigation", |b| {
        b.to_async(&rt).iter(|| async {
            let browser = SimpleBrowser::new().await.unwrap();
            browser.navigate_to("https://www.example.com").await.unwrap();
            black_box(browser.get_title().await.unwrap());
        });
    });
}

fn benchmark_browser_pool(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("browser_pool");
    
    for pool_size in [1, 2, 5].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(pool_size),
            pool_size,
            |b, &_size| {
                b.to_async(&rt).iter(|| async {
                    let pool = BrowserPool::new();
                    
                    // Acquire and release multiple browsers
                    for _ in 0..3 {
                        let handle = pool.acquire().await.unwrap();
                        if let Some(browser) = handle.browser() {
                            browser.navigate_to("https://www.example.com").await.unwrap();
                        }
                        drop(handle);
                    }
                });
            },
        );
    }
    
    group.finish();
}

fn benchmark_workflow_execution(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let simple_workflow = Workflow::from_yaml(r#"
name: simple-workflow
steps:
  - name: wait
    action:
      type: wait
      wait_for: time
      seconds: 0.1
"#).unwrap();

    let complex_workflow = Workflow::from_yaml(r#"
name: complex-workflow
variables:
  test_var: "value"
steps:
  - name: wait1
    action:
      type: wait
      wait_for: time
      seconds: 0.1
  - name: wait2
    action:
      type: wait
      wait_for: time
      seconds: 0.1
  - name: conditional
    action:
      type: conditional
      if:
        check: variable_equals
        var: test_var
        value: "value"
      then:
        - name: wait3
          action:
            type: wait
            wait_for: time
            seconds: 0.1
"#).unwrap();

    let mut group = c.benchmark_group("workflow_execution");
    
    group.bench_function("simple_workflow", |b| {
        b.to_async(&rt).iter(|| async {
            let mut engine = WorkflowEngine::new_simple();
            let result = engine.execute(&simple_workflow).await.unwrap();
            black_box(result);
        });
    });
    
    group.bench_function("complex_workflow", |b| {
        b.to_async(&rt).iter(|| async {
            let mut engine = WorkflowEngine::new_simple();
            let result = engine.execute(&complex_workflow).await.unwrap();
            black_box(result);
        });
    });
    
    group.finish();
}

fn benchmark_cache_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("cache_operations");
    
    // LLM Cache benchmarks
    group.bench_function("llm_cache_insert", |b| {
        b.to_async(&rt).iter(|| async {
            let cache = LLMCache::new();
            for i in 0..100 {
                cache.insert(
                    &format!("prompt_{}", i),
                    "gpt-3.5-turbo",
                    serde_json::json!({"response": format!("response_{}", i)}),
                ).await;
            }
        });
    });
    
    group.bench_function("llm_cache_get", |b| {
        let cache = LLMCache::new();
        rt.block_on(async {
            for i in 0..100 {
                cache.insert(
                    &format!("prompt_{}", i),
                    "gpt-3.5-turbo",
                    serde_json::json!({"response": format!("response_{}", i)}),
                ).await;
            }
        });
        
        b.to_async(&rt).iter(|| async {
            for i in 0..100 {
                let result = cache.get(&format!("prompt_{}", i), "gpt-3.5-turbo").await;
                black_box(result);
            }
        });
    });
    
    // Workflow Cache benchmarks
    group.bench_function("workflow_cache_insert", |b| {
        b.to_async(&rt).iter(|| async {
            let cache = WorkflowCache::new();
            let workflow = Workflow::from_yaml(r#"
name: test-workflow
steps:
  - name: wait
    action:
      type: wait
      wait_for: time
      seconds: 1
"#).unwrap();
            
            for i in 0..50 {
                cache.insert(&format!("workflow_{}", i), workflow.clone()).await;
            }
        });
    });
    
    group.bench_function("workflow_cache_get", |b| {
        let cache = WorkflowCache::new();
        let workflow = Workflow::from_yaml(r#"
name: test-workflow
steps:
  - name: wait
    action:
      type: wait
      wait_for: time
      seconds: 1
"#).unwrap();
        
        rt.block_on(async {
            for i in 0..50 {
                cache.insert(&format!("workflow_{}", i), workflow.clone()).await;
            }
        });
        
        b.to_async(&rt).iter(|| async {
            for i in 0..50 {
                let result = cache.get(&format!("workflow_{}", i)).await;
                black_box(result);
            }
        });
    });
    
    group.finish();
}

fn benchmark_metrics_collection(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("metrics_collection");
    
    group.bench_function("record_operation", |b| {
        b.to_async(&rt).iter(|| async {
            let metrics = MetricsCollector::new();
            for _ in 0..100 {
                metrics.record_operation(
                    Duration::from_millis(10),
                    true,
                    0.001,
                ).await;
            }
        });
    });
    
    group.bench_function("get_metrics", |b| {
        let metrics = MetricsCollector::new();
        rt.block_on(async {
            for _ in 0..100 {
                metrics.record_operation(
                    Duration::from_millis(10),
                    true,
                    0.001,
                ).await;
            }
        });
        
        b.to_async(&rt).iter(|| async {
            let result = metrics.get_metrics().await;
            black_box(result);
        });
    });
    
    group.bench_function("export_prometheus", |b| {
        let metrics = MetricsCollector::new();
        rt.block_on(async {
            for _ in 0..100 {
                metrics.record_operation(
                    Duration::from_millis(10),
                    true,
                    0.001,
                ).await;
            }
        });
        
        b.to_async(&rt).iter(|| async {
            let result = metrics.export_prometheus().await;
            black_box(result);
        });
    });
    
    group.finish();
}

fn benchmark_cost_tracking(c: &mut Criterion) {
    let mut group = c.benchmark_group("cost_tracking");
    
    group.bench_function("record_operation", |b| {
        b.iter(|| {
            let mut tracker = CostTracker::new(100.0);
            for i in 0..100 {
                tracker.record_operation(
                    "browser".to_string(),
                    format!("operation_{}", i),
                    0.001,
                    true,
                ).unwrap();
            }
        });
    });
    
    group.bench_function("generate_report", |b| {
        let mut tracker = CostTracker::new(100.0);
        for i in 0..100 {
            tracker.record_operation(
                "browser".to_string(),
                format!("operation_{}", i),
                0.001,
                true,
            ).unwrap();
        }
        
        b.iter(|| {
            let report = tracker.generate_daily_report();
            black_box(report);
        });
    });
    
    group.finish();
}

fn benchmark_workflow_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("workflow_parsing");
    
    let simple_yaml = r#"
name: simple-workflow
steps:
  - name: navigate
    action:
      type: navigate
      url: https://example.com
"#;

    let complex_yaml = r#"
name: complex-workflow
description: Complex workflow with many features
variables:
  var1: value1
  var2: value2
  var3: value3
on_error: continue
steps:
  - name: step1
    action:
      type: navigate
      url: https://example.com
  - name: step2
    action:
      type: wait
      wait_for: element
      selector: "#content"
      timeout: 10
  - name: step3
    action:
      type: click
      selector: "button.submit"
  - name: conditional
    action:
      type: conditional
      if:
        check: variable_equals
        var: var1
        value: value1
      then:
        - name: then1
          action:
            type: wait
            wait_for: time
            seconds: 1
        - name: then2
          action:
            type: screenshot
            filename: test.png
      else:
        - name: else1
          action:
            type: wait
            wait_for: time
            seconds: 2
  - name: loop
    action:
      type: loop
      times: 5
      steps:
        - name: loop_step
          action:
            type: wait
            wait_for: time
            seconds: 0.1
"#;
    
    group.bench_function("parse_simple_workflow", |b| {
        b.iter(|| {
            let workflow = Workflow::from_yaml(simple_yaml).unwrap();
            black_box(workflow);
        });
    });
    
    group.bench_function("parse_complex_workflow", |b| {
        b.iter(|| {
            let workflow = Workflow::from_yaml(complex_yaml).unwrap();
            black_box(workflow);
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_browser_navigation,
    benchmark_browser_pool,
    benchmark_workflow_execution,
    benchmark_cache_operations,
    benchmark_metrics_collection,
    benchmark_cost_tracking,
    benchmark_workflow_parsing
);

criterion_main!(benches);