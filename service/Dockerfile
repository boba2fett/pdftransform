FROM rust:slim-bullseye as build

WORKDIR /app

RUN apt-get update -qq && \
    DEBIAN_FRONTEND=noninteractive apt-get install pkg-config libssl-dev wget git python3 procps xz-utils lsb-release sudo musl-tools -y --no-install-recommends && \
    apt-get clean && find /var/lib/apt/lists -type f -delete

RUN rustup target add x86_64-unknown-linux-musl

RUN cargo new --bin common
COPY common/Cargo.* common/

RUN echo "[workspace]\n"\
  "members = ['common']"\
  > Cargo.toml

COPY Cargo.lock .

RUN cargo build --release

RUN cargo new --bin service
COPY service/Cargo.* service/

RUN echo "[workspace]\n"\
  "members = ['common', 'service']"\
  > Cargo.toml

RUN cargo build --release

COPY common/src common/src
COPY service/src service/src

RUN cargo build --release --target x86_64-unknown-linux-musl

FROM scratch

WORKDIR /service
COPY --from=build /service/target/x86_64-unknown-linux-musl/release/service .
ENV RUST_LOG=debug
EXPOSE 8000

ENTRYPOINT [ "./service"]
CMD [ "./service"]
