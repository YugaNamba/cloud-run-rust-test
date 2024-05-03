FROM rust:latest as builder

WORKDIR /work
COPY . .
RUN cargo build --release
RUN strip /work/target/release/api -o /api

FROM gcr.io/distroless/cc

COPY --from=builder /api /
COPY /.env /
COPY /service_account.json /

ENTRYPOINT ["/api"]
