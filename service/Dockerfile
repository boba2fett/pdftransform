FROM rust:slim-bullseye as openssl

WORKDIR /app

RUN apt-get update -qq && \
    DEBIAN_FRONTEND=noninteractive apt-get install pkg-config libssl-dev wget git python3 procps xz-utils lsb-release sudo musl-tools build-essential -y --no-install-recommends && \
    apt-get clean && find /var/lib/apt/lists -type f -delete

RUN ln -s /usr/include/x86_64-linux-gnu/asm /usr/include/x86_64-linux-musl/asm &&     ln -s /usr/include/asm-generic /usr/include/x86_64-linux-musl/asm-generic &&     ln -s /usr/include/linux /usr/include/x86_64-linux-musl/linux

RUN mkdir -p /app/lib

RUN wget https://github.com/openssl/openssl/archive/OpenSSL_1_1_1f.tar.gz
RUN tar zxvf OpenSSL_1_1_1f.tar.gz 
WORKDIR /app/openssl-OpenSSL_1_1_1f

RUN CC="musl-gcc -fPIE -pie" ./Configure no-shared no-async --prefix=/app/lib --openssldir=/app/lib/ssl linux-x86_64
RUN make depend
RUN make -j$(nproc)
RUN make install

RUN ls /app/lib/

FROM rust:slim-bullseye as build

WORKDIR /app

RUN apt-get update -qq && \
    DEBIAN_FRONTEND=noninteractive apt-get install pkg-config libssl-dev wget git python3 procps xz-utils lsb-release sudo musl-tools -y --no-install-recommends && \
    apt-get clean && find /var/lib/apt/lists -type f -delete

RUN rustup target add x86_64-unknown-linux-musl

COPY --from=openssl /app/lib/ lib/

ENV PKG_CONFIG_ALLOW_CROSS=1
ENV OPENSSL_STATIC=true
ENV OPENSSL_DIR=/app/lib

RUN cargo new --bin common
COPY common/Cargo.* common/

RUN echo "[workspace]\n"\
  "members = ['common']"\
  > Cargo.toml

COPY Cargo.lock .

RUN cargo build --release --target x86_64-unknown-linux-musl

RUN cargo new --bin service
COPY service/Cargo.* service/

RUN echo "[workspace]\n"\
  "members = ['common', 'service']"\
  > Cargo.toml

RUN cargo build --release --target x86_64-unknown-linux-musl

COPY common/src common/src
COPY service/src service/src

RUN cargo build --release --target x86_64-unknown-linux-musl

FROM scratch

WORKDIR /service
COPY --from=build /app/target/x86_64-unknown-linux-musl/release/service .
ENV RUST_LOG=debug
EXPOSE 8000

ENTRYPOINT [ "./service"]
CMD [ "./service"]
