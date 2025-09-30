#########################################################################################
## docker will invoke this file from ../.. dir in order to access code
#########################################################################################
FROM rust:1.90-bullseye AS builder

ARG BUILD_VERSION=0.1

WORKDIR /src/

COPY ./ /src

RUN --mount=type=cache,target=/usr/local/cargo/registry \
      CARGO_APP_VERSION=$BUILD_VERSION cargo build --release --bin authware
#########################################################################################
FROM gcr.io/distroless/cc-debian11 AS ssl
#########################################################################################
# Debian 12 does not include ssl libs
FROM gcr.io/distroless/cc-debian12 AS runner
#########################################################################################

COPY LICENSE /licenses/LICENSE-bsd-3

WORKDIR /app

### /ssl
COPY --from=ssl /usr/lib/x86_64-linux-gnu/libssl.so.* /lib/x86_64-linux-gnu/
COPY --from=ssl /usr/lib/x86_64-linux-gnu/libcrypto.so.* /lib/x86_64-linux-gnu/
#########################################################################################

WORKDIR /app
EXPOSE 8000

COPY --from=builder /src/target/release/authware /app

ENTRYPOINT ["/app/authware"]
