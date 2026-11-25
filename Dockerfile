# Build stage
FROM rust:1.99.1-bookworm AS build
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY apps/api ./apps/api
COPY crates ./crates
RUN cargo build --release -p api

# Runtime stage
FROM debian:bookworm-slim
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates \
 && rm -rf /var/lib/apt/lists/* \
 && useradd --system --no-create-home forager
COPY --from=build /app/target/release/api /usr/local/bin/forager-api
USER forager
ENV BIND_ADDR=0.0.0.0:3001
EXPOSE 3001
CMD ["forager-api"]
