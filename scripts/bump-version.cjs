#!/usr/bin/env node
/**
 * 一键更新项目版本号（package.json、Cargo.toml、tauri.conf.json、Makefile）
 * 用法: node scripts/bump-version.cjs <版本号>
 * 或:   npm run version 0.0.26
 */
const fs = require('fs');
const path = require('path');

const root = path.resolve(__dirname, '..');
const version = process.argv[2];

if (!version || !/^\d+\.\d+\.\d+$/.test(version)) {
  console.error('用法: npm run version <x.y.z>');
  console.error('示例: npm run version 0.0.26');
  process.exit(1);
}

const files = [
  {
    path: path.join(root, 'package.json'),
    replace: (content) => content.replace(/"version":\s*"[^"]+"/, `"version": "${version}"`),
  },
  {
    path: path.join(root, 'src-tauri', 'Cargo.toml'),
    replace: (content) => content.replace(/^version\s*=\s*"[^"]+"/m, `version = "${version}"`),
  },
  {
    path: path.join(root, 'src-tauri', 'tauri.conf.json'),
    replace: (content) => content.replace(/"version":\s*"[^"]+"/, `"version": "${version}"`),
  },
  {
    path: path.join(root, 'Makefile'),
    replace: (content) => content.replace(/@echo "Version: [^"]+"/, `@echo "Version: ${version}"`),
  },
];

let ok = true;
for (const { path: filePath, replace } of files) {
  try {
    let content = fs.readFileSync(filePath, 'utf8');
    const next = replace(content);
    if (next === content) {
      console.warn('⚠ 未匹配到版本号，跳过:', path.relative(root, filePath));
      continue;
    }
    fs.writeFileSync(filePath, next);
    console.log('✓', path.relative(root, filePath));
  } catch (e) {
    console.error('✗', filePath, e.message);
    ok = false;
  }
}

if (!ok) process.exit(1);
console.log('\n版本已统一为', version, '，可提交并打 tag: v' + version);
