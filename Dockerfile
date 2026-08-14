# Use Rust image to build the server
FROM rust:1.75 as builder

WORKDIR /usr/src/mneme

# Copy workspace files
COPY Cargo.toml Cargo.lock* ./
COPY core ./core
COPY cli ./cli
COPY bindings/python ./bindings/python
COPY server ./server

# Build the server
RUN cargo build --release -p mneme-server

# Use a minimal Debian image for runtime
FROM debian:bookworm-slim

# Install ca-certificates for HTTPS if needed
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy built binary
COPY --from=builder /usr/src/mneme/target/release/mneme-server /usr/local/bin/mneme-server

# Expose the server port
EXPOSE 8000

# Run the server
CMD ["mneme-server"]