#!/usr/bin/env node
/**
 * 为 Tauri updater 生成 latest.json，供 GitHub Release 使用。
 * 在 CI release 阶段运行：下载 mac/win artifacts 后，生成 latest.json 并随 Release 一起上传。
 *
 * 环境变量：
 *   GITHUB_REF_NAME  例如 v0.0.29
 *   GITHUB_REPOSITORY 例如 miaoxworld/openclaw-manager
 *
 * 用法: node scripts/generate-updater-latest.cjs [macDir] [winDir] [outPath]
 * 默认: openclaw-manager-macos openclaw-manager-windows release-assets/latest.json
 */
const fs = require('fs');
const path = require('path');

const tag = process.env.GITHUB_REF_NAME || '';
const repo = process.env.GITHUB_REPOSITORY || 'miaoxworld/openclaw-manager';
const version = tag.replace(/^v/, '') || '0.0.0';

const macDir = process.argv[2] || path.join(__dirname, '..', 'openclaw-manager-macos');
const winDir = process.argv[3] || path.join(__dirname, '..', 'openclaw-manager-windows');
const outPath = process.argv[4] || path.join(__dirname, '..', 'release-assets', 'latest.json');

const baseUrl = `https://github.com/${repo}/releases/download/${tag}`;

function findFiles(dir, pattern) {
  const results = [];
  if (!fs.existsSync(dir)) return results;
  const walk = (d) => {
    const entries = fs.readdirSync(d, { withFileTypes: true });
    for (const e of entries) {
      const full = path.join(d, e.name);
      if (e.isDirectory()) walk(full);
      else if (pattern.test(e.name)) results.push(full);
    }
  };
  walk(dir);
  return results;
}

function readSig(sigPath) {
  return fs.readFileSync(sigPath, 'utf8').trim();
}

const platforms = {};
let hasAny = false;

// macOS: .app.tar.gz + .app.tar.gz.sig（可能在 macos/ 或 dmg 同级）
const macTarGz = findFiles(macDir, /\.tar\.gz$/).find((f) => !f.endsWith('.sig'));
const macSig = macTarGz ? findFiles(macDir, /\.tar\.gz\.sig$/)[0] : null;
if (macTarGz && macSig) {
  const name = path.basename(macTarGz);
  platforms['darwin-x86_64'] = {
    url: `${baseUrl}/${encodeURIComponent(name)}`,
    signature: readSig(macSig),
  };
  hasAny = true;
}

// Windows: *-setup.exe + 同名的 .exe.sig（通常在 nsis/）
const winExe = findFiles(winDir, /-setup\.exe$/)[0] || findFiles(winDir, /\.exe$/)[0];
const winExeBasename = winExe ? path.basename(winExe) : '';
const winSig = winExe
  ? findFiles(winDir, /\.sig$/).find((f) => path.basename(f) === winExeBasename + '.sig')
  : null;
if (winExe && winSig) {
  const name = path.basename(winExe);
  platforms['windows-x86_64'] = {
    url: `${baseUrl}/${encodeURIComponent(name)}`,
    signature: readSig(winSig),
  };
  hasAny = true;
}

if (!hasAny) {
  console.error('generate-updater-latest: no updater artifacts found (mac .tar.gz+.sig or win .exe+.sig). Skip.');
  process.exit(0);
}

const pubDate = new Date().toISOString();
const latest = {
  version,
  notes: '',
  pub_date: pubDate,
  platforms,
};

const outDir = path.dirname(outPath);
if (!fs.existsSync(outDir)) fs.mkdirSync(outDir, { recursive: true });
fs.writeFileSync(outPath, JSON.stringify(latest, null, 2), 'utf8');
console.log('Generated', outPath, 'version', version, 'platforms', Object.keys(platforms));
