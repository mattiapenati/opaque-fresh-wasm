FROM alpine:latest AS build

WORKDIR /build
COPY . .

RUN --mount=type=cache,target=/root/.rustup \
    --mount=type=cache,target=/root/.cache \
    --mount=type=cache,target=/var/cache/apk \
    --mount=type=cache,target=/build/target \
    set -eux; \
    apk update; \
    apk add binutils ca-certificates curl gcc musl-dev; \
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --profile minimal -y; \
    source ~/.cargo/env; \
    cargo build --release; \
    cp target/release/fresh-auth-api /

FROM scratch

COPY --from=build /fresh-auth-api /fresh-auth-api
EXPOSE 8080
ENTRYPOINT ["/fresh-auth-api"]
