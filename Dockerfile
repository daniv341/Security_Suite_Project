# ---------- Etapa 1: build ----------
# Usamos la imagen oficial de Rust (slim) solo para compilar.
FROM rust:1-slim-bookworm AS builder

WORKDIR /app

# Dependencias de sistema necesarias para compilar sqlx (native-tls -> openssl).
RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# 1) Copiamos solo los manifiestos y creamos un main.rs/lib.rs "dummy"
#    para poder compilar y cachear las dependencias en una capa de
#    Docker independiente del código fuente (acelera builds).
COPY Cargo.toml ./
RUN mkdir src \
    && echo "fn main() { println!(\"placeholder\"); }" > src/main.rs \
    && echo "// placeholder" > src/lib.rs \
    && cargo build --release \
    && rm -rf src

# 2) Copiamos el código fuente real y las migraciones, y compilamos
#    el binario final en modo release.
COPY src ./src
COPY migrations ./migrations
RUN touch src/main.rs src/lib.rs \
    && cargo build --release

# ---------- Etapa opcional: desarrollo con recarga automática ----------
# Se usa solo con docker-compose.dev.yml (target: dev). El código
# fuente se monta como volumen en tiempo de ejecución (bind mount),
# así que acá no se copia `src/`: `cargo-watch` recompila y reinicia
# el servidor cada vez que detecta un cambio en los archivos montados.
FROM rust:1-slim-bookworm AS dev

WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/* \
    && cargo install cargo-watch --locked

CMD ["cargo", "watch", "--why", "-x", "run"]

# ---------- Etapa 2: runtime (producción) ----------
# Imagen ligera para ejecutar el binario ya compilado.
FROM debian:bookworm-slim AS runtime

WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --create-home --shell /usr/sbin/nologin appuser

COPY --from=builder /app/target/release/security-suite /app/security-suite
COPY --from=builder /app/migrations /app/migrations

USER appuser

EXPOSE 8080

CMD ["/app/security-suite"]
