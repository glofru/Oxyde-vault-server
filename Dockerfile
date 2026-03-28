FROM rust:1.90.0 AS builder

WORKDIR /Oxyde-vault-server
COPY . .
RUN cargo build --release --bin Oxyde-vault-server

FROM debian:trixie-slim AS runtime

RUN apt-get update && apt-get install \
    git \
    ca-certificates \
    -yq --no-install-suggests --no-install-recommends --allow-downgrades --allow-remove-essential --allow-change-held-packages \
  && apt-get clean \

WORKDIR /Oxyde-vault-server
COPY start.sh .
COPY --from=builder /Oxyde-vault-server/target/release/Oxyde-vault-server /usr/local/bin

ENTRYPOINT ["/start.sh"]