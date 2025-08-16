use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use rainbow_poc::{Cache, Config, SimpleBrowser, BrowserPool, WorkflowEngine, Workflow};
use std::time::Duration;
use tokio::runtime::Runtime;

fn benchmark_cache_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("cache");
    
    // Benchmark cache insertions
    group.bench_function("insert", |b| {
        b.iter(|| {
            rt.block_on(async {
                let cache: Cache<i32, String> = Cache::new(Duration::from_secs(60), 1000);
                for i in 0..100 {
                    cache.insert(black_box(i), format!("value_{}", i)).await;
                }
            });
        });
    });
    
    // Benchmark cache lookups
    group.bench_function("lookup_hit", |b| {
        let cache: Cache<i32, String> = Cache::new(Duration::from_secs(60), 1000);
        rt.block_on(async {
            for i in 0..100 {
                cache.insert(i, format!("value_{}", i)).await;
            }
        });
        
        b.iter(|| {
            rt.block_on(async {
                for i in 0..100 {
                    let _ = cache.get(&black_box(i)).await;
                }
            });
        });
    });
    
    // Benchmark cache misses
    group.bench_function("lookup_miss", |b| {
        let cache: Cache<i32, String> = Cache::new(Duration::from_secs(60), 1000);
        
        b.iter(|| {
            rt.block_on(async {
                for i in 1000..1100 {
                    let _ = cache.get(&black_box(i)).await;
                }
            });
        });
    });
    
    // Benchmark with different cache sizes
    for size in [100, 1000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::new("sized_insert", size),
            size,
            |b, &size| {
                b.iter(|| {
                    rt.block_on(async {
                        let cache: Cache<i32, String> = Cache::new(Duration::from_secs(60), size);
                        for i in 0..size/10 {
                            cache.insert(black_box(i as i32), format!("value_{}", i)).await;
                        }
                    });
                });
            },
        );
    }
    
    group.finish();
}

fn benchmark_config_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("config");
    
    // Benchmark config creation
    group.bench_function("default", |b| {
        b.iter(|| {
            let _ = Config::default();
        });
    });
    
    // Benchmark config validation
    group.bench_function("validate", |b| {
        let config = Config::default();
        b.iter(|| {
            let _ = config.validate();
        });
    });
    
    // Benchmark config serialization
    group.bench_function("serialize_json", |b| {
        let config = Config::default();
        b.iter(|| {
            let _ = serde_json::to_string(&config).unwrap();
        });
    });
    
    group.bench_function("serialize_yaml", |b| {
        let config = Config::default();
        b.iter(|| {
            let _ = serde_yaml::to_string(&config).unwrap();
        });
    });
    
    // Benchmark config deserialization
    let json_str = serde_json::to_string(&Config::default()).unwrap();
    group.bench_function("deserialize_json", |b| {
        b.iter(|| {
            let _: Config = serde_json::from_str(black_box(&json_str)).unwrap();
        });
    });
    
    let yaml_str = serde_yaml::to_string(&Config::default()).unwrap();
    group.bench_function("deserialize_yaml", |b| {
        b.iter(|| {
            let _: Config = serde_yaml::from_str(black_box(&yaml_str)).unwrap();
        });
    });
    
    group.finish();
}

fn benchmark_workflow_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("workflow");
    
    // Simple workflow
    let simple_workflow = r#"
name: Simple Workflow
description: Basic test
steps:
  - name: Navigate
    action:
      type: navigate
      url: https://example.com
"#;
    
    // Complex workflow with multiple steps
    let complex_workflow = r#"
name: Complex Workflow
description: Multi-step workflow
inputs:
  - name: url
    input_type: string
    required: true
  - name: username
    input_type: string
    required: true
steps:
  - name: Navigate
    action:
      type: navigate
      url: "{{url}}"
  - name: Wait
    action:
      type: wait
      wait_for: element
      selector: "#login"
  - name: Fill Username
    action:
      type: fill
      selector: "#username"
      value: "{{username}}"
  - name: Click Submit
    action:
      type: click
      selector: "#submit"
  - name: Extract Result
    action:
      type: extract
      selector: ".result"
      attribute: text
    store_as: result
