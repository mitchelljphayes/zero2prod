FROM rust:1.71.1 as builder

WORKDIR /app
RUN apt update && apt install lld clang -y
COPY . .
ENV SQLX_OFFLINE true
RUN cargo build --release

FROM rust:1.71.1 as runtime
WORKDIR /app
COPY --from=builder /app/target/release/zero2prod zero2prod
ENV APP_ENVIRONMENT production
ENTRYPOINT ["./zero2prod"]

