import { describe, it, expect, beforeAll } from 'vitest';
import { readFileSync, existsSync } from 'fs';
import { join } from 'path';

// Import the NAPI functions
import { parseManifestSync, parseManifestAsync, parseManifestBuffer } from '../index.js';

describe('NAPI Manifest Parser', () => {
    const manifestPath = join(__dirname, '..', 'test-manifests', 'valid-small.manifest');
    const jsonManifestPath = join(__dirname, '..', 'test-manifests', 'valid-json-format.manifest');
    let manifestBuffer: Buffer;
    let jsonManifestBuffer: Buffer;

    beforeAll(() => {
        // Check if manifest file exists
        if (!existsSync(manifestPath)) {
            throw new Error(`Manifest file not found at ${manifestPath}`);
        }

        // Check if JSON manifest file exists
        if (!existsSync(jsonManifestPath)) {
            throw new Error(`JSON manifest file not found at ${jsonManifestPath}`);
        }

        // Read the manifest files as buffers
        manifestBuffer = readFileSync(manifestPath);
        jsonManifestBuffer = readFileSync(jsonManifestPath);
    });

    describe('parseManifestSync', () => {
        it('should parse manifest file synchronously', () => {
            const result = parseManifestSync(manifestPath);

            expect(result).toBeDefined();
            expect(result.header).toBeDefined();
            expect(result.meta).toBeDefined();
            expect(result.chunkList).toBeDefined();
            expect(result.fileList).toBeDefined();
        });

        it('should have valid header structure', () => {
            const result = parseManifestSync(manifestPath);
            const { header } = result;


            expect(header.headerSize).toBeTypeOf('number');
            expect(header.dataSizeUncompressed).toBeTypeOf('number');
            expect(header.dataSizeCompressed).toBeTypeOf('number');
            expect(header.sha1Hash).toBeTypeOf('string');
            expect(header.storedAs).toBeTypeOf('number');
            expect(header.version).toBeTypeOf('number');

            // Validate SHA1 hash format (40 hex characters)
            expect(header.sha1Hash).toMatch(/^[a-fA-F0-9]{40}$/);
        });

        it('should have valid meta structure', () => {
            const result = parseManifestSync(manifestPath);
            const { meta } = result;

            expect(meta?.dataSize).toBeTypeOf('number');
            expect(meta?.dataVersion).toBeTypeOf('number');
            expect(meta?.featureLevel).toBeTypeOf('number');
            expect(meta?.isFileData).toBeTypeOf('boolean');
            expect(meta?.appId).toBeTypeOf('number');
            expect(meta?.appName).toBeTypeOf('string');
            expect(meta?.buildVersion).toBeTypeOf('string');
            expect(meta?.launchExe).toBeTypeOf('string');
            expect(meta?.launchCommand).toBeTypeOf('string');
            expect(Array.isArray(meta?.prereqIds)).toBe(true);
            expect(meta?.prereqName).toBeTypeOf('string');
            expect(meta?.prereqPath).toBeTypeOf('string');
            expect(meta?.prereqArgs).toBeTypeOf('string');
        });

        it('should have valid chunk data list', () => {
            const result = parseManifestSync(manifestPath);
            const { chunkList } = result;

            expect(Array.isArray(chunkList?.elements)).toBe(true);

            if (chunkList && chunkList?.elements.length > 0) {
                const chunk = chunkList?.elements[0];
                expect(chunk?.guid).toBeTypeOf('string');
                expect(chunk?.hash).toBeTypeOf('string');
                expect(chunk?.shaHash).toBeTypeOf('string');
                expect(chunk?.group).toBeTypeOf('number');
                expect(chunk?.windowSize).toBeTypeOf('number');
                expect(chunk?.fileSize).toBeTypeOf('string');

                // Validate GUID format
                expect(chunk?.guid).toMatch(/^[a-fA-F0-9]{8}-[a-fA-F0-9]{4}-[a-fA-F0-9]{4}-[a-fA-F0-9]{4}-[a-fA-F0-9]{12}$/i);
                // Validate SHA1 hash format
                expect(chunk?.shaHash).toMatch(/^[a-fA-F0-9]{40}$/);
            }
        });

        it('should have valid file manifest list', () => {
            const result = parseManifestSync(manifestPath);
            const { fileList } = result;

            expect(Array.isArray(fileList?.fileManifestList)).toBe(true);

            if (fileList && fileList?.fileManifestList.length > 0) {
                const file = fileList?.fileManifestList[0];
                expect(file.filename).toBeTypeOf('string');
                expect(file.symlinkTarget).toBeTypeOf('string');
                expect(file.shaHash).toBeTypeOf('string');
                expect(file.fileMetaFlags).toBeTypeOf('number');
                expect(Array.isArray(file.installTags)).toBe(true);
                expect(Array.isArray(file.chunkParts)).toBe(true);
                expect(file.fileSize).toBeTypeOf('number');

                // Validate SHA1 hash format
                expect(file.shaHash).toMatch(/^[a-fA-F0-9]{40}$/);

                // Validate chunk parts if they exist
                if (file.chunkParts.length > 0) {
                    const chunkPart = file.chunkParts[0];
                    expect(chunkPart.chunk?.guid).toBeTypeOf('string');
                    expect(chunkPart.offset).toBeTypeOf('number');
                    expect(chunkPart.size).toBeTypeOf('number');
                    expect(chunkPart.chunk?.guid).toMatch(/^[a-fA-F0-9]{8}-[a-fA-F0-9]{4}-[a-fA-F0-9]{4}-[a-fA-F0-9]{4}-[a-fA-F0-9]{12}$/i);
                }
            }
        });

        it('should handle non-existent file gracefully', () => {
            // The function might return undefined or throw an error for non-existent file
            try {
                const result = parseManifestSync('/non/existent/file.manifest');
                expect(result).toBeUndefined();
            } catch (error) {
                expect(error).toBeDefined();
            }
        });

        it('should handle invalid file gracefully', () => {
            // The function might return undefined or throw an error for invalid file
            try {
                const result = parseManifestSync(__filename); // This TypeScript file is not a valid manifest
                expect(result).toBeUndefined();
            } catch (error) {
                expect(error).toBeDefined();
            }
        });
    });

    describe('parseManifestAsync', () => {
        it('should parse manifest file asynchronously', async () => {
            const result = await parseManifestAsync(manifestPath);

            expect(result).toBeDefined();
            expect(result.header).toBeDefined();
            expect(result.meta).toBeDefined();
            expect(result.chunkList).toBeDefined();
            expect(result.fileList).toBeDefined();
        });

        it('should return same result as sync version', async () => {
            const asyncResult = await parseManifestAsync(manifestPath);
            const syncResult = parseManifestSync(manifestPath);

            expect(asyncResult).toEqual(syncResult);
        });

        it('should handle non-existent file gracefully', async () => {
            // The function might return undefined or reject for non-existent file
            try {
                const result = await parseManifestAsync('/non/existent/file.manifest');
                expect(result).toBeUndefined();
            } catch (error) {
                expect(error).toBeDefined();
            }
        });

        it('should handle invalid file gracefully', async () => {
            // The function might return undefined or reject for invalid file
            try {
                const result = await parseManifestAsync(__filename);
                expect(result).toBeUndefined();
            } catch (error) {
                expect(error).toBeDefined();
            }
        });
    });

    describe('parseManifestBuffer', () => {
        it('should parse manifest from buffer', () => {
            const result = parseManifestBuffer(manifestBuffer);

            expect(result).toBeDefined();
            expect(result.header).toBeDefined();
            expect(result.meta).toBeDefined();
            expect(result.chunkList).toBeDefined();
            expect(result.fileList).toBeDefined();
        });

        it('should return same result as file-based parsing', () => {
            const bufferResult = parseManifestBuffer(manifestBuffer);
            const fileResult = parseManifestSync(manifestPath);

            expect(bufferResult).toEqual(fileResult);
        });

        it('should handle empty buffer gracefully', () => {
            const emptyBuffer = Buffer.alloc(0);

            // The function might return undefined or throw an error for empty buffer
            // Let's test what actually happens
            try {
                const result = parseManifestBuffer(emptyBuffer);
                // If it doesn't throw, result should be undefined or have some indication of failure
                expect(result).toBeUndefined();
            } catch (error) {
                // If it throws, that's also acceptable behavior
                expect(error).toBeDefined();
            }
        });

        it('should handle invalid buffer gracefully', () => {
            const invalidBuffer = Buffer.from('invalid manifest data');

            // The function might return undefined or throw an error for invalid buffer
            // Let's test what actually happens
            try {
                const result = parseManifestBuffer(invalidBuffer);
                // If it doesn't throw, result should be undefined or have some indication of failure
                expect(result).toBeUndefined();
            } catch (error) {
                // If it throws, that's also acceptable behavior
                expect(error).toBeDefined();
            }
        });

        it('should handle large buffers', () => {
            // Test with the actual manifest buffer which should be reasonably large
            expect(manifestBuffer.length).toBeGreaterThan(0);

            const result = parseManifestBuffer(manifestBuffer);
            expect(result).toBeDefined();
        });
    });

    describe('JSON Manifest Parsing', () => {
        it('should parse JSON manifest file synchronously', () => {
            const result = parseManifestSync(jsonManifestPath);

            expect(result).toBeDefined();
            expect(result.header).toBeDefined();
            expect(result.meta).toBeDefined();
            expect(result.chunkList).toBeDefined();
            expect(result.fileList).toBeDefined();
        });

        it('should parse JSON manifest file asynchronously', async () => {
            const result = await parseManifestAsync(jsonManifestPath);

            expect(result).toBeDefined();
            expect(result.header).toBeDefined();
            expect(result.meta).toBeDefined();
            expect(result.chunkList).toBeDefined();
            expect(result.fileList).toBeDefined();
        });

        it('should parse JSON manifest from buffer', () => {
            const result = parseManifestBuffer(jsonManifestBuffer);

            expect(result).toBeDefined();
            expect(result.header).toBeDefined();
            expect(result.meta).toBeDefined();
            expect(result.chunkList).toBeDefined();
            expect(result.fileList).toBeDefined();
        });

        it('should have valid JSON manifest meta structure', () => {
            const result = parseManifestSync(jsonManifestPath);
            const { meta } = result;

            expect(meta?.appName).toBeTypeOf('string');
            expect(meta?.buildVersion).toBeTypeOf('string');
            expect(meta?.launchExe).toBeTypeOf('string');
            expect(meta?.appName).toBe('32dbb6444ce14e9198129b746c0d056f');
            expect(meta?.buildVersion).toBe('1.4.30.0');
            expect(meta?.launchExe).toBe('TheFalconeer.exe');
        });

        it('should have valid JSON manifest file list', () => {
            const result = parseManifestSync(jsonManifestPath);
            const { fileList } = result;

            expect(Array.isArray(fileList?.fileManifestList)).toBe(true);
            expect(fileList?.fileManifestList.length).toBeGreaterThan(0);

            if (fileList && fileList?.fileManifestList.length > 0) {
                const file = fileList?.fileManifestList[0];
                expect(file.filename).toBeTypeOf('string');
                expect(file.filename).toBe('MonoBleedingEdge/EmbedRuntime/mono-2.0-bdwgc.dll');
                expect(file.fileSize).toBeTypeOf('number');
                expect(file.fileSize).toBe(101003264); // Updated to actual file size
                expect(Array.isArray(file.chunkParts)).toBe(true);
            }
        });

        it('should have consistent results across all JSON parsing methods', async () => {
            const syncResult = parseManifestSync(jsonManifestPath);
            const asyncResult = await parseManifestAsync(jsonManifestPath);
            const bufferResult = parseManifestBuffer(jsonManifestBuffer);

            // Compare key fields instead of exact object equality
            expect(syncResult.meta?.appName).toBe(asyncResult.meta?.appName);
            expect(syncResult.meta?.buildVersion).toBe(asyncResult.meta?.buildVersion);
            expect(syncResult.meta?.launchExe).toBe(asyncResult.meta?.launchExe);
            expect(syncResult.fileList?.fileManifestList.length).toBe(asyncResult.fileList?.fileManifestList.length);

            expect(syncResult.meta?.appName).toBe(bufferResult.meta?.appName);
            expect(syncResult.meta?.buildVersion).toBe(bufferResult.meta?.buildVersion);
            expect(syncResult.meta?.launchExe).toBe(bufferResult.meta?.launchExe);
            expect(syncResult.fileList?.fileManifestList.length).toBe(bufferResult.fileList?.fileManifestList.length);

            expect(asyncResult.meta?.appName).toBe(bufferResult.meta?.appName);
            expect(asyncResult.meta?.buildVersion).toBe(bufferResult.meta?.buildVersion);
            expect(asyncResult.meta?.launchExe).toBe(bufferResult.meta?.launchExe);
            expect(asyncResult.fileList?.fileManifestList.length).toBe(bufferResult.fileList?.fileManifestList.length);
        });
    });

    describe('Cross-function consistency', () => {
        it('all parsing methods should return identical results', async () => {
            const syncResult = parseManifestSync(manifestPath);
            const asyncResult = await parseManifestAsync(manifestPath);
            const bufferResult = parseManifestBuffer(manifestBuffer);

            expect(syncResult).toEqual(asyncResult);
            expect(syncResult).toEqual(bufferResult);
            expect(asyncResult).toEqual(bufferResult);
        });
    });

    describe('Performance tests', () => {
        it('sync parsing should complete within reasonable time', () => {
            const start = Date.now();
            const result = parseManifestSync(manifestPath);
            const duration = Date.now() - start;

            expect(result).toBeDefined();
            expect(result.chunkList).toBeDefined();
            expect(result.fileList).toBeDefined();
            // Should complete within 5 seconds for most manifest files
            expect(duration).toBeLessThan(5000);
        });

        it('async parsing should complete within reasonable time', async () => {
            const start = Date.now();
            const result = await parseManifestAsync(manifestPath);
            const duration = Date.now() - start;

            expect(result).toBeDefined();
            expect(result.chunkList).toBeDefined();
            expect(result.fileList).toBeDefined();
            // Should complete within 5 seconds for most manifest files
            expect(duration).toBeLessThan(5000);
        });

        it('buffer parsing should complete within reasonable time', () => {
            const start = Date.now();
            const result = parseManifestBuffer(manifestBuffer);
            const duration = Date.now() - start;

            expect(result).toBeDefined();
            expect(result.chunkList).toBeDefined();
            expect(result.fileList).toBeDefined();
            // Should complete within 5 seconds for most manifest files
            expect(duration).toBeLessThan(5000);
        });
    });
});