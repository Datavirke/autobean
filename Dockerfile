# Based on: https://levelup.gitconnected.com/create-an-optimized-rust-alpine-docker-image-1940db638a6c

##### Builder
FROM rust:1.71.0-slim as builder

WORKDIR /usr/src

# Create blank project
RUN USER=root cargo new autobean

# We want dependencies cached, so copy those first.
COPY Cargo.toml Cargo.lock /usr/src/autobean/

# Set the working directory
WORKDIR /usr/src/autobean

RUN rustup target add x86_64-unknown-linux-musl

# This is a dummy build to get the dependencies cached.
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/src/autobean/target \
    cargo build --target x86_64-unknown-linux-musl --release

COPY src /usr/src/autobean/src/
RUN touch /usr/src/autobean/src/main.rs

# Build it for real with caching, and copy the resulting binary
# into /usr/local/bin since cache directories become inaccessible
# at the end of the running command (apparently)
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/src/autobean/target \
    cargo build --target x86_64-unknown-linux-musl --release && \
    cp /usr/src/autobean/target/x86_64-unknown-linux-musl/release/autobean /usr/local/bin

##### Runtime
FROM scratch AS runtime 

COPY --from=builder /usr/local/bin /

VOLUME /data
WORKDIR /data

ENTRYPOINT ["/autobean"]
CMD ["/data"]
