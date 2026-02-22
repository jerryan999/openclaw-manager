/**
 * 应用对外链接配置（仓库地址、微信群二维码等）
 * 使用当前仓库的 GitHub 地址
 */
export const GITHUB_REPO = 'jerryan999/openclaw-manager';
const GITHUB_BRANCH = 'main';

export const GITHUB_RELEASES_URL = `https://github.com/${GITHUB_REPO}/releases`;

export const WECHAT_QR_URL = `https://raw.githubusercontent.com/${GITHUB_REPO}/${GITHUB_BRANCH}/public/wechat-group.jpg`;

/** 学习天地案例列表（JSON）。开发环境用本地 public 文件，生产环境从 GitHub 拉取便于无需发版更新 */
export const LEARNING_CASES_URL = import.meta.env.DEV
  ? '/learning-cases.json'
  : `https://raw.githubusercontent.com/${GITHUB_REPO}/${GITHUB_BRANCH}/public/learning-cases.json`;
