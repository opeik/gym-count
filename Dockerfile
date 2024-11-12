FROM rust:1.82-bookworm AS builder
WORKDIR /usr/src/myapp
COPY Cargo.* .
COPY src ./src
RUN cargo install --path .

FROM debian:bookworm-slim AS runner
RUN rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/gym-count /usr/local/bin/gym-count
CMD ["gym-count"]
