FROM rust:1.71.0-slim-bookworm as build

WORKDIR /app

RUN apt-get update -qq && \
    DEBIAN_FRONTEND=noninteractive apt-get install pkg-config libssl-dev wget git python3 procps xz-utils lsb-release sudo musl-tools build-essential file g++-10 -y --no-install-recommends && \
    apt-get clean && find /var/lib/apt/lists -type f -delete

RUN update-alternatives --install /usr/bin/gcc gcc /usr/bin/gcc-10 10
RUN update-alternatives --install /usr/bin/g++ g++ /usr/bin/g++-10 10

RUN rustup target add x86_64-unknown-linux-musl
ENV RUST_BACKTRACE=1

RUN git clone https://chromium.googlesource.com/chromium/tools/depot_tools.git
ENV PATH="${PATH}:/app/depot_tools"

RUN gclient config --unmanaged https://pdfium.googlesource.com/pdfium.git
RUN gclient sync --no-history

WORKDIR /app/pdfium

RUN ./build/install-build-deps.sh --no-prompt
RUN rm out/Default/args.gn || true
RUN gn gen out/Default

COPY pdfium_options.sh .
RUN chmod +x pdfium_options.sh && ./pdfium_options.sh

RUN ninja -C out/Default pdfium

RUN stat /app/pdfium/out/Default/obj/libpdfium.a

WORKDIR /app

RUN cargo new --bin common
COPY common/Cargo.* common/

RUN echo "[workspace]\n"\
  "members = ['common']"\
  > Cargo.toml

COPY Cargo.lock .

RUN cargo build --release --target x86_64-unknown-linux-musl

RUN cargo new --bin preview
COPY preview/Cargo.* preview/

RUN echo "[workspace]\n"\
  "members = ['common', 'preview']"\
  > Cargo.toml

RUN cargo build --release --target x86_64-unknown-linux-musl --features static

COPY common/src common/src
COPY preview/src preview/src

RUN PDFIUM_STATIC_LIB_PATH=/app/pdfium/out/Default/obj/ cargo build --release --target x86_64-unknown-linux-musl  --features static

FROM scratch

WORKDIR /app
COPY --from=build /app/target/x86_64-unknown-linux-musl/release/preview preview
ENV RUST_LOG=debug
EXPOSE 8000

ENTRYPOINT [ "./preview"]
CMD [ "./preview"]
