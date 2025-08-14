import path from 'path';

export function repoDir(cwd = process.cwd()) {
  return cwd;
}

export function flowDir(cwd = process.cwd()) {
  return path.join(repoDir(cwd), '.flow');
}

export function objectsDir(cwd = process.cwd()) {
  return path.join(flowDir(cwd), 'objects');
}

export function indexPath(cwd = process.cwd()) {
  return path.join(flowDir(cwd), 'index.json');
}

export function commitsPath(cwd = process.cwd()) {
  return path.join(flowDir(cwd), 'commits.json');
}

export function repoMetaPath(cwd = process.cwd()) {
  return path.join(flowDir(cwd), 'repo.json');
}
