FROM node:22-bookworm-slim AS build
WORKDIR /build
ENV PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD=1
COPY connectors/line-oa/package.json connectors/line-oa/package-lock.json ./
RUN --mount=type=cache,target=/root/.npm npm ci --no-audit --no-fund
COPY connectors/line-oa/build.mjs ./
COPY connectors/line-oa/src ./src
COPY connectors/line-oa/test ./test
RUN npm test && npm run build && npm prune --omit=dev --ignore-scripts --no-audit --no-fund \
    && mkdir /release && cp build/* package.json /release/ && cp -a node_modules /release/ \
    && printf '1.64.1\n' > /release/playwright.version
FROM scratch AS artifact
COPY --from=build /release/ /
