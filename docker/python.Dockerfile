FROM python:3.13-alpine

RUN addgroup -S appgroup && adduser -S appuser -G appgroup
WORKDIR /app
USER appuser

RUN wget -qO- https://astral.sh/uv/install.sh | sh
RUN python3 -m pip install pytest
#COPY --chown=appuser:appgroup entrypoint-python.sh /usr/local/bin/entrypoint.sh
#RUN chmod +x /usr/local/bin/entrypoint.sh
#ENV TYPE=default
#ENTRYPOINT [ "/usr/local/bin/entrypoint.sh" ]
#CMD ["main.py"]
