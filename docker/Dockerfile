FROM rust:1-alpine3.23 AS builder
COPY src/ /app/src/
COPY Cargo.lock /app/Cargo.lock
COPY Cargo.toml /app/Cargo.toml
WORKDIR /app/
RUN cargo build --release --features docker

## main docker to actually run this stuff
FROM nikolaik/python-nodejs:python3.14-nodejs25-alpine
RUN addgroup -S appgroup && adduser -S appuser -G appgroup
RUN mkdir -p /home/appuser/code_location && chown -R appuser:appgroup /home/appuser/code_location && chmod 700 /home/appuser/code_location
WORKDIR /app
USER appuser
COPY --from=builder --chown=appuser:appgroup /app/target/release/code-exec-mcp /app/code-exec-mcp
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
ENV PATH="$HOME/.cargo/bin:${PATH}"
CMD ["./code-exec-mcp"]
#RUN source "$HOME/.cargo/env"
