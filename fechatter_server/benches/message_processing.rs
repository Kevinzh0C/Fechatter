use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use fechatter_server::dtos::models::{CreateMessageDto, MessageDto};
use serde_json;
use std::time::Duration;

fn benchmark_message_serialization(c: &mut Criterion) {
    let message = MessageDto {
        id: 12345.into(),
        chat_id: 67890.into(),
        sender_id: 11111.into(),
        content: "Hello, this is a benchmark test message!".to_string(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        reply_to: None,
        forwarded_from: None,
        edited: false,
        deleted: false,
        system: false,
        reactions: None,
        attachments: None,
        mentions: None,
        read_by: None,
        delivered_to: None,
        urls: None,
        thread_id: None,
        thread_count: 0,
    };

    let mut group = c.benchmark_group("message_serialization");

    group.bench_function("to_json", |b| {
        b.iter(|| {
            let json = serde_json::to_string(black_box(&message)).unwrap();
            black_box(json);
        })
    });

    group.bench_function("to_json_vec", |b| {
        b.iter(|| {
            let json = serde_json::to_vec(black_box(&message)).unwrap();
            black_box(json);
        })
    });

    group.finish();
}

fn benchmark_message_deserialization(c: &mut Criterion) {
    let json = r#"{
        "id": 12345,
        "chat_id": 67890,
        "sender_id": 11111,
        "content": "Hello, this is a benchmark test message!",
        "created_at": "2024-01-01T00:00:00Z",
        "updated_at": "2024-01-01T00:00:00Z",
        "edited": false,
        "deleted": false,
        "system": false,
        "thread_count": 0
    }"#;

    c.bench_function("message_from_json", |b| {
        b.iter(|| {
            let message: MessageDto = serde_json::from_str(black_box(json)).unwrap();
            black_box(message);
        })
    });
}

fn benchmark_batch_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_processing");

    for size in [10, 100, 1000].iter() {
        let messages: Vec<CreateMessageDto> = (0..*size)
            .map(|i| CreateMessageDto {
                chat_id: 1.into(),
                content: format!("Message {}", i),
                reply_to: None,
                forwarded_from: None,
                attachments: None,
                mentions: None,
            })
            .collect();

        group.bench_with_input(BenchmarkId::from_parameter(size), &messages, |b, msgs| {
            b.iter(|| {
                // Simulate batch validation
                for msg in msgs {
                    let _ = msg.content.len() < 10000;
                    let _ = !msg.content.is_empty();
                }
            })
        });
    }

    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .sample_size(100)
        .measurement_time(Duration::from_secs(10));
    targets = benchmark_message_serialization, benchmark_message_deserialization, benchmark_batch_processing
}

criterion_main!(benches);
