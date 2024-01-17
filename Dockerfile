# Use the latest version of the Rust base image
FROM rust:latest

# Set the working directory in the container to /my
WORKDIR /usr/src/my-app

# Copy the Rust project files to the working directory
COPY . .

# Build the Rust app
RUN cargo build

# Set the command to run the Rust app
CMD cargo run

# FROM rust:latest as builder
# WORKDIR /usr/src/myapp
# COPY . .
# RUN cargo install --path .
# FROM debian:buster-slim
# RUN apt-get update & apt-get install -y extra-runtime-dependencies & rm -rf /var/lib/apt/lists/*
# COPY --from=builder /usr/local/cargo/bin/myapp /usr/local/bin/myapp
# CMD ["myapp"]