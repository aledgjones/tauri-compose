#!/usr/bin/env node

import fs from "fs";
import path from "path";
import archiver from "archiver";
import { fileURLToPath } from "url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const srcDir = path.join(__dirname, "src");
const distDir = path.join(__dirname, "dist");

// Ensure dist directory exists
if (!fs.existsSync(distDir)) {
  fs.mkdirSync(distDir, { recursive: true });
}

// Get all folders in src directory
const folders = fs
  .readdirSync(srcDir, { withFileTypes: true })
  .filter((dirent) => dirent.isDirectory())
  .map((dirent) => dirent.name);

console.log(
  `Found ${folders.length} instrument folders: ${folders.join(", ")}`,
);

// Function to zip a folder
function zipFolder(folderName) {
  return new Promise((resolve, reject) => {
    const folderPath = path.join(srcDir, folderName);
    const zipPath = path.join(distDir, `${folderName}.scvi`);

    console.log(`Zipping ${folderName}...`);

    const output = fs.createWriteStream(zipPath);
    const archive = archiver("zip", {
      zlib: { level: 9 }, // Maximum compression
    });

    output.on("close", () => {
      console.log(`${folderName}.scvi created (${archive.pointer()} bytes)`);
      resolve();
    });

    archive.on("error", (err) => {
      reject(err);
    });

    archive.pipe(output);
    archive.directory(folderPath, false);
    archive.finalize();
  });
}

// Zip all folders
async function zipAllFolders() {
  try {
    for (const folder of folders) {
      await zipFolder(folder);
    }
    console.log("All instruments zipped successfully!");
  } catch (error) {
    console.error("Error zipping folders:", error);
    process.exit(1);
  }
}

zipAllFolders();
