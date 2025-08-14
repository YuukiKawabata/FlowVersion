import { promises as fs } from 'fs';
import path from 'path';
import { createHash } from 'crypto';
import { objectsDir } from './paths';

export async function ensureDirs(cwd = process.cwd()) {
  await fs.mkdir(objectsDir(cwd), { recursive: true });
}

export async function writeObject(data: Buffer | string, cwd = process.cwd()) {
  const buf = Buffer.isBuffer(data) ? data : Buffer.from(data);
  const sha = createHash('sha256').update(buf).digest('hex');
  const dir = path.join(objectsDir(cwd), sha.slice(0, 2));
  const file = path.join(dir, sha.slice(2));
  await fs.mkdir(dir, { recursive: true });
  try { await fs.access(file); } catch { await fs.writeFile(file, buf); }
  return sha;
}

export async function readObject(sha: string, cwd = process.cwd()) {
  const dir = path.join(objectsDir(cwd), sha.slice(0, 2));
  const file = path.join(dir, sha.slice(2));
  return fs.readFile(file);
}
