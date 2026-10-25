# Runtime packaging only. All JavaScript and dependencies were built on LOCAL.
FROM mcr.microsoft.com/playwright:v1.64.1-noble
WORKDIR /app
RUN mkdir -p /profile && chmod 700 /profile && chown pwuser:pwuser /profile
COPY --chown=pwuser:pwuser dist/oa-connector-linux/ /app/
USER pwuser
ENV OA_PROFILE_PATH=/profile OA_CONTROL_PORT=8082
VOLUME ["/profile"]
EXPOSE 8082
HEALTHCHECK --interval=20s --timeout=5s --start-period=45s --retries=3 \
  CMD node -e "fetch('http://127.0.0.1:8082/health').then(r=>process.exit(r.ok?0:1)).catch(()=>process.exit(1))"
CMD ["node", "connector.mjs"]
