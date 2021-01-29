FROM rust:1.49-slim-buster AS build
LABEL maintainer="hello@informal.systems"
COPY . /tendermint-rs/
WORKDIR /tendermint-rs/abci
RUN cargo build --release --bin kvstore-rs --features binary,kvstore-app

FROM debian:buster-slim
COPY --from=build /tendermint-rs/target/release/kvstore-rs /usr/local/bin/kvstore-rs
ENTRYPOINT [ "/usr/local/bin/kvstore-rs" ]
