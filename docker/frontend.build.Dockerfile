# LOCAL COMPUTER ONLY: build a relocatable Linux/amd64 Next.js release.
FROM node:22-bookworm-slim AS builder
WORKDIR /build
ENV NEXT_TELEMETRY_DISABLED=1
COPY frontend/package.json frontend/package-lock.json ./
RUN npm ci
COPY frontend/ ./
ARG BACKEND_URL=http://backend:8080
ENV BACKEND_URL=${BACKEND_URL}
RUN npm run lint --if-present && npm run build \
    && test -f .next/standalone/server.js \
    && mkdir -p /release/.next \
    && cp -a .next/standalone/. /release/ \
    && cp -a .next/static /release/.next/static \
    && if [ -d public ]; then cp -a public /release/public; fi
FROM scratch AS artifact
COPY --from=builder /release/ /
