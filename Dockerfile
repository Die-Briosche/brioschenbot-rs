FROM rust as builder
WORKDIR /usr/src/brioschenbot-rs
COPY . .
RUN cargo install --path .

FROM debian:buster-slim
COPY --from=builder /usr/local/cargo/bin/brioschenbot-rs /usr/local/bin/brioschenbot-rs
RUN apt update && apt install -y libssl1.1 && apt install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY configuration.json /root/
WORKDIR /root
CMD ["brioschenbot-rs"]