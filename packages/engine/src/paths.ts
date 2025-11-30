import { existsSync } from 'fs';
import { dirname, join, resolve } from 'path';
import { fileURLToPath } from 'url';

const FILE_URL_STRING_REGEXP = /^file:/i;

export function findProjectRoot(fromDir: string): string {
  let dir = FILE_URL_STRING_REGEXP.test(fromDir)
    ? fileURLToPath(fromDir)
    : fromDir;

  while (true) {
    if (existsSync(join(dir, 'package.json'))) {
      return dir;
    }

    const parent = dirname(dir);
    if (parent === dir) {
      return dir;
    }
    dir = parent;
  }
}

const MODULE_DIR = findProjectRoot(import.meta.url);

const VULFRAM_CORE_LIBS: Record<string, any> = {
  darwin: {
    arm64: 'assets/arm64/darwin/vulfram-core.dylib',
    x64: 'assets/x64/darwin/vulfram-core.dylib',
  },
  linux: {
    arm64: 'assets/arm64/linux/vulfram-core.so',
    x64: 'assets/x64/linux/vulfram-core.so',
  },
  win32: {
    arm64: 'assets/arm64/win32/vulfram-core.dll',
    x64: 'assets/x64/win32/vulfram-core.dll',
  },
};

function getDynamicLibPath(path?: string) {
  if (!path) return null;
  const relative = join('../..', path);
  if (existsSync(relative)) return relative;
  const absolute = resolve(MODULE_DIR, path);
  if (existsSync(absolute)) return absolute;
  return path;
}

export const VULFRAM_CORE_PATH = getDynamicLibPath(
  VULFRAM_CORE_LIBS[process.platform]?.[process.arch],
);
