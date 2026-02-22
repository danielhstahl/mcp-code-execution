FROM rust:1.93-alpine
RUN addgroup -S appgroup -g 1001 && adduser -S appuser -u 1001 -G appgroup
WORKDIR /app
USER appuser
