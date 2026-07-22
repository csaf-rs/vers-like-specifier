use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use std::fmt::Write;
use std::hint::black_box;
use std::str::FromStr;
use vls::Vls;

/// Build a vls string with `n` constraints, e.g. ">=1.0.0|!=1.0.1|<=1.0.2|..."
fn build_vls_string(n: usize) -> String {
    let comparators = [">=", "!=", "<=", ">", "<", "=", ""];
    let mut buf = String::new();
    for i in 0..n {
        if i > 0 {
            buf.push('|');
        }
        let cmp = comparators[i % comparators.len()];
        write!(
            buf,
            "{}{}.{}.{}",
            cmp,
            i / 10_000,
            (i / 100) % 1000,
            i % 100
        )
        .unwrap();
    }
    buf
}

fn bench_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("vls_parse");

    let test_data: Vec<(usize, String)> = [100, 1_000, 10_000, 50_000, 100_000]
        .iter()
        .map(|&n| (n, build_vls_string(n)))
        .collect();

    for (count, input) in test_data {
        group.throughput(criterion::Throughput::Elements(count as u64));
        group.bench_with_input(
            BenchmarkId::new("constraints", count),
            &input,
            |b, input| {
                b.iter(|| black_box(Vls::from_str(black_box(input)).unwrap()));
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_parsing);
criterion_main!(benches);