"#;
    
    group.bench_function("parse_simple_yaml", |b| {
        b.iter(|| {
            let _: Workflow = serde_yaml::from_str(black_box(simple_workflow)).unwrap();
        });
    });
    
    group.bench_function("parse_complex_yaml", |b| {
        b.iter(|| {
            let _: Workflow = serde_yaml::from_str(black_box(complex_workflow)).unwrap();
        });
    });
    
    // JSON versions
    let simple_json = serde_json::to_string(&serde_yaml::from_str::<Workflow>(simple_workflow).unwrap()).unwrap();
    let complex_json = serde_json::to_string(&serde_yaml::from_str::<Workflow>(complex_workflow).unwrap()).unwrap();
    
    group.bench_function("parse_simple_json", |b| {
        b.iter(|| {
            let _: Workflow = serde_json::from_str(black_box(&simple_json)).unwrap();
        });
    });
    
    group.bench_function("parse_complex_json", |b| {
        b.iter(|| {
            let _: Workflow = serde_json::from_str(black_box(&complex_json)).unwrap();
        });
    });
    
    group.finish();
}

fn benchmark_string_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("string");
    
    // Template rendering
    let engine = WorkflowEngine::new();
    let template = "Hello {{name}}, you are visiting {{url}} at {{time}}";
    
    rt.block_on(async {
        engine.set_variable("name", serde_json::json!("User"));
        engine.set_variable("url", serde_json::json!("https://example.com"));
        engine.set_variable("time", serde_json::json!("2024-01-01"));
    });
    
    group.bench_function("template_render", |b| {
        b.iter(|| {
            rt.block_on(async {
                let _ = engine.render_template(black_box(template)).unwrap();
            });
        });
    });
    
    // URL sanitization
    let urls = vec![
        "https://example.com",
        "https://example.com/path/to/page?query=value&other=123",
        "https://user:pass@example.com:8080/path",
    ];
    
    group.bench_function("url_parse", |b| {
        b.iter(|| {
            for url in &urls {
                let _ = url::Url::parse(black_box(url)).unwrap();
            }
        });
    });
    
    group.finish();
}

fn benchmark_browser_pool(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("browser_pool");
    
    // Skip these benchmarks in CI as they require ChromeDriver
    if std::env::var("CI").is_ok() {
        group.finish();
        return;
    }
    
    let pool = BrowserPool::with_config(
        3,
        Duration::from_secs(300),
        Duration::from_secs(3600),
        100
    );
    
    group.bench_function("acquire_release", |b| {
        b.iter(|| {
            rt.block_on(async {
                let handle = pool.acquire().await.unwrap();
                handle.release().await;
            });
        });
    });
    
    group.finish();
}

fn benchmark_metrics_collection(c: &mut Criterion) {
    use rainbow_poc::MetricsCollector;
    
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("metrics");
    
    let collector = MetricsCollector::new();
    
    group.bench_function("record_operation", |b| {
        b.iter(|| {
            rt.block_on(async {
                collector.record_operation(
                    Duration::from_millis(100),
                    true,
                    0.01
                ).await;
            });
        });
    });
    
    group.bench_function("get_metrics", |b| {
        // Pre-populate with some data
        rt.block_on(async {
            for _ in 0..100 {
                collector.record_operation(
                    Duration::from_millis(100),
                    true,
                    0.01
                ).await;
            }
        });
        
        b.iter(|| {
            rt.block_on(async {
                let _ = collector.get_metrics().await;
            });
        });
    });
    
    group.bench_function("calculate_percentiles", |b| {
        let metrics = rt.block_on(async {
            for i in 0..1000 {
                collector.record_operation(
                    Duration::from_millis(i),
                    true,
                    0.01
                ).await;
            }
            collector.get_metrics().await
        });
        
        b.iter(|| {
            let _ = metrics.operation_duration_percentiles();
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_cache_operations,
    benchmark_config_operations,
    benchmark_workflow_parsing,
    benchmark_string_operations,
    benchmark_browser_pool,
    benchmark_metrics_collection
);

criterion_main!(benches);