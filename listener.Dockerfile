FROM rust:1.92 AS builder

WORKDIR /app

RUN apt-get update && apt-get install -y musl-tools

RUN rustup target add x86_64-unknown-linux-musl

COPY Cargo.toml Cargo.lock ./

RUN cargo init --bin listener/
RUN cargo init --lib common/
RUN cargo init --lib pinger/
RUN cargo init --bin dns_updater/

COPY common/Cargo.toml ./common/
COPY dns_updater/Cargo.toml ./dns_updater/
COPY listener/Cargo.toml ./listener/
COPY pinger/Cargo.toml ./pinger/

RUN cargo fetch --locked --target x86_64-unknown-linux-musl

COPY listener/src ./listener/src
COPY common/src ./common/src

RUN cargo build --release --locked --target x86_64-unknown-linux-musl --bin listener

# Runner
FROM scratch

COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/listener .

ENV ROCKET_ADDRESS=0.0.0.0
ENV ROCKET_PORT=8080
ENV RUST_LOG=info

EXPOSE 8080

CMD ["./listener"]
