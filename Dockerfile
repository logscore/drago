# Build stage
FROM rust AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim
RUN apt update && apt install -y ca-certificates libmariadb3 && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/drago-dns .
EXPOSE 8080
CMD ["./drago-dns"]
