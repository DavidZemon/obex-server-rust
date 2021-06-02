FROM rustlang/rust:nightly-alpine3.12 as builder
RUN apk add --update musl-dev

RUN mkdir -p /opt/server/src
WORKDIR /opt/server
COPY Cargo.* /opt/server/
# Drop an empty main file in place so that all dependencies can be downloaded and built
RUN echo "fn main () {}" > /opt/server/src/main.rs
RUN cargo build --release --color never
COPY . .
RUN cargo clean --release --color never --package obex-server \
 && cargo build --release --color never --package obex-server

FROM alpine:3.12.7
COPY --from=builder /opt/server/target/release/obex-server /opt/server/obex-server
CMD ["/opt/server/obex-server"]
EXPOSE 8000
