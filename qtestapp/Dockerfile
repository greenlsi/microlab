FROM rust:1.83.0-alpine

RUN apk add --no-cache musl-dev gcc g++ make


# Establecer el directorio de trabajo
WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src

ENV QEMU_URL=0.0.0.0:3000
ENV WS_URL=0.0.0.0:8081

RUN cargo build --release
# Exponer los puertos necesarios
EXPOSE 3000 8081 8080

# Comando por defecto para ejecutar el servidor
CMD ["cargo", "run","--release"]
