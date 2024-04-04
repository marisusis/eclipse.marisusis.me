FROM node:21-alpine as build_node

WORKDIR /site
COPY ./app ./app
WORKDIR /site/app
RUN corepack enable pnpm
RUN pnpm install -d
RUN pnpm run build

FROM rust:latest as build_rust

RUN cargo install --verbose --path .

FROM alpine:latest

WORKDIR /site
COPY . .

COPY --from=0 /site/app /site/app
COPY --from=1 /usr/local/cargo/bin/eclipse-marisusis-me /usr/local/bin/eclipse-marisusis-me

CMD ["eclipse-marisusis-me"]
