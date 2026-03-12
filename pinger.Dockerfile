FROM rust:1.92 AS builder

WORKDIR /app

ARG TARGETARCH # https://docs.docker.com/build/building/multi-platform/
RUN if [ "$TARGETARCH" = "" ];      then echo  "x86_64-unknown-linux-musl" | tee /rust-target; fi
RUN if [ "$TARGETARCH" = "amd64" ]; then echo  "x86_64-unknown-linux-musl" | tee /rust-target; fi
RUN if [ "$TARGETARCH" = "arm64" ]; then echo "aarch64-unknown-linux-musl" | tee /rust-target; fi

RUN apt-get update && apt-get install -y musl-tools ca-certificates

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

COPY dns_updater/src ./dns_updater/src
COPY common/src ./common/src
COPY pinger/src ./pinger/src

RUN cargo build --release --locked --target $(cat /rust-target) --bin dns_updater
RUN mv /app/target/$(cat /rust-target)/release/dns_updater /dns_updater

# Runner
FROM scratch

# Copied certificates from build container
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

COPY --from=builder /dns_updater .

ENV RUST_LOG=info
ENV REMOTE=""
ENV CLOUDFLARE_AUTH_KEY=""
ENV ZONE_ID=""
ENV MOCK=false

EXPOSE 8080

CMD ["./dns_updater"]
