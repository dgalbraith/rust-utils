use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rust_utils::fs::should_exclude;
use std::path::Path;

fn bench_pattern_matching(c: &mut Criterion) {
    // Test simple pattern matching using the public API
    c.bench_function("should_exclude_log_files", |b| {
        let patterns = vec!["*.log".to_string()];
        b.iter(|| should_exclude(black_box(Path::new("file.log")), black_box(&patterns)))
    });

    c.bench_function("should_exclude_complex_path", |b| {
        let patterns = vec!["var/log/*".to_string()];
        b.iter(|| {
            should_exclude(
                black_box(Path::new("var/log/app/error.log")),
                black_box(&patterns),
            )
        })
    });

    c.bench_function("should_exclude_no_match", |b| {
        let patterns = vec!["*.log".to_string()];
        b.iter(|| should_exclude(black_box(Path::new("src/main.rs")), black_box(&patterns)))
    });
}

fn bench_exclusion_checking(c: &mut Criterion) {
    let patterns = vec![
        "*.log".to_string(),
        "tmp/*".to_string(),
        "var/cache/*".to_string(),
        "*.sock".to_string(),
        "run/*".to_string(),
    ];

    c.bench_function("should_exclude_match", |b| {
        b.iter(|| should_exclude(black_box(Path::new("app.log")), black_box(&patterns)))
    });

    c.bench_function("should_exclude_no_match", |b| {
        b.iter(|| should_exclude(black_box(Path::new("src/main.rs")), black_box(&patterns)))
    });
}

criterion_group!(benches, bench_pattern_matching, bench_exclusion_checking);
criterion_main!(benches);
