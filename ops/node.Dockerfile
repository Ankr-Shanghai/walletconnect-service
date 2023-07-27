# Use a base image with the latest version of Rust installed
FROM rust:latest

# Set the working directory in the container
WORKDIR /app

# Copy the local application code into the container
COPY . .

# Build the Rust application
RUN cargo build --release

# Specify the command to run when the container starts
RUN cp ./target/release/main ./
CMD ["./main","--host","0.0.0.0","--redis","redis://redis:6379/"]
