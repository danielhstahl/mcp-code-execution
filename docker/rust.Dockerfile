FROM rust:1.93-alpine
RUN addgroup -S appgroup && adduser -S appuser -G appgroup
WORKDIR /app
USER appuser
