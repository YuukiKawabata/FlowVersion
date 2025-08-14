#!/usr/bin/env node
import { Command } from 'commander';
import pc from 'picocolors';
import { promises as fs } from 'fs';
import path from 'path';
import { ensureDirs, writeObject, readObject } from './utils/objectStore';
import { indexPath, flowDir } from './utils/paths';
import { loadConfig, saveConfig } from './utils/config';
import { readJsonSafe, writeJsonSafe } from './utils/json';
import { createTwoFilesPatch } from 'diff';
import fg from 'fast-glob';

type IndexEntry = {
  path: string;
  intention: string | null;
  sha: string;
  size: number;
  mtime: number;
  addedAt: number;
};

type Commit = {
  id: string;
  intention: string | null;
  confidence: number;
  timestamp: string;
  goal?: string | null;
  context?: string | null;
  impact?: string | null;
  snapshot?: IndexEntry[];
};

function normalizeRel(file: string, cwd = process.cwd()) {
  const rel = path.relative(cwd, path.isAbsolute(file) ? file : path.join(cwd, file));
  // 正規化してスラッシュに統一
  return rel.split(path.sep).join('/');
}

async function loadIndex(idxPath = indexPath()): Promise<IndexEntry[]> {
  return readJsonSafe<IndexEntry[]>(idxPath, []);
}

async function saveIndex(entries: IndexEntry[], idxPath = indexPath()) {
  await writeJsonSafe(idxPath, entries);
}

const program = new Command();
program
  .name('flow-ts')
  .description('FlowVersion TypeScript CLI (lightweight)')
  .version('0.1.0');

// flow-ts init --name "my-project"
program
  .command('init')
  .description('Initialize a new FlowVersion repo')
  .option('-n, --name <name>', 'Project name')
  .option('-d, --description <desc>', 'Project description')
  .action(async (opts) => {
  const repoDir = process.cwd();
  const fvDir = path.join(repoDir, '.flow');
  await fs.mkdir(fvDir, { recursive: true });
  await fs.mkdir(path.join(fvDir, 'objects'), { recursive: true });
    const meta = { name: opts.name ?? path.basename(repoDir), createdAt: new Date().toISOString() };
    await fs.writeFile(path.join(fvDir, 'repo.json'), JSON.stringify(meta, null, 2));
    await saveConfig({
      name: opts.name ?? path.basename(repoDir),
      description: opts.description ?? '',
      createdAt: new Date().toISOString(),
    });
    console.log(pc.green(`Initialized FlowVersion repo at ${fvDir}`));
  });

// flow config get/set/show
const config = program.command('config').description('Manage repository config');

config
  .command('show')
  .description('Show current configuration (config.yml)')
  .action(async () => {
    const cfg = await loadConfig();
    console.log(JSON.stringify(cfg, null, 2));
  });

config
  .command('get')
  .argument('<key>')
  .description('Get a config value')
  .action(async (key) => {
    const cfg = await loadConfig();
    console.log((cfg as any)?.[key] ?? '');
  });

config
  .command('set')
  .argument('<key>')
  .argument('<value>')
  .description('Set a config value')
  .action(async (key, value) => {
    const cfg = await loadConfig();
    (cfg as any)[key] = value;
    await saveConfig(cfg);
    console.log(pc.green(`Set ${key}`));
  });

// flow-ts add <file> --intention "..."
program
  .command('add')
  .argument('<file>')
  .option('-i, --intention <text>', 'Intention/description')
  .description('Stage file(s); supports glob patterns')
  .action(async (file, opts) => {
    const repoDir = process.cwd();
    const fvDir = path.join(repoDir, '.flow');
    await fs.mkdir(fvDir, { recursive: true });
    await ensureDirs();
    const idxPath = indexPath();
    const index = await loadIndex(idxPath);
    const patterns = [file];
    const entries = await fg(patterns, { dot: true, cwd: repoDir, onlyFiles: true, followSymbolicLinks: false });
    if (entries.length === 0) {
      console.log(pc.yellow(`No files matched: ${file}`));
      return;
    }
    let added = 0;
    for (const relPosix of entries.map((p: string) => p.split(path.sep).join('/'))) {
      const rel = normalizeRel(relPosix, repoDir);
      const abs = path.join(repoDir, rel);
      const stat = await fs.stat(abs);
      const blob = await fs.readFile(abs);
      const sha = await writeObject(blob);
      const intention = opts.intention ?? null;
      const entry: IndexEntry = { path: rel, intention, sha, size: stat.size, mtime: stat.mtimeMs, addedAt: Date.now() };
      const i = index.findIndex(e => e.path === rel);
      if (i >= 0) index[i] = entry; else index.push(entry);
      added++;
    }
    await saveIndex(index, idxPath);
    console.log(pc.cyan(`Added ${added} file(s)`));
  });

