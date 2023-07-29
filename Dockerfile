FROM rust:latest as builder
WORKDIR /app
COPY . .
RUN cargo build --release -p server

FROM debian:bullseye-slim
# RUN apt-get update && apt-get install -y extra-runtime-dependencies && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release .
COPY .env .
EXPOSE 3001
ENV DATABASE_USER=canglong
ENV DATABASE_PASSWORD=fa461neiOt2Ujr5L
CMD ["./server"]
