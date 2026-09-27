# Etapa 1: Builder
FROM rust:1.75-bookworm AS builder

WORKDIR /app

# Instalar dependencias del sistema necesarias para compilar
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copiar solo los archivos de manifiesto primero para aprovechar la caché de Docker
COPY Cargo.toml Cargo.lock ./

# Crear un proyecto dummy para cachear las dependencias
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Copiar el código fuente real
COPY src ./src

# Compilar la aplicación en modo release
RUN cargo build --release

# Etapa 2: Runner (imagen final mínima)
FROM debian:bookworm-slim

WORKDIR /app

# Instalar solo certificados CA, librerías de runtime necesarias y curl para healthcheck
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Crear usuario no root para seguridad
RUN useradd -m -s /bin/bash appuser

# Copiar el binario compilado desde la etapa builder
COPY --from=builder /app/target/release/odysseus_engine .

# Cambiar propietario al usuario no root
RUN chown appuser:appuser odysseus_engine

# Cambiar a usuario no privilegiado
USER appuser

# Variables de entorno por defecto
ENV PORT=3000
ENV API_KEY=change-me-in-production
ENV RUST_LOG=info

# Exponer puerto
EXPOSE 3000

# Comando de entrada
CMD ["./odysseus_engine"]