// flow-ts commit --intention "..." --confidence 0.8
program
  .command('commit')
  .option('-i, --intention <text>')
  .option('-c, --confidence <num>', '0..1', (v) => Number(v), 1)
  .option('--goal <text>', 'Goal of the change')
  .option('--context <text>', 'Context for the change')
  .option('--impact <text>', 'Expected impact')
  .action(async (opts) => {
    const repoDir = process.cwd();
    const fvDir = path.join(repoDir, '.flow');
    await fs.mkdir(fvDir, { recursive: true });
    const commitsPath = path.join(fvDir, 'commits.json');
    const idxPath = indexPath();
    const index = await loadIndex(idxPath);
    if (index.length === 0) {
      console.log(pc.yellow('Nothing to commit. Stage changes with `flow add <file>`.'));
      return;
    }
    const id = Math.random().toString(36).slice(2, 10);
    const commit: Commit = {
      id,
      intention: opts.intention ?? null,
      confidence: Math.max(0, Math.min(1, Number(opts.confidence ?? 1))),
      timestamp: new Date().toISOString(),
      goal: opts.goal ?? null,
      context: opts.context ?? null,
      impact: opts.impact ?? null,
      snapshot: index,
    };
    const commits = await readJsonSafe<Commit[]>(commitsPath, []);
    commits.push(commit);
    await writeJsonSafe(commitsPath, commits);
    await saveIndex([], idxPath);
    console.log(pc.green(`Committed ${id}`));
  });

// flow-ts status
program
  .command('status')
  .description('Show staged files and repository status')
  .action(async () => {
    const idxPath = indexPath();
    const index = await loadIndex(idxPath);
    if (index.length === 0) {
      console.log('Nothing staged.');
      return;
  }
    console.log(pc.bold('Staged files:'));
    for (const e of index) {
      const abs = path.join(process.cwd(), e.path);
      let status = 'staged';
      try {
        const buf = await fs.readFile(abs);
        // 簡易に sha 比較
        const tmpSha = await writeObject(buf); // 既存オブジェクトを再利用（書込みは同一なら no-op）
        if (tmpSha !== e.sha) status = 'modified';
      } catch {
        status = 'deleted';
      }
      console.log(` - ${e.path}  ${e.sha?.slice(0,8) ?? ''}  ${e.intention ?? ''}  [${status}]`);
    }
  });

// Unstage a file
program
  .command('unstage')
  .argument('<file>')
  .description('Remove a file from staging area')
  .action(async (file) => {
    const idxPath = indexPath();
    const index = await loadIndex(idxPath);
    const rel = normalizeRel(file);
    const next = index.filter(e => e.path !== rel);
    const removed = index.length - next.length;
    await saveIndex(next, idxPath);
    console.log(removed > 0 ? pc.yellow(`Unstaged ${rel}`) : pc.dim(`No match: ${rel}`));
  });

// Clear staging area
program
  .command('reset')
  .description('Clear staging area')
  .action(async () => {
    await saveIndex([], indexPath());
    console.log(pc.yellow('Cleared staging area.'));
  });

// Show diff between working tree and staged blob
program
  .command('diff')
  .argument('<file>')
  .description('Show diff between working file and staged version')
  .action(async (file) => {
    const idx = await loadIndex(indexPath());
    const rel = normalizeRel(file);
    const e = idx.find(x => x.path === rel);
    if (!e) {
      console.log(pc.yellow(`Not staged: ${rel}`));
      return;
    }
    const cwd = process.cwd();
    const abs = path.join(cwd, rel);
    let left = '';
    try { left = (await fs.readFile(abs)).toString('utf8'); } catch { left = ''; }
    const right = (await readObject(e.sha)).toString('utf8');
    const patch = createTwoFilesPatch(rel + ' (WORKTREE)', rel + ' (STAGED)', left, right, undefined, undefined, { context: 3 });
    console.log(patch);
  });

program
  .command('log')
  .description('Show commit log')
  .action(async () => {
    const commitsPath = path.join(process.cwd(), '.flow', 'commits.json');
    const commits = await readJsonSafe<Commit[]>(commitsPath, []);
    if (commits.length === 0) { console.log('No commits yet.'); return; }
    const idW = 8;
    const confColor = (v: number) => v >= 0.8 ? pc.green : v >= 0.5 ? pc.yellow : pc.red;
    for (const c of commits) {
      const idShort = c.id.slice(0, idW);
      const conf = confColor(c.confidence)(`conf=${c.confidence}`);
      const intent = c.intention ?? '';
      console.log(`${pc.bold(idShort)}  ${c.timestamp}  ${intent}  ${conf}`);
    }
  });

function formatId(id: string) { return pc.bold(id); }
function findByPrefix<T extends { id: string }>(items: T[], q: string): { item: T | null; ambiguous: boolean; candidates: T[] } {
  const exact = items.find(x => x.id === q);
  if (exact) return { item: exact, ambiguous: false, candidates: [] };
  const list = items.filter(x => x.id.startsWith(q));
  if (list.length === 1) return { item: list[0], ambiguous: false, candidates: [] };
  if (list.length > 1) return { item: null, ambiguous: true, candidates: list };
  return { item: null, ambiguous: false, candidates: [] };
}

