# Build stage
FROM rust:1.80-alpine AS builder

# Install build dependencies
RUN apk add --no-cache musl-dev openssl-dev openssl-libs-static

WORKDIR /app

# Copy Cargo files first for better caching
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src

# Build release binary
ENV RUSTFLAGS="-C target-feature=+crt-static"
RUN cargo build --release

# Runtime stage
FROM alpine:latest

# Install runtime dependencies
RUN apk add --no-cache ca-certificates

# Copy binary from builder
COPY --from=builder /app/target/release/cx /usr/local/bin/cx

# Create working directory
WORKDIR /workspace

# Expose port
EXPOSE 8080

# Set entrypoint and default command
ENTRYPOINT ["cx"]
CMD ["serve", "--host", "0.0.0.0"]
