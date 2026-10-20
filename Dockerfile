# syntax=docker/dockerfile:1
FROM rust:1.99.1-bookworm AS builder
WORKDIR /source
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY examples ./examples
RUN cargo build --release --locked

FROM python:3.12-slim-bookworm
RUN apt-get update && apt-get install -y --no-install-recommends libgomp1 ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --uid 10001 --create-home paja \
    && mkdir /data && chown paja:paja /data
WORKDIR /app
COPY web/requirements.lock.txt ./web/requirements.lock.txt
RUN pip install --no-cache-dir -r web/requirements.lock.txt
COPY --from=builder /source/target/release/paja /usr/local/bin/paja
COPY web ./web
RUN chmod -R a+rX /app/web
COPY LICENSE NOTICE ./
ENV PAJA_BINARY=/usr/local/bin/paja PAJA_DATA=/data PYTHONUNBUFFERED=1 HF_HUB_DISABLE_TELEMETRY=1 HF_HOME=/data/huggingface
USER 10001:10001
EXPOSE 8000
CMD ["gunicorn", "--bind", "0.0.0.0:8000", "--workers", "1", "--threads", "4", "--timeout", "120", "--graceful-timeout", "120", "--worker-tmp-dir", "/tmp", "web.server:create_app()"]
