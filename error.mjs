import { parseManifestSync } from "./index.js";
import { readdirSync } from "fs";
import { join } from "path";

const testManifestsDir = "./test-manifests";
const manifestFiles = readdirSync(testManifestsDir).filter(file => file.endsWith('.manifest'));

console.log(`Testing ${manifestFiles.length} manifest files:\n`);

for (const file of manifestFiles) {
    const filePath = join(testManifestsDir, file);
    console.log(`\n=== Testing: ${file} ===`);

    try {
        const manifest = parseManifestSync(filePath);
        console.log(`✅ SUCCESS: Parsed successfully`);

        // Check the actual structure
        if (manifest.meta) {
            console.log(`   App Name: ${manifest.meta.appName}`);
            console.log(`   Build Version: ${manifest.meta.buildVersion}`);
        } else {
            console.log(`   App Name: ${manifest.appName || 'N/A'}`);
            console.log(`   Build Version: ${manifest.buildVersion || 'N/A'}`);
        }

        if (manifest.chunkList) {
            console.log(`   Chunks: ${manifest.chunkList.elements ? manifest.chunkList.elements.length : manifest.chunkList.count || 'N/A'}`);
        }
        
        if (manifest.fileList) {
            console.log(`   Files: ${manifest.fileList.fileManifestList ? manifest.fileList.fileManifestList.length : manifest.fileList.count || 'N/A'}`);
        }
    } catch (error) {
        console.log(`❌ FAILED: ${error.message}`);
        if (error.code) {
            console.log(`   Error Code: ${error.code}`);
        }
    }
}

console.log(`\n=== Test Summary Complete ===`);