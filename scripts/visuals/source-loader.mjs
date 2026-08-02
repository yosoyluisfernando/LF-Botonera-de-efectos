import { execFileSync } from 'node:child_process';
import { existsSync, mkdirSync } from 'node:fs';
import { mkdtemp } from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';

export async function loadSources(config) {
  const supplied = process.env.LF_VISUAL_SOURCE_ROOT;
  const workspace = supplied || await mkdtemp(path.join(os.tmpdir(), 'lf-visual-packs-'));
  const tabler = path.join(workspace, 'tabler');
  const gameIcons = path.join(workspace, 'game-icons');
  if (!existsSync(tabler)) clone(config.tabler.repository, config.tabler.tag, tabler);
  if (!existsSync(gameIcons)) cloneCommit(
    config.gameIcons.repository, config.gameIcons.commit, gameIcons);
  assertCommit(tabler, config.tabler.commit, 'tabler');
  assertCommit(gameIcons, config.gameIcons.commit, 'game-icons');
  return { workspace, tabler, gameIcons };
}

function clone(repository, reference, destination) {
  execFileSync('git', [
    'clone', '--depth', '1', '--branch', reference, repository, destination,
  ], { stdio: 'inherit' });
}

function cloneCommit(repository, commit, destination) {
  mkdirSync(destination, { recursive: true });
  execFileSync('git', ['-C', destination, 'init'], { stdio: 'inherit' });
  execFileSync(
    'git', ['-C', destination, 'remote', 'add', 'origin', repository],
    { stdio: 'inherit' });
  execFileSync(
    'git', ['-C', destination, 'fetch', '--depth', '1', 'origin', commit],
    { stdio: 'inherit' });
  execFileSync(
    'git', ['-C', destination, 'checkout', '--detach', 'FETCH_HEAD'],
    { stdio: 'inherit' });
}

function assertCommit(repository, expected, name) {
  const actual = execFileSync(
    'git', ['-C', repository, 'rev-parse', 'HEAD'], { encoding: 'utf8' }).trim();
  if (actual !== expected) throw new Error(`${name}_commit:${actual}`);
}
