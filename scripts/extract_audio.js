import fs from "fs";
import path from "path";
import { fileURLToPath } from "url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

function extractAudioFromPatches() {
  const patchesDir = path.join(__dirname, "..", "src", "assets", "patches");

  function processDirectory(dir) {
    const items = fs.readdirSync(dir);
    for (const item of items) {
      const fullPath = path.join(dir, item);
      const stat = fs.statSync(fullPath);
      if (stat.isDirectory()) {
        processDirectory(fullPath);
      } else if (path.extname(item) === ".json") {
        processJsonFile(fullPath);
      }
    }
  }

  function processJsonFile(jsonPath) {
    console.log(`Processing ${jsonPath}`);
    const content = fs.readFileSync(jsonPath, "utf8");
    const patches = JSON.parse(content);

    // Skip if not an array of patches (e.g., contents.json)
    if (
      !Array.isArray(patches) ||
      patches.length === 0 ||
      !Array.isArray(patches[0])
    ) {
      return;
    }

    const jsonName = path.basename(jsonPath, ".json");
    const audioDir = path.join(path.dirname(jsonPath), jsonName);

    if (!fs.existsSync(audioDir)) {
      fs.mkdirSync(audioDir, { recursive: true });
    }

    for (let i = 0; i < patches.length; i++) {
      const patch = patches[i];
      if (patch.length >= 4 && typeof patch[3] === "string") {
        if (patch[3].startsWith("data:audio/aac;base64,")) {
          const base64Data = patch[3].substring(
            "data:audio/aac;base64,".length,
          );
          const buffer = Buffer.from(base64Data, "base64");
          const audioFileName = `${i + 1}.aac`;
          const audioPath = path.join(audioDir, audioFileName);
          fs.writeFileSync(audioPath, buffer);
        }
        // Always update the path
        const audioFileName = `${i + 1}.aac`;
        const audioPath = path.join(audioDir, audioFileName);
        patch[3] =
          "/assets/patches/" +
          path.relative(patchesDir, audioPath).replace(/\\/g, "/");
      }
    }

    fs.writeFileSync(jsonPath, JSON.stringify(patches, null, 2));
  }

  processDirectory(patchesDir);
}

extractAudioFromPatches();
