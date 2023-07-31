# Use a base image with the latest version of Rust installed
FROM rust:1.71.0 as builder

# Set the working directory in the container
WORKDIR /app

# Copy the local application code into the container
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Build the Rust application
RUN cargo build --release --locked

# Runtime Image
FROM debian:bullseye-slim
WORKDIR /app

RUN set -ex; \
    apt-get update; \
    apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Specify the command to run when the container starts
COPY --from=builder /app/target/release/main ./
COPY log_config.yaml ./
CMD ["/app/main","--host","0.0.0.0","--redis","redis://redis:6379/"]
