# --- Build Stage ---
FROM rust:bookworm AS builder

WORKDIR /build

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libgtk-3-dev \
    libwebkit2gtk-4.1-dev \
    libssl-dev \
    cmake \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock build.rs ./
COPY src/ ./src/
COPY ui/ ./ui/

RUN cargo build --release

# --- Runtime Stage ---
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libgtk-3-0 \
    libwebkit2gtk-4.1-0 \
    libglib2.0-0 \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /build/target/release/amele /usr/local/bin/amele

RUN mkdir -p /root/.config/amele /cases /evidence

ENV AMELE_HOST=0.0.0.0 \
    AMELE_PORT=8080 \
    AMELE_NO_BROWSER=1 \
    AMELE_ALLOW_REMOTE=1

EXPOSE 8080

VOLUME ["/root/.config/amele", "/cases", "/evidence"]

HEALTHCHECK --interval=15s --timeout=5s --start-period=5s --retries=3 \
    CMD curl -f http://127.0.0.1:8080/api/health || exit 1

ENTRYPOINT ["amele"]
CMD ["server"]
