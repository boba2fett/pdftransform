FROM rust:slim-bullseye as build

WORKDIR /pdftransform

RUN apt-get update -qq && \
    DEBIAN_FRONTEND=noninteractive apt-get install pkg-config libssl-dev wget -y --no-install-recommends && \
    apt-get clean && find /var/lib/apt/lists -type f -delete

COPY common/Cargo.* common/
RUN mkdir common/src/
RUN touch common/src/lib.rs

COPY worker/Cargo.* worker/
RUN mkdir worker/src/
RUN touch worker/src/lib.rs

COPY service/Cargo.* service/
RUN mkdir service/src/
RUN touch service/src/lib.rs
COPY Cargo.* .

RUN cargo build --release

RUN rm common/src/lib.rs
RUN rm service/src/lib.rs
RUN rm worker/src/lib.rs

COPY . .

COPY get_pdfium.sh .
RUN chmod +x get_pdfium.sh && ./get_pdfium.sh

RUN cargo build --release

FROM debian:bullseye-slim

RUN apt-get update -qq && \
    DEBIAN_FRONTEND=noninteractive apt-get install libreoffice-core-nogui libreoffice-java-common openjdk-17-jre-headless -y --no-install-recommends && \
    apt-get clean && find /var/lib/apt/lists -type f -delete

WORKDIR /pdftransform
COPY --from=build /pdftransform/target/release/service pdftransform
COPY --from=build /pdftransform/libpdfium.so .
ENV RUST_LOG=debug
EXPOSE 8000

ENTRYPOINT [ "./pdftransform"]
CMD [ "./pdftransform"]
