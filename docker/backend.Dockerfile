# Runtime packaging only. Compile on the LOCAL computer with build-line.bat.
FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --uid 10001 --no-create-home appuser
WORKDIR /app
COPY dist/line-linux/backend /app/backend
RUN chmod 755 /app/backend
ENV PORT=8080 RUST_LOG=info
USER appuser
EXPOSE 8080
HEALTHCHECK --interval=30s --timeout=5s --start-period=15s --retries=3 \
  CMD curl -fsS http://127.0.0.1:${PORT}/api/health || exit 1
CMD ["/app/backend"]
