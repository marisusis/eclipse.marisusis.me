# FROM node:21-alpine
FROM rust:latest as builder

WORKDIR /app
COPY . .

RUN cargo install --verbose --path .

CMD ["eclipse-marisusis-me"]
