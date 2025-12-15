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

# Create non-root user
RUN addgroup -g 1001 -S drago && \
    adduser -S drago -u 1001

WORKDIR /app
COPY --from=builder /app/build ./build
COPY --from=builder /app/package.json ./package.json

# Install only production dependencies
RUN bun install --frozen-lockfile --production

# Change ownership of app directory to non-root user
RUN chown -R drago:drago /app

# Switch to non-root user
USER drago

EXPOSE 3000
HEALTHCHECK --interval=5s --timeout=3s --retries=3 \
    CMD curl --fail http://localhost:3000 || exit 1
CMD ["bun", "run", "build/index.js"]
