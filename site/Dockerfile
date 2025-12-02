# Build stage
FROM oven/bun:1-alpine AS builder
WORKDIR /app
COPY bun.lock package.json ./
RUN bun install --frozen-lockfile
COPY . .
RUN bun run build

# Production stage
FROM oven/bun:1-alpine
RUN apk add --no-cache curl
WORKDIR /app
COPY --from=builder /app/build ./build
COPY --from=builder /app/package.json ./package.json
EXPOSE 3000
HEALTHCHECK --interval=5s --timeout=3s --retries=3 \
    CMD curl --fail http://localhost:3000 || exit 1
CMD ["bun", "run", "build/index.js"]
