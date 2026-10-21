FROM mcr.microsoft.com/devcontainers/rust:1-bookworm

# Firmen-Proxy-Root-CAs (TLS-Inspection), lokal aus dem Windows-Store exportiert
COPY certs/*.crt /usr/local/share/ca-certificates/
RUN update-ca-certificates

# Cross-Compile nach Windows, damit die GUI nativ unter Windows laeuft
RUN apt-get update && apt-get install -y --no-install-recommends gcc-mingw-w64-x86-64 \
    && rm -rf /var/lib/apt/lists/* \
    && rustup target add x86_64-pc-windows-gnu
