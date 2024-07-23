/// a criterion benchmark that compares the performance of counting the number
/// of strings in an array that match a particular prefix between GermanString and a regular rust String
use criterion::{criterion_group, criterion_main, Criterion};
use german_string::GermanString;
use pprof::criterion::{Output, PProfProfiler};

fn scan_prefix_long(c: &mut Criterion) {
    // generate a large array of random strings, half of which start with "t"
    let strings: Vec<String> = (0..10000000)
        .map(|i| {
            if i % 2 == 0 {
                "thompson is a bad bad dog".into()
            } else {
                "routers is full of bugs".into()
            }
        })
        .collect();
    let prefix = "thom";

    let expected = strings.iter().filter(|s| s.starts_with(prefix)).count();

    let mut group = c.benchmark_group("scan_prefix_long");
    group.bench_function("rust_string", |b| {
        b.iter(|| {
            let mut count = 0;
            for s in strings.iter() {
                if s.starts_with(prefix) {
                    count += 1;
                }
            }
            assert_eq!(count, expected);
        });
    });

    let strings: Vec<GermanString> = strings.into_iter().map(|s| s.into()).collect();
    group.bench_function("german_string", |b| {
        b.iter(|| {
            let mut count = 0;
            for s in strings.iter() {
                if s.starts_with(prefix) {
                    count += 1;
                }
            }
            assert_eq!(count, expected);
        });
    });
    group.finish();
}

fn scan_prefix_short(c: &mut Criterion) {
    // generate a large array of random strings, half of which start with "t"
    let strings: Vec<String> = (0..10000000)
        .map(|i| {
            if i % 2 == 0 {
                "thompson".into()
            } else {
                "routers".into()
            }
        })
        .collect();
    let prefix = "tho";

    let expected = strings
        .iter()
        .filter(|s| {
            assert!(s.len() <= 12);
            s.starts_with(prefix)
        })
        .count();

    let mut group = c.benchmark_group("scan_prefix_short");
    group.bench_function("rust_string", |b| {
        b.iter(|| {
            let mut count = 0;
            for s in strings.iter() {
                if s.starts_with(prefix) {
                    count += 1;
                }
            }
            assert_eq!(count, expected);
        });
    });

    let strings: Vec<GermanString> = strings.into_iter().map(|s| s.into()).collect();
    group.bench_function("german_string", |b| {
        b.iter(|| {
            let mut count = 0;
            for s in strings.iter() {
                if s.starts_with(prefix) {
                    count += 1;
                }
            }
            assert_eq!(count, expected);
        });
    });
    group.finish();
}

criterion_group! {
    name=benches;
    config = Criterion::default().with_profiler(PProfProfiler::new(10, Output::Flamegraph(None)));
    targets=scan_prefix_long, scan_prefix_short
}
criterion_main!(benches);
