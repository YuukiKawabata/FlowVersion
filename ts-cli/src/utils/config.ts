import { promises as fs } from 'fs';
import path from 'path';
import YAML from 'yaml';
import { flowDir } from './paths';

export type FlowConfig = {
  name?: string;
  description?: string;
  createdAt?: string;
  [k: string]: any;
};

export function configPath(cwd = process.cwd()) {
  return path.join(flowDir(cwd), 'config.yml');
}

export async function loadConfig(cwd = process.cwd()): Promise<FlowConfig> {
  const p = configPath(cwd);
  try {
    const txt = await fs.readFile(p, 'utf-8');
    return YAML.parse(txt) ?? {};
  } catch {
    return {};
  }
}

export async function saveConfig(cfg: FlowConfig, cwd = process.cwd()) {
  const p = configPath(cwd);
  const dir = path.dirname(p);
  await fs.mkdir(dir, { recursive: true });
  const doc = new YAML.Document(cfg);
  await fs.writeFile(p, doc.toString(), 'utf-8');
}
