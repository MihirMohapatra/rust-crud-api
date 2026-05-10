FROM rust:1.95-bookworm AS builder

WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends libpq-dev pkg-config \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY diesel.toml ./
COPY migrations ./migrations
COPY src ./src

RUN cargo build --release
RUN cargo install diesel_cli --no-default-features --features postgres --locked

FROM debian:bookworm-slim AS runtime

WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates libpq5 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/my-crud-app /usr/local/bin/my-crud-app
COPY --from=builder /usr/local/cargo/bin/diesel /usr/local/bin/diesel
COPY diesel.toml ./
COPY migrations ./migrations

EXPOSE 8080

CMD ["sh", "-c", "diesel migration run && my-crud-app"]
