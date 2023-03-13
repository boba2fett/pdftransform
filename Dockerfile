FROM rust:slim-bullseye as build

RUN USER=root cargo new --bin pdftransform
WORKDIR /pdftransform

RUN apt-get update -qq && \
    DEBIAN_FRONTEND=noninteractive apt-get install pkg-config libssl-dev wget -y --no-install-recommends && \
    apt-get clean && find /var/lib/apt/lists -type f -delete

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

RUN cargo build --release && rm src/*.rs

COPY ./src ./src

RUN rm ./target/release/deps/pdftransform*

COPY get_pdfium.sh .
RUN chmod +x get_pdfium.sh && ./get_pdfium.sh

RUN cargo build --release

FROM debian:bullseye-slim

RUN apt-get update -qq && \
    DEBIAN_FRONTEND=noninteractive apt-get install libreoffice-core-nogui libreoffice-java-common openjdk-17-jre-headless -y --no-install-recommends && \
    apt-get clean && find /var/lib/apt/lists -type f -delete

WORKDIR /pdftransform
COPY --from=build /pdftransform/target/release/pdftransform .
COPY --from=build /pdftransform/libpdfium.so .
ENV RUST_LOG=debug
EXPOSE 8000

ENTRYPOINT [ "./pdftransform"]
CMD [ "./pdftransform"]
