// this file simply builds the static typescript files static/ts,
// these are the files required to make the client code work

import { build } from "bun";
import fs from "node:fs";

if (process.env.DO_NOT_CLEAR_DIST === undefined)
    fs.rmSync("./static/js", { recursive: true }); // reset dist directory

const output = await build({
    entrypoints: [
        "./static/ts/pages/Footer.ts",
    ],
    minify: {
        identifiers: true,
        syntax: true,
        whitespace: true,
    },
    target: "browser",
    outdir: "./static/js",
    splitting: true,
    naming: {
        asset: "[name].[ext]",
        chunk: "[name]-[hash].[ext]",
        entry: "[name].[ext]", // do not include [dir]!!! files will NOT be findable if you include [dir]!!!
    },
});

// log finished
console.log("\x1b[30;100m info \x1b[0m Build finished!");
