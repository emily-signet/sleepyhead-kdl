use criterion::{criterion_group, criterion_main, Criterion};
use sleepyhead_kdl::assembler::*;
use sleepyhead_kdl::parser::*;

fn bench_parse(c: &mut Criterion) {
    let input = include_str!("schema.kdl");
    let mut group = c.benchmark_group("parse kdl schema in kdl");

    group.bench_function("sleepyhead-kdl", |b| {
        b.iter(|| parse_document(&mut Parser::from_str(input)))
    });

    group.bench_function("kdl-rs", |b| b.iter(|| input.parse::<kdl::KdlDocument>()));

    group.finish();
}

criterion_group!(benches, bench_parse);
criterion_main!(benches);
