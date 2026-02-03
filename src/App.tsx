import { useState, useEffect } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { invoke } from '@tauri-apps/api/core';
import { Sidebar } from './components/Layout/Sidebar';
import { Header } from './components/Layout/Header';
import { Dashboard } from './components/Dashboard';
import { AIConfig } from './components/AIConfig';
import { Channels } from './components/Channels';
import { Settings } from './components/Settings';
import { Testing } from './components/Testing';
import { Logs } from './components/Logs';
import { Setup } from './components/Setup';
import { appLogger } from './lib/logger';
import { isTauri } from './lib/tauri';

export type PageType = 'dashboard' | 'ai' | 'channels' | 'testing' | 'logs' | 'settings';

interface EnvironmentStatus {
  node_installed: boolean;
  node_version: string | null;
  node_version_ok: boolean;
  openclaw_installed: boolean;
  openclaw_version: string | null;
  config_dir_exists: boolean;
  ready: boolean;
  os: string;
}

interface ServiceStatus {
  running: boolean;
  pid: number | null;
  port: number;
}

function App() {
  const [currentPage, setCurrentPage] = useState<PageType>('dashboard');
  const [isReady, setIsReady] = useState<boolean | null>(null);
  const [showSetup, setShowSetup] = useState(false);
  const [serviceStatus, setServiceStatus] = useState<ServiceStatus | null>(null);

  // 检查环境
  useEffect(() => {
    appLogger.info('🦞 App 组件已挂载');
    
    // 检查是否在 Tauri 环境中
    if (!isTauri()) {
      appLogger.warn('不在 Tauri 环境中，跳过环境检查');
      setIsReady(true);
      setShowSetup(false);
      return;
    }
    
    const checkEnv = async () => {
      appLogger.info('开始检查系统环境...');
      try {
        const status = await invoke<EnvironmentStatus>('check_environment');
        appLogger.info('环境检查完成', status);
        setIsReady(status.ready);
        setShowSetup(!status.ready);
      } catch (e) {
        appLogger.error('环境检查失败', e);
        setIsReady(true);
        setShowSetup(false);
      }
    };
    checkEnv();
  }, []);

  // 定期获取服务状态
  useEffect(() => {
    // 不在 Tauri 环境中则不轮询
    if (!isTauri()) return;
    
    const fetchServiceStatus = async () => {
      try {
        const status = await invoke<ServiceStatus>('get_service_status');
        setServiceStatus(status);
      } catch {
        // 静默处理轮询错误
      }
    };
    fetchServiceStatus();
    const interval = setInterval(fetchServiceStatus, 3000);
    return () => clearInterval(interval);
  }, []);

  const handleSetupComplete = () => {
    appLogger.info('安装向导完成');
    setIsReady(true);
    setShowSetup(false);
  };

  // 页面切换处理
  const handleNavigate = (page: PageType) => {
    appLogger.action('页面切换', { from: currentPage, to: page });
    setCurrentPage(page);
  };

  const renderPage = () => {
    const pageVariants = {
      initial: { opacity: 0, x: 20 },
      animate: { opacity: 1, x: 0 },
      exit: { opacity: 0, x: -20 },
    };

    const pages: Record<PageType, JSX.Element> = {
      dashboard: <Dashboard />,
      ai: <AIConfig />,
      channels: <Channels />,
      testing: <Testing />,
      logs: <Logs />,
      settings: <Settings />,
    };

    return (
      <AnimatePresence mode="wait">
        <motion.div
          key={currentPage}
          variants={pageVariants}
          initial="initial"
          animate="animate"
          exit="exit"
          transition={{ duration: 0.2 }}
          className="h-full"
        >
          {pages[currentPage]}
        </motion.div>
      </AnimatePresence>
    );
  };

  // 正在检查环境
  if (isReady === null) {
    return (
      <div className="flex h-screen bg-dark-900 items-center justify-center">
        <div className="fixed inset-0 bg-gradient-radial pointer-events-none" />
        <div className="relative z-10 text-center">
          <div className="inline-flex items-center justify-center w-16 h-16 rounded-xl bg-gradient-to-br from-brand-500 to-purple-600 mb-4 animate-pulse">
            <span className="text-3xl">🦞</span>
          </div>
          <p className="text-dark-400">正在启动...</p>
        </div>
      </div>
    );
  }

  // 显示安装向导
  if (showSetup) {
    return <Setup onComplete={handleSetupComplete} />;
  }

  // 正常界面
  return (
    <div className="flex h-screen bg-dark-900 overflow-hidden">
      {/* 背景装饰 */}
      <div className="fixed inset-0 bg-gradient-radial pointer-events-none" />
      
      {/* 侧边栏 */}
      <Sidebar currentPage={currentPage} onNavigate={handleNavigate} serviceStatus={serviceStatus} />
      
      {/* 主内容区 */}
      <div className="flex-1 flex flex-col overflow-hidden">
        {/* 标题栏（macOS 拖拽区域） */}
        <Header currentPage={currentPage} />
        
        {/* 页面内容 */}
        <main className="flex-1 overflow-hidden p-6">
          {renderPage()}
        </main>
      </div>
    </div>
  );
}

export default App;
