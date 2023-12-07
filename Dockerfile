FROM rust:latest as builder
WORKDIR /app
COPY . .
RUN cargo build --release -p server

FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y ca-certificates && apt install -y openssl && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release .
EXPOSE 3001
CMD ["./server"]
