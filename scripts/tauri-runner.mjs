import { readFileSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const repoRoot = path.resolve(__dirname, '..');

function readPackageVersion() {
  const packageJsonPath = path.join(repoRoot, 'package.json');
  const packageJson = JSON.parse(readFileSync(packageJsonPath, 'utf8'));
  return String(packageJson.version ?? '').trim();
}

function toWindowsPrereleaseVersion(version) {
  const [cleanVersion] = version.split('+', 1);
  const separatorIndex = cleanVersion.indexOf('-');
  if (separatorIndex === -1) {
    return cleanVersion;
  }

  const core = cleanVersion.slice(0, separatorIndex);
  const prerelease = cleanVersion.slice(separatorIndex + 1);
  const numericToken = prerelease
    .split('.')
    .reverse()
    .find((token) => /^\d+$/.test(token));

  if (!numericToken) {
    throw new Error(
      `Windows MSI packaging requires a numeric prerelease identifier. Current version: ${version}`,
    );
  }

  const buildNumber = Number.parseInt(numericToken, 10);
  if (!Number.isInteger(buildNumber) || buildNumber < 0 || buildNumber > 65535) {
    throw new Error(
      `Windows MSI prerelease number must be between 0 and 65535. Current version: ${version}`,
    );
  }

  return `${core}-${buildNumber}`;
}

function shouldInjectWindowsBuildVersion(args, version) {
  if (process.platform !== 'win32') {
    return false;
  }

  if (!version.includes('-')) {
    return false;
  }

  if (args[0] !== 'build') {
    return false;
  }

  if (args.includes('--no-bundle')) {
    return false;
  }

  return true;
}

const tauriArgs = process.argv.slice(2);
const version = readPackageVersion();

if (tauriArgs.length === 0) {
  console.error('[tauri-runner] Missing Tauri command.');
  process.exit(1);
}

const extraConfigs = [];

if (shouldInjectWindowsBuildVersion(tauriArgs, version)) {
  const windowsVersion = toWindowsPrereleaseVersion(version);
  if (windowsVersion !== version) {
    extraConfigs.push(JSON.stringify({ version: windowsVersion }));
    console.log(
      `[tauri-runner] Windows prerelease build detected, overriding bundle version ${version} -> ${windowsVersion}`,
    );
  }
}

const tauriCliPath = path.join(repoRoot, 'node_modules', '@tauri-apps', 'cli', 'tauri.js');
const command = process.execPath;
const commandArgs = [tauriCliPath, ...tauriArgs];

for (const config of extraConfigs) {
  commandArgs.push('--config', config);
}

const result = spawnSync(command, commandArgs, {
  cwd: repoRoot,
  stdio: 'inherit',
});

if (result.error) {
  console.error(result.error.message);
  process.exit(1);
}

process.exit(result.status ?? 0);
