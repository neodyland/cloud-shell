FROM rust:slim AS builder

WORKDIR /src/builder

COPY . .
RUN --mount=type=cache,target=/src/builder/target/ cargo build --release && \
    cp /src/builder/target/release/cloud-shell /tmp/cloud-shell

FROM gcr.io/distroless/cc-debian12

WORKDIR /src/app

COPY --from=builder /tmp/cloud-shell .

CMD ["/src/app/cloud-shell"]