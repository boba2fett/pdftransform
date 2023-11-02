FROM rust:1.73.0-slim-bookworm as build

WORKDIR /app
ENV RUST_BACKTRACE=1

RUN apt-get update -qq && \
    DEBIAN_FRONTEND=noninteractive apt-get install wget -y --no-install-recommends && \
    apt-get clean && find /var/lib/apt/lists -type f -delete

WORKDIR /app

RUN cargo new --bin common
COPY common/Cargo.* common/

RUN echo "[workspace]\n"\
  "members = ['common']"\
  > Cargo.toml

COPY Cargo.lock .

RUN cargo build --release --target x86_64-unknown-linux-gnu

RUN cargo new --bin transform
COPY transform/Cargo.* transform/

RUN echo "[workspace]\n"\
  "members = ['common', 'transform']"\
  > Cargo.toml

RUN cargo build --release --target x86_64-unknown-linux-gnu

COPY common/src common/src
COPY transform/src transform/src

RUN cargo build --release --target x86_64-unknown-linux-gnu

COPY get_pdfium.sh .
RUN bash get_pdfium.sh

FROM debian:bookworm-slim

WORKDIR /app
COPY --from=build /app/target/x86_64-unknown-linux-gnu/release/transform transform
COPY --from=build /app/libpdfium.so libpdfium.so
ENV RUST_LOG=debug
ENV RUST_BACKTRACE=1
EXPOSE 8000

ENTRYPOINT [ "./transform"]
CMD [ "./transform"]
