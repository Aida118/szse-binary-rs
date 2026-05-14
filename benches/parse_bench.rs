use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use szse_binary_rs::{TickTrade, TickOrder, MsgHeader, TICK_TRADE_BODY_LEN, TICK_ORDER_BODY_LEN};

fn make_trade_buf() -> Vec<u8> {
    let mut buf = vec![0u8; TICK_TRADE_BODY_LEN];
    buf[0..2].copy_from_slice(&2011u16.to_be_bytes());
    buf[2..10].copy_from_slice(&1i64.to_be_bytes());
    buf[10..13].copy_from_slice(b"011");
    buf[13..21].copy_from_slice(&100i64.to_be_bytes());
    buf[21..29].copy_from_slice(&200i64.to_be_bytes());
    buf[29..37].copy_from_slice(b"000001  ");
    buf[37..41].copy_from_slice(b"102 ");
    buf[41..49].copy_from_slice(&186400i64.to_be_bytes());
    buf[49..57].copy_from_slice(&100000i64.to_be_bytes());
    buf[56] = b'F';
    buf[57..65].copy_from_slice(&20250512093000000i64.to_be_bytes());
    buf
}

fn bench_header(c: &mut Criterion) {
    let mut buf = [0u8; 8];
    buf[0..4].copy_from_slice(&300191u32.to_be_bytes());
    buf[4..8].copy_from_slice(&66u32.to_be_bytes());

    let mut group = c.benchmark_group("MsgHeader");
    group.throughput(Throughput::Elements(1));
    group.bench_function("parse", |b| {
        b.iter(|| MsgHeader::parse(black_box(&buf)))
    });
    group.finish();
}

fn bench_tick_trade(c: &mut Criterion) {
    let buf = make_trade_buf();

    let mut group = c.benchmark_group("TickTrade");
    group.throughput(Throughput::Bytes(TICK_TRADE_BODY_LEN as u64));
    group.bench_function("parse", |b| {
        b.iter(|| TickTrade::parse(black_box(&buf)))
    });
    group.finish();
}

fn bench_tick_trade_batch(c: &mut Criterion) {
    // 模拟批量解析 10000 条消息
    let single = make_trade_buf();
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

criterion_group!(benches, bench_header, bench_tick_trade, bench_tick_trade_batch);
criterion_main!(benches);