/**
 * 应用对外链接配置（仓库地址、微信群二维码等）
 * 使用当前仓库的 GitHub 地址
 */
export const GITHUB_REPO = 'jerryan999/openclaw-manager';
const GITHUB_BRANCH = 'main';

export const GITHUB_RELEASES_URL = `https://github.com/${GITHUB_REPO}/releases`;

export const WECHAT_QR_URL = `https://raw.githubusercontent.com/${GITHUB_REPO}/${GITHUB_BRANCH}/public/wechat-group.jpg`;

/** 学习天地案例列表（JSON），可从 GitHub 更新无需发版 */
export const LEARNING_CASES_URL = `https://raw.githubusercontent.com/${GITHUB_REPO}/${GITHUB_BRANCH}/public/learning-cases.json`;
