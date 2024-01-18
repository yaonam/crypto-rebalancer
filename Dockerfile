# # Use the latest version of the Rust base image
# FROM rust:latest

# # Set the working directory in the container to /my
# WORKDIR /usr/src/my-app

# # Copy the Rust project files to the working directory
# COPY . .

# # Build the Rust app
# RUN cargo build

# # Set the command to run the Rust app
# CMD cargo run


FROM rust:latest
WORKDIR /usr/src/myapp
COPY . .
RUN cargo install --path .
CMD ["rebalancer"]


# FROM rust:latest as builder
# WORKDIR /usr/src/myapp
# COPY . .
# RUN cargo install --path .
# FROM debian:buster-slim
# RUN apt-get update & apt-get install -y extra-runtime-dependencies & rm -rf /var/lib/apt/lists/*
# COPY --from=builder /usr/local/cargo/bin/rebalancer /usr/local/bin/rebalancer
# CMD ["rebalancer"]


# FROM rust:latest as builder
# WORKDIR /usr/src/myapp
# COPY . .
# RUN cargo build --release
# FROM debian:buster-slim
# COPY --from=builder /usr/src/myapp/target/release/rebalancer /rebalancer
# CMD [ "/rebalancer" ]


# ####################################################################################################
# ## Builder
# ####################################################################################################
# FROM rust:latest AS builder

# RUN update-ca-certificates

# # # Create appuser
# # ENV USER=myip
# # ENV UID=10001

# # RUN adduser \
# #     --disabled-password \
# #     --gecos "" \
# #     --home "/nonexistent" \
# #     --shell "/sbin/nologin" \
# #     --no-create-home \
# #     --uid "${UID}" \
# #     "${USER}"


# WORKDIR /usr

# COPY ./ .

# # We no longer need to use the x86_64-unknown-linux-musl target
# RUN cargo build --release

# ####################################################################################################
# ## Final image
# ####################################################################################################
# FROM gcr.io/distroless/cc

# # # Import from builder.
# # COPY --from=builder /etc/passwd /etc/passwd
# # COPY --from=builder /etc/group /etc/group

# WORKDIR /usr

# # Copy our build
# COPY --from=builder /usr/target/release/rebalancer ./

# # # Use an unprivileged user.
# # USER myip:myip

# CMD ["/usr/rebalancer"]


# FROM scratch
# COPY crypto-rebalancer/target/release/rebalancer /rebalancer
# CMD ["/rebalancer"]