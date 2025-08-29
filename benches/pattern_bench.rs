// benches/pattern_bench.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use patternhunt::predicates::FileType;
use patternhunt::{GlobOptions, GlobOptionsBuilder, PatternHunt, Predicates};
use std::time::{Duration, SystemTime};

fn bench_basic_patterns(c: &mut Criterion) {
    let options = GlobOptions::default();
    let patterns = vec!["*.rs", "*.toml"];
    let roots = vec!["."];

    c.bench_function("basic_patterns", |b| {
        b.iter(|| {
            let result = PatternHunt::sync(
                black_box(&patterns),
                black_box(&roots),
                black_box(options.clone()),
            );
            black_box(result.unwrap())
        })
    });
}

fn bench_complex_patterns(c: &mut Criterion) {
    let options = GlobOptions::default();
    let patterns = vec!["src/**/*.@(rs|toml)", "test/*.{rs,json,toml}"];
    let roots = vec!["."];

    c.bench_function("complex_patterns", |b| {
        b.iter(|| {
            let result = PatternHunt::sync(
                black_box(&patterns),
                black_box(&roots),
                black_box(options.clone()),
            );
            black_box(result.unwrap())
        })
    });
}

fn bench_regex_patterns(c: &mut Criterion) {
    let options = GlobOptions::default();
    let patterns = vec!["re:^[a-z_]+\\.[rs|py]$"];
    let roots = vec!["."];

    c.bench_function("regex_patterns", |b| {
        b.iter(|| {
            let result = PatternHunt::sync(
                black_box(&patterns),
                black_box(&roots),
                black_box(options.clone()),
            );
            black_box(result.unwrap())
        })
    });
}

fn bench_with_predicates(c: &mut Criterion) {
    let predicates = Predicates {
        min_size: Some(1024),
        max_size: Some(1024 * 1024),
        file_type: Some(FileType::File),
        mtime_after: Some(SystemTime::now() - Duration::from_secs(24 * 3600)),
        mtime_before: Some(SystemTime::now()),
        ctime_after: None,
        ctime_before: None,
        follow_symlinks: false,
    };

    let options = GlobOptionsBuilder::new().predicates(predicates).build();

    let patterns = vec!["*.rs", "*.toml"];
    let roots = vec!["."];

    c.bench_function("with_predicates", |b| {
        b.iter(|| {
            let result = PatternHunt::sync(
                black_box(&patterns),
                black_box(&roots),
                black_box(options.clone()),
            );
            black_box(result.unwrap())
        })
    });
}

fn bench_multiple_roots(c: &mut Criterion) {
    let options = GlobOptions::default();
    let patterns = vec!["*.rs", "*.toml"];
    let roots = vec!["src", "tests", "examples"];

    c.bench_function("multiple_roots", |b| {
        b.iter(|| {
            let result = PatternHunt::sync(
                black_box(&patterns),
                black_box(&roots),
                black_box(options.clone()),
            );
            black_box(result.unwrap())
        })
    });
}

#[cfg(feature = "async")]
fn bench_async_search(c: &mut Criterion) {
    use futures::{pin_mut, StreamExt};
    use tokio::runtime::Runtime;

    let options = GlobOptions::default();
    let patterns = vec!["*.rs", "*.toml"];
    let roots = vec!["."];

    c.bench_function("async_search", |b| {
        b.iter(|| {
            let rt = Runtime::new().unwrap();
            rt.block_on(async {
                let stream = PatternHunt::stream(
                    black_box(&patterns),
                    black_box(&roots),
                    black_box(options.clone()),
                )
                .unwrap();

                // Закрепляем поток с помощью pin_mut
                pin_mut!(stream);

                let mut count = 0;
                while let Some(result) = stream.next().await {
                    black_box(result.unwrap());
                    count += 1;
                }
                count
            })
        })
    });
}

// Создаем группу бенчмарков в зависимости от наличия фичи async
#[cfg(not(feature = "async"))]
criterion_group!(
    benches,
    bench_basic_patterns,
    bench_complex_patterns,
    bench_regex_patterns,
    bench_with_predicates,
    bench_multiple_roots
);

#[cfg(feature = "async")]
criterion_group!(
    benches,
    bench_basic_patterns,
    bench_complex_patterns,
    bench_regex_patterns,
    bench_with_predicates,
    bench_multiple_roots,
    bench_async_search
);

criterion_main!(benches);
