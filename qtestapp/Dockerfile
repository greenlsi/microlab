# FROM rust:1.83.0-alpine

# RUN apk add --no-cache musl-dev gcc g++ make


# # Establecer el directorio de trabajo
# WORKDIR /app

# COPY Cargo.toml Cargo.lock ./
# COPY src ./src

# ENV QEMU_URL=0.0.0.0:3000
# ENV WS_URL=0.0.0.0:8081

# RUN cargo build --release
# # Exponer los puertos necesarios
# EXPOSE 3000 8081 8080

# # Comando por defecto para ejecutar el servidor
# CMD ["cargo", "run","--release"]




#NUEVA VERSIÓN MÁS OPTIMIZADA


### Etapa de construcción (build stage)
FROM rust:1.83.0-alpine AS builder

# Instalamos herramientas necesarias para compilar Rust en Alpine
RUN apk add --no-cache musl-dev gcc g++ make

# Establecemos el directorio de trabajo
WORKDIR /app

# Copiamos primero los archivos que definen las dependencias (esto permite aprovechar el cache de Docker)
COPY Cargo.toml Cargo.lock ./

#  Copiamos el código fuente
COPY src ./src

#  Compilamos el binario en modo release (más rápido y optimizado)
RUN cargo build --release

# Opcional: reducir el tamaño final eliminando símbolos de depuración
RUN strip target/release/qtestapp

###

###  Etapa de ejecución (runtime stage)
FROM alpine:3.19

#  Instalamos solo las librerías necesarias para ejecutar el binario (no compilar)
RUN apk add --no-cache libgcc libstdc++

# Creamos el directorio de trabajo
WORKDIR /app

#  Copiamos solo el binario final desde la etapa de build
COPY --from=builder /app/target/release/qtestapp  /app/servidor

# Variables de entorno necesarias para el servidor
ENV QEMU_URL=0.0.0.0:3000
ENV WS_URL=0.0.0.0:8081

# 📡 Exponemos los puertos utilizados
EXPOSE 3000 8081 8080

# 🔧 Comando por defecto al arrancar el contenedor
CMD ["./servidor"]