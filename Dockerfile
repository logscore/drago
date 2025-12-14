# Build stage
FROM rust AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN CARGO_BUILD_JOBS=2 cargo build --release

# Runtime stage
FROM debian:bookworm-slim
RUN apt update && apt install -y ca-certificates libmariadb3 && apt-get install -y --no-install-recommends curl && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/drago-dns .
EXPOSE 8088
HEALTHCHECK --interval=5s --timeout=3s --retries=3 \
    CMD curl --fail http://localhost:8088/health || exit 1
CMD ["./drago-dns"]
