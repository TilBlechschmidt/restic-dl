FROM node:22-alpine3.20 AS assets

WORKDIR /usr/src

COPY package.json .
RUN npm install

COPY ./templates ./templates
RUN npm run prod

# ------------ ------------ ------------ ------------ ------------

FROM rust:1.78.0 AS builder

WORKDIR /usr/src

RUN rustup target add x86_64-unknown-linux-musl
RUN apt update && apt install -y musl-tools && rm -rf /var/lib/apt/lists/*

COPY . .
COPY --from=assets /usr/src/assets/css ./assets/css

RUN --mount=type=cache,target=./target \
    --mount=type=cache,target=/usr/local/cargo/registry \
    cargo install --target x86_64-unknown-linux-musl --path .

# ------------ ------------ ------------ ------------ ------------

FROM scratch

COPY --from=builder /usr/local/cargo/bin/restic-dl .

USER 1000
EXPOSE 9242

ENTRYPOINT ["./restic-dl"]
