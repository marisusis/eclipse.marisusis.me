FROM node:21-alpine as build_node

WORKDIR /site
COPY ./app ./app
WORKDIR /site/app
RUN corepack enable pnpm
RUN pnpm install -d
RUN pnpm run build

FROM rust:1.77-bullseye as build_rust

COPY ./src ./src
COPY ./Cargo.toml ./Cargo.lock ./

RUN cargo install --verbose --path .

RUN ls -la /usr/local/cargo/bin

FROM debian:bullseye

WORKDIR /site

COPY --from=build_node /site/app /site/app
COPY --from=build_rust /usr/local/cargo/bin/eclipse-marisusis-me /usr/local/bin/eclipse-marisusis-me

RUN apt-get update && apt install libssl-dev libudev-dev -y
RUN apt install build-essential -y

CMD ["eclipse-marisusis-me"]