// Show commit details or a file content from the commit snapshot
program
  .command('show')
  .argument('<id>', 'commit id or prefix')
  .argument('[file]', 'optional file path to show from snapshot')
  .description('Show commit details or file contents from a commit')
  .action(async (id, file) => {
  const commitsFile = path.join(flowDir(), 'commits.json');
  let commits = await readJsonSafe<Commit[]>(commitsFile, []);
    if (commits.length === 0) { console.log('No commits.'); return; }
    const res = findByPrefix(commits, id);
    if (!res.item) {
      if (res.ambiguous) {
        const tips = res.candidates.slice(0, 5).map(c => ` - ${c.id.slice(0,8)}  ${c.intention ?? ''}`).join('\n');
        console.log(pc.red('Ambiguous commit id. Be more specific. Candidates:'));
        if (tips) console.log(tips);
      } else {
        console.log(pc.red('Commit not found.'));
      }
      return;
    }
    const commit = res.item;
    console.log(`${formatId(commit.id)}  ${commit.timestamp}  ${commit.intention ?? ''}  (conf=${commit.confidence})`);
    if (commit.goal || commit.context || commit.impact) {
      if (commit.goal) console.log(`  goal: ${commit.goal}`);
      if (commit.context) console.log(`  context: ${commit.context}`);
      if (commit.impact) console.log(`  impact: ${commit.impact}`);
    }
    const snapshot: IndexEntry[] | undefined = commit.snapshot;
    if (!snapshot || snapshot.length === 0) {
      console.log(pc.dim('No snapshot stored for this commit.'));
      return;
    }
    if (!file) {
      for (const e of snapshot) {
        console.log(` - ${e.path}  ${e.sha.slice(0,8)}  ${e.intention ?? ''}`);
      }
      return;
    }
    const rel = normalizeRel(file);
    const entry = snapshot.find(e => e.path === rel);
    if (!entry) { console.log(pc.yellow(`File not found in snapshot: ${rel}`)); return; }
    const buf = await readObject(entry.sha);
    process.stdout.write(buf);
  });

// Restore files from a commit snapshot
program
  .command('checkout')
  .argument('<id>', 'commit id or prefix')
  .option('-p, --path <file>', 'restore a single file from the commit')
  .option('-A, --all', 'restore all files in the commit')
  .option('-f, --force', 'overwrite existing files')
  .option('-n, --dry-run', 'do not write files, only show actions')
  .description('Restore files from a commit snapshot into the working directory')
  .action(async (id, opts) => {
    const commitsFile = path.join(flowDir(), 'commits.json');
    let commits = await readJsonSafe<Commit[]>(commitsFile, []);
    if (commits.length === 0) { console.log('No commits.'); return; }
    const res = findByPrefix(commits, id);
    if (!res.item) {
      if (res.ambiguous) {
        const tips = res.candidates.slice(0, 5).map(c => ` - ${c.id.slice(0,8)}  ${c.intention ?? ''}`).join('\n');
        console.log(pc.red('Ambiguous commit id. Be more specific. Candidates:'));
        if (tips) console.log(tips);
      } else {
        console.log(pc.red('Commit not found.'));
      }
      return;
    }
    const commit = res.item;
    const snapshot: IndexEntry[] | undefined = commit.snapshot;
    if (!snapshot || snapshot.length === 0) { console.log(pc.dim('No snapshot stored for this commit.')); return; }
    const cwd = process.cwd();
    const force = !!opts.force;
    if (opts.path && opts.all) { console.log(pc.red('Use either --path or --all.')); return; }
    let targets: IndexEntry[] = [];
    if (opts.path) {
      const rel = normalizeRel(opts.path);
      const e = snapshot.find(x => x.path === rel);
      if (!e) { console.log(pc.yellow(`File not in commit: ${rel}`)); return; }
      targets = [e];
    } else if (opts.all) {
      targets = snapshot;
    } else {
      console.log(pc.yellow('Specify --path <file> or --all'));
      return;
    }
    let restored = 0, skipped = 0;
    const plan: { path: string; action: 'write' | 'skip-exists'; }[] = [];
    for (const e of targets) {
      const relOs = e.path.split('/').join(path.sep);
      const dest = path.join(cwd, relOs);
      await fs.mkdir(path.dirname(dest), { recursive: true });
      try {
        await fs.access(dest);
        if (!force) { skipped++; plan.push({ path: e.path, action: 'skip-exists' }); continue; }
      } catch {}
      if (!opts.dryRun) {
        const buf = await readObject(e.sha);
        await fs.writeFile(dest, buf);
      }
      plan.push({ path: e.path, action: 'write' });
      restored++;
    }
    if (opts.dryRun) {
      console.log(pc.bold('Dry run plan:'));
      for (const p of plan) console.log(` - ${p.action} ${p.path}`);
    }
    console.log(pc.green(`Restored: ${restored}  Skipped: ${skipped}`));
  });

program.parseAsync(process.argv);
