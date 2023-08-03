FROM rust:1.71.0-slim-bookworm as build

WORKDIR /app
ENV RUST_BACKTRACE=1

RUN cargo new --bin common
COPY common/Cargo.* common/

RUN echo "[workspace]\n"\
  "members = ['common']"\
  > Cargo.toml

COPY Cargo.lock .

RUN cargo build --release --target x86_64-unknown-linux-gnu

RUN cargo new --bin test
COPY test/Cargo.* test/

RUN echo "[workspace]\n"\
  "members = ['common', 'test']"\
  > Cargo.toml

RUN cargo build --release --target x86_64-unknown-linux-gnu

COPY common/src common/src
COPY test/src test/src

RUN cargo build --release --target x86_64-unknown-linux-gnu

FROM scratch

WORKDIR /test
COPY --from=build /app/target/x86_64-unknown-linux-gnu/release/test .
ENV RUST_LOG=debug
ENV RUST_BACKTRACE=1
EXPOSE 8000

ENTRYPOINT [ "./test"]
CMD [ "./test"]
