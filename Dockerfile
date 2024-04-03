FROM node:21-alpine as build_node

WORKDIR /site
COPY ./app ./app
WORKDIR /site/app
RUN corepack enable pnpm
RUN pnpm install -d
RUN pnpm run build

FROM rust:latest as build_rust

WORKDIR /site
COPY . .

COPY --from=0 /site/app /site/app

RUN cargo install --verbose --path .

CMD ["eclipse-marisusis-me"]
