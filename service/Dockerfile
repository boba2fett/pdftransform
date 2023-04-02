FROM rust:1.68.2-slim-bullseye as build

WORKDIR /app

RUN apt-get update -qq && \
    DEBIAN_FRONTEND=noninteractive apt-get install pkg-config libssl-dev wget -y --no-install-recommends && \
    apt-get clean && find /var/lib/apt/lists -type f -delete

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

RUN cargo build --release

FROM debian:bullseye-slim

WORKDIR /app
COPY --from=build /app/target/release/service service
ENV RUST_LOG=debug
EXPOSE 8000

ENTRYPOINT [ "./service"]
CMD [ "./service"]
