import { parseManifestSync } from "./index.js";

const manifest = parseManifestSync("./fail.manifest");

console.log(manifest);