FROM node:25-alpine

RUN addgroup -S appgroup && adduser -S appuser -G appgroup
WORKDIR /app
USER appuser

COPY --chown=appuser:appgroup entrypoint-js.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh
ENV TYPE=default
ENTRYPOINT [ "/usr/local/bin/entrypoint.sh" ]
CMD ["index.js"]
