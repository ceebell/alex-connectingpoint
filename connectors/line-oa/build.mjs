import { build } from "esbuild";
import { mkdir, copyFile } from "node:fs/promises";
await mkdir("build", { recursive: true });
await build({ entryPoints: ["src/index.mjs"], outfile: "build/connector.mjs", bundle: true, platform: "node", target: "node22", format: "esm", external: ["playwright"], sourcemap: false });
await copyFile("src/control.html", "build/control.html");
