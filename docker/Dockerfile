FROM rust:slim AS builder
COPY src/ /app/src/
COPY Cargo.lock /app/Cargo.lock
COPY Cargo.toml /app/Cargo.toml
WORKDIR /app/
RUN cargo build --release --features docker
## main docker to actually run this stuff
FROM nikolaik/python-nodejs:python3.14-nodejs25-slim
RUN apt-get update && apt-get install -y curl gcc libc6-dev
RUN python -m pip install pytest
RUN mkdir -p /app/code_location && chown -R pn:pn /app/code_location && chmod 770 /app/code_location
WORKDIR /app
USER pn
COPY --from=builder --chown=pn:pn /app/target/release/code-exec-mcp /app/code-exec-mcp
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/app/.cargo/bin:${PATH}"
CMD ["./code-exec-mcp"]
