# LOCAL COMPUTER ONLY: produce Linux/amd64 release artifacts for Ubuntu.
FROM rust:1-bookworm AS builder
WORKDIR /build
ARG CARGO_BUILD_JOBS=2
COPY backend/Cargo.toml backend/Cargo.lock ./
RUN mkdir src && echo 'fn main() {}' > src/main.rs \
    && cargo build --locked --release -j "${CARGO_BUILD_JOBS}" && rm -rf src
COPY backend/src ./src
RUN touch src/main.rs && cargo build --locked --release -j "${CARGO_BUILD_JOBS}"
FROM scratch AS artifact
COPY --from=builder /build/target/release/backend /backend
