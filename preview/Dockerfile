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

RUN cargo new --bin preview
COPY preview/Cargo.* preview/

RUN echo "[workspace]\n"\
  "members = ['common', 'preview']"\
  > Cargo.toml

RUN cargo build --release

COPY common/src common/src
COPY preview/src preview/src

RUN cargo build --release

COPY get_pdfium.sh .
RUN chmod +x get_pdfium.sh && ./get_pdfium.sh

FROM debian:bullseye-slim

WORKDIR /app
COPY --from=build /app/target/release/preview preview
COPY --from=build /app/libpdfium.so .
ENV RUST_LOG=debug
EXPOSE 8000

ENTRYPOINT [ "./preview"]
CMD [ "./preview"]
