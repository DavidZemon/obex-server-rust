FROM rustlang/rust:nightly-alpine3.12 as builder
RUN apk add --update musl-dev

RUN mkdir -p /opt/server/src
WORKDIR /opt/server
COPY Cargo.* /opt/server/

# Download and build dependencies
RUN echo "fn main () {}" > /opt/server/src/main.rs \
 && cargo build --release --color never

# Build primary application
COPY src src
RUN cargo clean --release --color never --package obex-server \
 && cargo build --release --color never --package obex-server

FROM alpine:3.12.7
RUN apk add --update git git-lfs
ENV EXE_PATH=/opt/obex/obex-server
COPY --from=builder /opt/server/target/release/obex-server "${EXE_PATH}"
COPY start.sh /
WORKDIR /opt/obex
ENTRYPOINT ["/start.sh"]
EXPOSE 8000
