FROM rust:1.78 AS builder
WORKDIR /app
COPY . .
RUN apt-get update && apt-get install -y pkg-config libpq-dev
RUN cargo build --release

FROM debian:buster-slim
WORKDIR /app
COPY --from=builder /app/target/release/restaurant_app .
COPY templates/ templates/
COPY static/ static/
CMD ["./restaurant_app"]
