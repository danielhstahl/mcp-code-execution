FROM python:3.13-alpine

RUN addgroup -S appgroup && adduser -S appuser -G appgroup
WORKDIR /app
USER appuser

RUN wget -qO- https://astral.sh/uv/install.sh | sh
COPY --chown=appuser:appgroup entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh
ENV TYPE=default
ENTRYPOINT [ "/usr/local/bin/entrypoint.sh" ]
CMD ["main.py"]
