FROM rust:1.49 as builder
WORKDIR /usr/src

# Download the target for static linking.
RUN rustup target add x86_64-unknown-linux-musl

# Create a dummy project and build the app's dependencies.
# If the Cargo.toml or Cargo.lock files have not changed,
# we can use the docker build cache and skip these (typically slow) steps.
RUN USER=root cargo new redis_orderbook

WORKDIR /usr/src/redis_orderbook
COPY redis/Cargo.lock redis/Cargo.toml ./redis/
COPY dummy.rs ./redis/src/main.rs
WORKDIR /usr/src/redis_orderbook/redis
RUN cargo build

WORKDIR /usr/src/redis_orderbook
COPY order/Cargo.lock order/Cargo.toml ./order/
COPY dummy.rs ./order/src/main.rs
WORKDIR /usr/src/redis_orderbook/order
RUN cargo build

WORKDIR /usr/src/redis_orderbook
COPY trade/Cargo.lock trade/Cargo.toml ./trade/
COPY dummy.rs ./trade/src/main.rs
WORKDIR /usr/src/redis_orderbook/trade
RUN cargo build

WORKDIR /usr/src/redis_orderbook
COPY Cargo.toml Cargo.lock ./
COPY dummy.rs ./src/main.rs
RUN cargo build

# Copy the source and build the application.
COPY src ./src/
COPY order ./order/
COPY redis ./redis/
COPY trade ./trade/
RUN cargo install --target x86_64-unknown-linux-musl --path . --bins


FROM debian:stable-slim
RUN apt-get update && apt-get install -y redis-server && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/* /usr/local/bin/
COPY entrypoint.sh /usr/bin/entrypoint.sh
CMD ["sh", "/usr/bin/entrypoint.sh"]
