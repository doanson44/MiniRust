const { spawnSync } = require("node:child_process");
const fs = require("node:fs");

function readStdin() {
  const chunks = [];
  const fd = 0;
  const buffer = Buffer.alloc(4096);
  let bytesRead;
  try {
    while ((bytesRead = fs.readSync(fd, buffer, 0, buffer.length, null)) > 0) {
      chunks.push(Buffer.from(buffer.subarray(0, bytesRead)));
    }
  } catch (error) {
    if (error.code !== "EAGAIN" && error.code !== "EOF") {
      throw error;
    }
  }
  return Buffer.concat(chunks).toString("utf8");
}

function filePathFrom(payload) {
  return (
    payload.file_path ||
    payload.filePath ||
    payload.path ||
    (payload.file && payload.file.path) ||
    ""
  );
}

let payload = {};
try {
  const raw = readStdin().trim();
  if (raw) {
    payload = JSON.parse(raw);
  }
} catch {
  process.stdout.write("{}\n");
  process.exit(0);
}

const filePath = filePathFrom(payload);
if (!filePath.toLowerCase().endsWith(".rs")) {
  process.stdout.write("{}\n");
  process.exit(0);
}

const result = spawnSync("rustfmt", [filePath], { encoding: "utf8" });
if (result.error || result.status !== 0) {
  process.stdout.write("{}\n");
  process.exit(0);
}

process.stdout.write("{}\n");
