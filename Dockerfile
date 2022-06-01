##
# Build binary
##

FROM rust:1.60 as builder

RUN USER=root cargo new --bin /app
WORKDIR /app

# Prepare dependencies
COPY ./Cargo.toml ./Cargo.lock /app/
RUN cargo build --release && \
    rm src/*.rs

# Compile app
COPY ./src /app/src/
RUN rm -rf ./target/release/vm-telegram-alerts* ./target/release/deps/vm_telegram_alerts* && \
    cargo build --release

##
# Prepare environment
##

FROM debian:bullseye-slim as runner

ENV TZ=Etc/UTC
ENV CONFIG_PATH=/app/config.yml

USER root
RUN apt-get update && apt-get install -y --no-install-recommends locales tzdata && \
    sed -i '/ru_RU.UTF-8/s/^# //g' /etc/locale.gen && locale-gen && \
    \
    apt-get install -y --no-install-recommends \
    ca-certificates netcat-openbsd wget curl libssl1.1 libfontconfig && \
    rm -rf /var/lib/apt/lists/* && rm -rf /var/lib/apt/lists.d/* && apt-get autoremove -y && apt-get clean && apt-get autoclean

WORKDIR /app
COPY --from=builder /app/target/release/vm-telegram-alerts /app/

CMD ["/app/vm-telegram-alerts"]
