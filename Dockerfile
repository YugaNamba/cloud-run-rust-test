FROM rust:latest as builder

RUN rustup target add "$(uname -m)"-unknown-linux-musl
RUN apt-get update && apt-get install -y musl-tools
WORKDIR /work
COPY . .
RUN cargo build --release --target "$(uname -m)"-unknown-linux-musl
RUN strip /work/target/"$(uname -m)"-unknown-linux-musl/release/api -o /api

FROM gcr.io/distroless/static

COPY --from=builder /api /
COPY /.env /
COPY /service_account.json /

ENTRYPOINT ["/api"]
