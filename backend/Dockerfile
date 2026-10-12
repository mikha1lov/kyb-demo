FROM rust:1.97-slim-bookworm AS build
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release && rm -rf src
COPY migrations migrations
COPY src src
RUN touch src/main.rs && cargo build --release

FROM debian:bookworm-slim
RUN apt-get update \
  && apt-get install -y --no-install-recommends ca-certificates curl \
  && rm -rf /var/lib/apt/lists/* \
  && useradd --system --uid 10001 app
COPY --from=build /app/target/release/kyb-backend /usr/local/bin/kyb-backend
USER app
ENV BIND_ADDR=0.0.0.0:8787
EXPOSE 8787
HEALTHCHECK --interval=10s --timeout=3s --start-period=20s --retries=5 \
  CMD curl -fsS http://127.0.0.1:8787/api/health || exit 1
CMD ["kyb-backend"]
