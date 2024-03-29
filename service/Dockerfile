FROM rust:1.71.0-slim-bookworm as build

WORKDIR /app
RUN rustup target add x86_64-unknown-linux-musl
ENV RUST_BACKTRACE=1

RUN apt-get update -qq && \
    DEBIAN_FRONTEND=noninteractive apt-get install musl-tools musl-dev build-essential -y --no-install-recommends && \
    apt-get clean && find /var/lib/apt/lists -type f -delete

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

WORKDIR /app
COPY --from=build /app/target/x86_64-unknown-linux-musl/release/service service
ENV RUST_LOG=debug
ENV RUST_BACKTRACE=1
EXPOSE 8000

ENTRYPOINT [ "./service"]
CMD [ "./service"]
