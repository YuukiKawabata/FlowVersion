import { promises as fs } from 'fs';
import path from 'path';

export async function readJsonSafe<T>(filePath: string, fallback: T): Promise<T> {
  try {
    const raw = await fs.readFile(filePath, 'utf-8');
    const txt = raw.replace(/^\uFEFF/, '');
    return JSON.parse(txt) as T;
  } catch (e) {
    try {
      const dir = path.dirname(filePath);
      const base = path.basename(filePath);
      const stamp = new Date().toISOString().replace(/[:.]/g, '-');
      const backup = path.join(dir, `${base}.corrupt-${stamp}.bak`);
      try { await fs.copyFile(filePath, backup); } catch {}
    } catch {}
    return fallback;
  }
}

export async function writeJsonSafe(filePath: string, data: unknown) {
  const dir = path.dirname(filePath);
  await fs.mkdir(dir, { recursive: true });
  const temp = path.join(dir, `.${path.basename(filePath)}.tmp`);
  try {
    const stamp = new Date().toISOString().replace(/[:.]/g, '-');
    await fs.copyFile(filePath, `${filePath}.bak-${stamp}`);
  } catch {}
  await fs.writeFile(temp, JSON.stringify(data, null, 2), 'utf-8');
  await fs.rename(temp, filePath);
}
