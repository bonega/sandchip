#[macro_use]
extern crate criterion;

use criterion::Criterion;
use criterion::black_box;
use sandchiplib::cpu::CPU;

fn criterion_benchmark(c: &mut Criterion) {
    let mut cpu = CPU::new();
    cpu.load_rom("pong.ch8");
    c.bench_function("pong.ch8", move |b| b.iter(|| cpu.tick()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);