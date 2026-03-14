import { useState, useEffect } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { invoke } from '@tauri-apps/api/core';
import { api } from '../../lib/tauri';
import { 
  CheckCircle2,
  Loader2, 
  Download,
  RefreshCw,
  ExternalLink,
  Cpu,
  Package,
  GitBranch,
} from 'lucide-react';
import { setupLogger } from '../../lib/logger';

interface EnvironmentStatus {
  node_installed: boolean;
  node_version: string | null;
  node_version_ok: boolean;
  has_bundled_nodejs: boolean;
  git_installed: boolean;
  git_version: string | null;
  has_offline_package: boolean;
  openclaw_installed: boolean;
  openclaw_version: string | null;
  config_dir_exists: boolean;
  ready: boolean;
  os: string;
}

interface InstallResult {
  success: boolean;
  message: string;
  error: string | null;
}

interface SetupProps {
  onComplete: () => void;
  /** 是否嵌入模式（嵌入到 Dashboard 中显示） */
  embedded?: boolean;
}

export function Setup({ onComplete, embedded = false }: SetupProps) {
  const [envStatus, setEnvStatus] = useState<EnvironmentStatus | null>(null);
  const [checking, setChecking] = useState(true);
  const [installing, setInstalling] = useState<'nodejs' | 'git' | 'openclaw' | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [step, setStep] = useState<'check' | 'install' | 'complete'>('check');
  /** OpenClaw 安装/更新渠道：latest | nightly */
  const [openclawChannel, setOpenclawChannel] = useState<string>('latest');

  const checkEnvironment = async () => {
    setupLogger.info('检查系统环境...');
    setChecking(true);
    setError(null);
    try {
      const status = await invoke<EnvironmentStatus>('check_environment');
      setupLogger.state('环境状态', status);
      setEnvStatus(status);

      if (status.ready && status.openclaw_installed) {
        setupLogger.info('✅ 环境就绪');
        setStep('complete');
        setTimeout(() => onComplete(), 1500);
      } else {
        setupLogger.warn('环境未就绪或 OpenClaw 未安装，需要用户手动触发安装');
        setStep('install');
      }
    } catch (e) {
      setupLogger.error('检查环境失败', e);
      setError(`检查环境失败: ${e}`);
    } finally {
      setChecking(false);
    }
  };

  useEffect(() => {
    setupLogger.info('Setup 组件初始化');
    checkEnvironment();
  }, []);

  useEffect(() => {
    if (typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window) {
      api.getOpenclawChannel().then(setOpenclawChannel).catch(() => {});
    }
  }, []);

  const handleInstallNodejs = async () => {
    setupLogger.action('安装 Node.js');
    setupLogger.info('开始安装 Node.js...');
    setInstalling('nodejs');
    setError(null);
    
    try {
      // 先尝试直接安装
      const result = await invoke<InstallResult>('install_nodejs');
      
      if (result.success) {
        setupLogger.info('✅ Node.js 安装成功');
        // 重新检查环境
        await checkEnvironment();
      } else if (result.message.includes('重启')) {
        // 需要重启应用
        setError('Node.js 安装完成，请重启应用以使环境变量生效');
      } else if (envStatus?.os === 'windows') {
        // Windows 离线包不应回退到在线安装终端
        setError(`Node.js 离线安装失败: ${result.error || result.message}`);
      } else {
        // 打开终端手动安装
        await invoke<string>('open_install_terminal', { installType: 'nodejs' });
        setError('已打开安装终端，请在终端中完成安装后点击"重新检查"');
      }
    } catch (e) {
      // 如果自动安装失败，打开终端
      if (envStatus?.os === 'windows') {
        setError(`Node.js 离线安装失败: ${e}`);
      } else {
        try {
          await invoke<string>('open_install_terminal', { installType: 'nodejs' });
          setError('已打开安装终端，请在终端中完成安装后点击"重新检查"');
        } catch (termErr) {
          setError(`安装失败: ${e}。${termErr}`);
        }
      }
    } finally {
      setInstalling(null);
    }
  };

  const handleInstallOpenclaw = async () => {
    setupLogger.action('安装 OpenClaw');
    setupLogger.info('开始安装 OpenClaw...');
    setInstalling('openclaw');
    setError(null);
    
    try {
      const result = await invoke<InstallResult>('install_openclaw');
      
      if (result.success) {
        setupLogger.info('✅ OpenClaw 安装成功，初始化配置...');
        // 初始化配置
        await invoke<InstallResult>('init_openclaw_config');
        setupLogger.info('✅ 配置初始化完成');
        // 重新检查环境
        await checkEnvironment();
      } else if (envStatus?.os === 'windows') {
        setError(`OpenClaw 离线安装失败: ${result.error || result.message}`);
      } else {
        setupLogger.warn('自动安装失败，打开终端手动安装');
        // 打开终端手动安装
        await invoke<string>('open_install_terminal', { installType: 'openclaw' });
        setError('已打开安装终端，请在终端中完成安装后点击"重新检查"');
      }
    } catch (e) {
      setupLogger.error('安装失败，尝试打开终端', e);
      if (envStatus?.os === 'windows') {
        setError(`OpenClaw 离线安装失败: ${e}`);
      } else {
        try {
          await invoke<string>('open_install_terminal', { installType: 'openclaw' });
          setError('已打开安装终端，请在终端中完成安装后点击"重新检查"');
        } catch (termErr) {
          setError(`安装失败: ${e}。${termErr}`);
        }
      }
    } finally {
      setInstalling(null);
    }
  };

  const handleInstallGit = async () => {
    setupLogger.action('安装 Git');
    setupLogger.info('开始安装 Git...');
    setInstalling('git');
    setError(null);

    try {
      await invoke<string>('open_install_terminal', { installType: 'git' });
      setError('已打开 Git 安装终端，请在终端中完成安装后点击"重新检查"');
    } catch (e) {
      setError(`打开 Git 安装终端失败: ${e}`);
    } finally {
      setInstalling(null);
    }
  };

  const getOsName = (os: string) => {
    switch (os) {
      case 'windows': return 'Windows';
      case 'macos': return 'macOS';
      case 'linux': return 'Linux';
      default: return os;
    }
  };

  // 渲染安装内容（复用于嵌入模式和全屏模式）
  const renderContent = () => {
    return (
      <AnimatePresence mode="wait">
        {/* 检查中状态 */}
        {checking && (
          <motion.div
            key="checking"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="text-center py-6"
          >
            <Loader2 className="w-10 h-10 text-brand-500 animate-spin mx-auto mb-3" />
            <p className="text-dark-300">正在检测系统环境...</p>
          </motion.div>
        )}

        {/* 安装步骤 */}
        {!checking && step === 'install' && envStatus && (
          <motion.div
            key="install"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="space-y-4"
          >
            {/* 系统信息（仅非嵌入模式） */}
            {!embedded && (
              <div className="flex items-center justify-between text-sm text-dark-400 pb-4 border-b border-dark-700">
                <span>操作系统</span>
                <span className="text-dark-200">{getOsName(envStatus.os)}</span>
              </div>
            )}

            {/* Node.js 状态 */}
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-3">
                <div className={`p-2 rounded-lg ${
                  (envStatus.node_installed && envStatus.node_version_ok) || envStatus.has_bundled_nodejs
                    ? 'bg-green-500/20 text-green-400' 
                    : 'bg-red-500/20 text-red-400'
                }`}>
                  <Cpu className="w-5 h-5" />
                </div>
                <div>
                  <p className="text-white font-medium">
                    Node.js {envStatus.has_bundled_nodejs && '✨'}
                  </p>
                  <p className="text-sm text-dark-400">
                    {envStatus.node_version 
                      ? `${envStatus.node_version} ${envStatus.node_version_ok ? '✓' : '(需要 v22.16+)'}` 
                      : envStatus.has_bundled_nodejs
                        ? '✨ 已内置 Node.js（离线可用）'
                        : '未安装'}
                  </p>
                </div>
              </div>
              
              {(envStatus.node_installed && envStatus.node_version_ok) || envStatus.has_bundled_nodejs ? (
                envStatus.has_bundled_nodejs && !envStatus.node_installed ? (
                  <span className="text-xs text-green-400 px-3 py-1 bg-green-500/10 rounded-lg border border-green-500/30">
                    已内置
                  </span>
                ) : (
                  <CheckCircle2 className="w-6 h-6 text-green-400" />
                )
              ) : (
                <button
                  onClick={handleInstallNodejs}
                  disabled={installing !== null}
                  className="btn-primary text-sm px-4 py-2 flex items-center gap-2"
                >
                  {installing === 'nodejs' ? (
                    <>
                      <Loader2 className="w-4 h-4 animate-spin" />
                      安装中...
                    </>
                  ) : (
                    <>
                      <Download className="w-4 h-4" />
                      安装
                    </>
                  )}
                </button>
              )}
            </div>

            {/* Git 状态 (仅 Windows) */}
            {envStatus.os === 'windows' && (
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-3">
                  <div className={`p-2 rounded-lg ${
                    envStatus.git_installed 
                      ? 'bg-green-500/20 text-green-400' 
                      : envStatus.has_offline_package
                        ? 'bg-blue-500/20 text-blue-400'
                        : 'bg-yellow-500/20 text-yellow-400'
                  }`}>
                    <GitBranch className="w-5 h-5" />
                  </div>
                  <div>
                    <p className="text-white font-medium">
                      Git {envStatus.has_offline_package && !envStatus.git_installed && '(可选)'}
                    </p>
                    <p className="text-sm text-dark-400">
                      {envStatus.git_version 
                        ? envStatus.git_version 
                        : envStatus.has_offline_package
                          ? '未安装 (已有离线包，无需 Git)'
                          : '未安装 (在线安装需要)'}
                    </p>
                  </div>
                </div>
                
                {envStatus.git_installed ? (
                  <CheckCircle2 className="w-6 h-6 text-green-400" />
                ) : (
                  <button
                    onClick={handleInstallGit}
                    disabled={installing !== null}
                    className="btn-primary text-sm px-4 py-2 flex items-center gap-2"
                    title={
                      envStatus.has_offline_package
                        ? '已有离线包，Git 可选安装'
                        : '在线安装需要 Git；建议优先放置 git-portable.zip'
                    }
                  >
                    {installing === 'git' ? (
                      <>
                        <Loader2 className="w-4 h-4 animate-spin" />
                        安装中...
                      </>
                    ) : (
                      <>
                        <Download className="w-4 h-4" />
                        {envStatus.has_offline_package ? '安装 (可选)' : '安装'}
                      </>
                    )}
                  </button>
                )}
              </div>
            )}

            {/* OpenClaw 渠道（仅 Tauri 环境） */}
            {typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window && (
              <div className="flex items-center justify-between text-sm">
                <span className="text-dark-400">安装/更新渠道</span>
                <div className="flex gap-2">
                  <button
                    type="button"
                    onClick={async () => {
                      try {
                        await api.setOpenclawChannel('latest');
                        setOpenclawChannel('latest');
                      } catch (e) {
                        setupLogger.error('切换渠道失败', e);
                      }
                    }}
                    className={`px-3 py-1.5 rounded-lg text-sm font-medium transition-colors ${
                      openclawChannel === 'latest'
                        ? 'bg-brand-500/30 text-brand-300 border border-brand-500/50'
                        : 'bg-dark-700 text-dark-300 hover:bg-dark-600 border border-dark-600'
                    }`}
                  >
                    Latest
                  </button>
                  <button
                    type="button"
                    onClick={async () => {
                      try {
                        await api.setOpenclawChannel('nightly');
                        setOpenclawChannel('nightly');
                      } catch (e) {
                        setupLogger.error('切换渠道失败', e);
                      }
                    }}
                    className={`px-3 py-1.5 rounded-lg text-sm font-medium transition-colors ${
                      openclawChannel === 'nightly'
                        ? 'bg-brand-500/30 text-brand-300 border border-brand-500/50'
                        : 'bg-dark-700 text-dark-300 hover:bg-dark-600 border border-dark-600'
                    }`}
                  >
                    Nightly
                  </button>
                </div>
              </div>
            )}

            {/* OpenClaw 状态 */}
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-3">
                <div className={`p-2 rounded-lg ${
                  envStatus.openclaw_installed 
                    ? 'bg-green-500/20 text-green-400' 
                    : 'bg-red-500/20 text-red-400'
                }`}>
                  <Package className="w-5 h-5" />
                </div>
                <div>
                  <p className="text-white font-medium">OpenClaw</p>
                  <p className="text-sm text-dark-400">
                    {envStatus.openclaw_installed
                      ? (envStatus.openclaw_version || '已安装')
                      : '未安装'}
                  </p>
                </div>
              </div>
              
              {envStatus.openclaw_installed ? (
                <CheckCircle2 className="w-6 h-6 text-green-400" />
              ) : (
                <button
                  onClick={handleInstallOpenclaw}
                  disabled={
                    installing !== null || 
                    (!envStatus.node_version_ok && !envStatus.has_bundled_nodejs) ||
                    (envStatus.os === 'windows' && !envStatus.has_offline_package && !envStatus.git_installed)
                  }
                  className={`btn-primary text-sm px-4 py-2 flex items-center gap-2 ${
                    (!envStatus.node_version_ok && !envStatus.has_bundled_nodejs) ||
                    (envStatus.os === 'windows' && !envStatus.has_offline_package && !envStatus.git_installed)
                      ? 'opacity-50 cursor-not-allowed' 
                      : ''
                  }`}
                  title={
                    (!envStatus.node_version_ok && !envStatus.has_bundled_nodejs)
                      ? '请先安装 Node.js（或下载内置版本）' 
                      : (envStatus.os === 'windows' && !envStatus.has_offline_package && !envStatus.git_installed)
                        ? '请先安装 Git（或使用包含离线包的版本）'
                        : (envStatus.has_bundled_nodejs && envStatus.has_offline_package)
                          ? '✨ 完全离线安装，无需任何依赖'
                          : envStatus.has_offline_package
                            ? '使用离线包安装，无需 Git'
                            : ''
                  }
                >
                  {installing === 'openclaw' ? (
                    <>
                      <Loader2 className="w-4 h-4 animate-spin" />
                      安装中...
                    </>
                  ) : (
                    <>
                      <Download className="w-4 h-4" />
                      {envStatus.has_bundled_nodejs && envStatus.has_offline_package 
                        ? '安装 (完全离线)' 
                        : envStatus.has_offline_package 
                          ? '安装 (离线)' 
                          : '安装'}
                    </>
                  )}
                </button>
              )}
            </div>

            {/* 错误信息 */}
            {error && (
              <motion.div
                initial={{ opacity: 0, y: -10 }}
                animate={{ opacity: 1, y: 0 }}
                className="p-3 bg-yellow-500/10 border border-yellow-500/30 rounded-lg"
              >
                <p className="text-yellow-400 text-sm">{error}</p>
              </motion.div>
            )}

            {/* 操作按钮 */}
            <div className="flex gap-3 pt-4 border-t border-dark-700/50">
              <button
                onClick={checkEnvironment}
                disabled={checking || installing !== null}
                className="flex-1 btn-secondary py-2.5 flex items-center justify-center gap-2"
              >
                <RefreshCw className={`w-4 h-4 ${checking ? 'animate-spin' : ''}`} />
                重新检查
              </button>
            </div>

            {/* 帮助链接 */}
            <div className="text-center pt-1">
              <a
                href="https://nodejs.org/en/download"
                target="_blank"
                rel="noopener noreferrer"
                className="text-sm text-dark-400 hover:text-brand-400 transition-colors inline-flex items-center gap-1"
              >
                手动下载 Node.js
                <ExternalLink className="w-3 h-3" />
              </a>
            </div>
          </motion.div>
        )}

        {/* 完成状态 */}
        {!checking && step === 'complete' && (
          <motion.div
            key="complete"
            initial={{ opacity: 0, scale: 0.9 }}
            animate={{ opacity: 1, scale: 1 }}
            className="text-center py-6"
          >
            <motion.div
              initial={{ scale: 0 }}
              animate={{ scale: 1 }}
              transition={{ type: 'spring', damping: 10, delay: 0.1 }}
            >
              <CheckCircle2 className="w-12 h-12 text-green-400 mx-auto mb-3" />
            </motion.div>
            <h3 className="text-lg font-bold text-white mb-1">环境就绪！</h3>
            <p className="text-dark-400 text-sm">
              Node.js 和 OpenClaw 已正确安装
            </p>
          </motion.div>
        )}
      </AnimatePresence>
    );
  };

  // 嵌入模式：作为卡片显示在 Dashboard 中
  if (embedded) {
    return (
      <div className="bg-gradient-to-br from-yellow-500/10 to-orange-500/10 border border-yellow-500/30 rounded-2xl p-6">
        <div className="flex items-start gap-4 mb-4">
          <div className="flex-shrink-0 w-12 h-12 rounded-xl bg-gradient-to-br from-yellow-500 to-orange-500 flex items-center justify-center">
            <span className="text-2xl">⚠️</span>
          </div>
          <div>
            <h2 className="text-lg font-bold text-white mb-1">环境配置</h2>
            <p className="text-dark-400 text-sm">检测到缺少必要的依赖，请完成以下安装</p>
          </div>
        </div>
        
        {renderContent()}
      </div>
    );
  }

  // 全屏模式（保留用于特殊情况）
  return (
    <div className="min-h-screen bg-dark-900 flex items-center justify-center p-8">
      {/* 背景装饰 */}
      <div className="fixed inset-0 bg-gradient-radial pointer-events-none" />
      <div className="absolute inset-0 overflow-hidden pointer-events-none">
        <div className="absolute -top-40 -right-40 w-80 h-80 bg-brand-500/10 rounded-full blur-3xl" />
        <div className="absolute -bottom-40 -left-40 w-80 h-80 bg-purple-500/10 rounded-full blur-3xl" />
      </div>
      
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        className="relative z-10 w-full max-w-lg"
      >
        {/* Logo 和标题 */}
        <div className="text-center mb-8">
          <motion.div
            initial={{ scale: 0.8 }}
            animate={{ scale: 1 }}
            transition={{ type: 'spring', damping: 15 }}
            className="inline-flex items-center justify-center w-20 h-20 rounded-2xl bg-gradient-to-br from-brand-500 to-purple-600 mb-4 shadow-lg shadow-brand-500/25"
          >
            <span className="text-4xl">🦞</span>
          </motion.div>
          <h1 className="text-2xl font-bold text-white mb-2">OpenClaw Manager</h1>
          <p className="text-dark-400">环境检测与安装向导</p>
        </div>

        {/* 主卡片 */}
        <motion.div
          layout
          className="glass-card rounded-2xl p-6 shadow-xl"
        >
          {renderContent()}
        </motion.div>

        {/* 版本信息 */}
        <p className="text-center text-dark-500 text-xs mt-6">
          OpenClaw Manager v0.0.8
        </p>
      </motion.div>
    </div>
  );
}
