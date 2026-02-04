FROM rust:1.92 AS builder

WORKDIR /app

ARG TARGETARCH # https://docs.docker.com/build/building/multi-platform/
RUN if [ "$TARGETARCH" = "amd64" ]; then echo  "x86_64-unknown-linux-musl" | tee /rust-target; fi
RUN if [ "$TARGETARCH" = "arm64" ]; then echo "aarch64-unknown-linux-musl" | tee /rust-target; fi

RUN apt-get update && apt-get install -y musl-tools

RUN rustup target add $(cat /rust-target)

COPY Cargo.toml Cargo.lock ./

RUN cargo init --bin listener/
RUN cargo init --lib common/
RUN cargo init --lib pinger/
RUN cargo init --bin dns_updater/

COPY common/Cargo.toml ./common/
COPY dns_updater/Cargo.toml ./dns_updater/
COPY listener/Cargo.toml ./listener/
COPY pinger/Cargo.toml ./pinger/

RUN cargo fetch --locked --target $(cat /rust-target)

COPY listener/src ./listener/src
COPY common/src ./common/src

RUN cargo build --release --locked --target $(cat /rust-target) --bin listener
RUN mv /app/target/$(cat /rust-target)/release/listener /listener

# Runner
FROM scratch

COPY --from=builder /listener .

ENV ROCKET_ADDRESS=0.0.0.0
ENV ROCKET_PORT=8080
ENV RUST_LOG=info

EXPOSE 8080

CMD ["./listener"]
