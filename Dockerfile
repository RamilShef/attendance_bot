FROM rust:1.75-slim AS builder

WORKDIR /app

RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock* ./
COPY src ./src

RUN cargo build --release

FROM debian:bookworm-slim

WORKDIR /app

RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/attendance_bot /app/attendance_bot


RUN mkdir -p /app/data
VOLUME ["/app/data"]

ENV DATABASE_URL=sqlite:///app/data/attendance.db?mode=rwc

CMD ["./attendance_bot"]