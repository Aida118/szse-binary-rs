use criterion::{Criterion, Throughput, black_box, criterion_group, criterion_main};
use szse_binary_rs::{MSG_HEADER_LEN, MsgHeader, TICK_TRADE_BODY_LEN, TickTrade, parse_frame};

fn make_trade_body() -> Vec<u8> {
    let mut buf = vec![0u8; TICK_TRADE_BODY_LEN];
    buf[0..2].copy_from_slice(&2011u16.to_be_bytes());
    buf[2..10].copy_from_slice(&1i64.to_be_bytes());
    buf[10..13].copy_from_slice(b"011");
    buf[13..21].copy_from_slice(&100i64.to_be_bytes());
    buf[21..29].copy_from_slice(&200i64.to_be_bytes());
    buf[29..37].copy_from_slice(b"000001  ");
    buf[37..41].copy_from_slice(b"102 ");
    buf[41..49].copy_from_slice(&186_400i64.to_be_bytes());
    buf[49..57].copy_from_slice(&100_000i64.to_be_bytes());
    buf[57] = b'F';
    buf[58..66].copy_from_slice(&20_250_512_093_000_000i64.to_be_bytes());
    buf
}

fn make_trade_frame() -> Vec<u8> {
    let body = make_trade_body();
    let mut f = Vec::with_capacity(MSG_HEADER_LEN + body.len());
    f.extend_from_slice(&300191u32.to_be_bytes());
    f.extend_from_slice(&(body.len() as u32).to_be_bytes());
    f.extend_from_slice(&body);
    f
}

fn bench_header(c: &mut Criterion) {
    let mut buf = [0u8; 8];
    buf[0..4].copy_from_slice(&300191u32.to_be_bytes());
    buf[4..8].copy_from_slice(&66u32.to_be_bytes());

    let mut group = c.benchmark_group("MsgHeader");
    group.throughput(Throughput::Elements(1));
    group.bench_function("parse", |b| b.iter(|| MsgHeader::parse(black_box(&buf))));
    group.finish();
}

fn bench_tick_trade(c: &mut Criterion) {
    let buf = make_trade_body();
    let mut group = c.benchmark_group("TickTrade");
    group.throughput(Throughput::Bytes(TICK_TRADE_BODY_LEN as u64));
    group.bench_function("parse", |b| b.iter(|| TickTrade::parse(black_box(&buf))));
    group.finish();
}

fn bench_parse_frame(c: &mut Criterion) {
    // End-to-end: header decode + dispatch + body decode.
    let frame = make_trade_frame();
    let mut group = c.benchmark_group("parse_frame");
    group.throughput(Throughput::Elements(1));
    group.bench_function("tick_trade", |b| b.iter(|| parse_frame(black_box(&frame))));
    group.finish();
}

fn bench_tick_trade_batch(c: &mut Criterion) {
    let single = make_trade_body();
    let batch: Vec<u8> = single.repeat(10_000);
    let n = 10_000usize;

    let mut group = c.benchmark_group("TickTrade_batch");
    group.throughput(Throughput::Elements(n as u64));
    group.bench_function("parse_10k", |b| {
        b.iter(|| {
            let mut count = 0usize;
            let mut offset = 0;
            while offset + TICK_TRADE_BODY_LEN <= batch.len() {
                let _ = TickTrade::parse(black_box(&batch[offset..offset + TICK_TRADE_BODY_LEN]));
                offset += TICK_TRADE_BODY_LEN;
                count += 1;
            }
            count
        })
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_header,
    bench_tick_trade,
    bench_parse_frame,
    bench_tick_trade_batch
);
criterion_main!(benches);
