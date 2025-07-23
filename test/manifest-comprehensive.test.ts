import { describe, it, expect } from 'vitest';
import { readdirSync } from 'fs';
import { join } from 'path';
import { parseManifestAsync } from '../index.js';

describe('Comprehensive Manifest Testing', () => {
    const testManifestsDir = join(__dirname, '..', 'test-manifests');

    // Get all manifest files
    const manifestFiles = readdirSync(testManifestsDir).filter(file => file.endsWith('.manifest'));

    describe('All Test Manifests', () => {
        it('should have test manifest files available', () => {
            expect(manifestFiles.length).toBeGreaterThan(0);
            console.log(`Found ${manifestFiles.length} test manifest files`);
        });

        manifestFiles.forEach(file => {
            describe(`Testing: ${file}`, () => {
                const filePath = join(testManifestsDir, file);

                it('should parse successfully', async () => {
                    let manifest = await parseManifestAsync(filePath);

                    expect(manifest).toBeDefined();

                    // Log detailed information
                    console.log(`\n=== ${file} ===`);
                    console.log('✅ SUCCESS: Parsed successfully');

                    if (manifest && manifest.meta) {
                        console.log(`   App Name: ${manifest.meta.appName}`);
                        console.log(`   Build Version: ${manifest.meta.buildVersion}`);
                    }

                    if (manifest && manifest.chunkList) {
                        const chunkCount = manifest.chunkList.elements ?
                            manifest.chunkList.elements.length :
                            manifest.chunkList.count || 0;
                        console.log(`   Chunks: ${chunkCount}`);
                    }

                    if (manifest && manifest.fileList) {
                        const fileCount = manifest.fileList.fileManifestList ?
                            manifest.fileList.fileManifestList.length :
                            manifest.fileList.count || 0;
                        console.log(`   Files: ${fileCount}`);
                    }

                    // Calculate download size by summing all chunk file sizes (compressed size)
                    const downloadSizeBytes =
                        manifest.chunkList?.elements.reduce((sum, chunk) => {
                            return sum + Number.parseInt(chunk.fileSize);
                        }, 0) || 0;

                    // Calculate installed size by summing window sizes of all chunks (uncompressed size)
                    const installedSizeBytes =
                        manifest.chunkList?.elements.reduce((sum, chunk) => {
                            return sum + (chunk.windowSize ?? chunk.fileSize);
                        }, 0) || 0;

                    console.log(`Download size: ${downloadSizeBytes / 1024 / 1024 / 1024} GB`);
                    console.log(`Installed size: ${installedSizeBytes / 1024 / 1024 / 1024} GB`);

                    expect(downloadSizeBytes, "download size").toBeGreaterThan(0);
                    expect(installedSizeBytes, "installed size").toBeGreaterThan(0);

                    expect(downloadSizeBytes, "download size").toBeLessThanOrEqual(installedSizeBytes);
                });

                it('should have valid structure', async () => {
                    const manifest = await parseManifestAsync(filePath);

                    // Basic structure validation
                    expect(manifest.header).toBeDefined();
                    expect(manifest.header.version).toBeTypeOf('number');

                    // SHA1 hash validation - some manifests may have empty hash
                    if (manifest.header.sha1Hash && manifest.header.sha1Hash.length > 0) {
                        expect(manifest.header.sha1Hash).toMatch(/^[a-fA-F0-9]{40}$/);
                    }

                    // Meta validation
                    if (manifest.meta) {
                        expect(manifest.meta.appName).toBeTypeOf('string');
                        expect(manifest.meta.buildVersion).toBeTypeOf('string');
                    }

                    // Chunk list validation
                    if (manifest.chunkList) {
                        expect(manifest.chunkList.count).toBeTypeOf('number');
                        if (manifest.chunkList.elements) {
                            expect(Array.isArray(manifest.chunkList.elements)).toBe(true);
                        }
                    }

                    // File list validation
                    if (manifest.fileList) {
                        expect(manifest.fileList.count).toBeTypeOf('number');
                        if (manifest.fileList.fileManifestList) {
                            expect(Array.isArray(manifest.fileList.fileManifestList)).toBe(true);
                        }
                    }
                });

                // Specific tests based on file type
                if (file.includes('corrupted')) {
                    it('should handle corrupted data gracefully', async () => {
                        const manifest = await parseManifestAsync(filePath);

                        // Should still parse basic structure even if corrupted
                        expect(manifest.header).toBeDefined();

                        // May have warnings but should not crash
                        expect(manifest).toBeDefined();
                    });
                }

                if (file.includes('truncated')) {
                    it('should handle truncated data with EOF tolerance', async () => {
                        const manifest = await parseManifestAsync(filePath);

                        // Should parse successfully with our EOF tolerance fix
                        expect(manifest.header).toBeDefined();
                        expect(manifest.meta).toBeDefined();
                        expect(manifest.chunkList).toBeDefined();
                        expect(manifest.fileList).toBeDefined();
                    });
                }

                if (file.includes('json')) {
                    it('should handle JSON format manifests', async () => {
                        const manifest = await parseManifestAsync(filePath);

                        // JSON manifests should have all standard fields
                        expect(manifest.header).toBeDefined();
                        expect(manifest.meta).toBeDefined();
                        expect(manifest.chunkList).toBeDefined();
                        expect(manifest.fileList).toBeDefined();

                        // Should have launch executable info
                        if (manifest.meta) {
                            expect(manifest.meta.launchExe).toBeTypeOf('string');
                        }
                    });
                }

                if (file.includes('small')) {
                    it('should handle small manifests efficiently', async () => {
                        const manifest = await parseManifestAsync(filePath);

                        // Small manifests should have reasonable counts
                        expect(manifest.chunkList?.count).toBeLessThan(1000);
                        expect(manifest.fileList?.count).toBeLessThan(500);
                    });
                }
            });
        });
    });

    describe('Error Handling', () => {
        it('should handle non-existent files gracefully', async () => {
            // The function might return undefined or throw an error for non-existent file
            try {
                const result = await parseManifestAsync('/non/existent/file.manifest');
                expect(result).toBeUndefined();
            } catch (error) {
                expect(error).toBeDefined();
            }
        });

        it('should handle invalid file formats gracefully', async () => {
            // The function might return undefined or throw an error for invalid file
            try {
                const result = await parseManifestAsync(__filename); // This TypeScript file
                expect(result).toBeUndefined();
            } catch (error) {
                expect(error).toBeDefined();
            }
        });
    });

    describe('Performance', () => {
        it('should parse manifests within reasonable time', async () => {
            const startTime = Date.now();

            await Promise.all(manifestFiles.map(file => {
                const filePath = join(testManifestsDir, file);
                return parseManifestAsync(filePath);
            }));

            const endTime = Date.now();
            const totalTime = endTime - startTime;

            console.log(`\nParsed ${manifestFiles.length} manifests in ${totalTime}ms`);
            console.log(`Average time per manifest: ${(totalTime / manifestFiles.length).toFixed(2)}ms`);

            // Should parse all manifests in under 10 seconds
            expect(totalTime).toBeLessThan(10000);
        });
    });
});