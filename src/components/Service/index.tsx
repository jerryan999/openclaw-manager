import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Play, Square, RotateCcw, Loader2 } from 'lucide-react';
import clsx from 'clsx';
import { serviceLogger } from '../../lib/logger';

export function ServiceManager() {
  const [actionLoading, setActionLoading] = useState<string | null>(null);

  serviceLogger.debug('ServiceManager 组件渲染');

  const handleAction = async (action: 'start' | 'stop' | 'restart') => {
    serviceLogger.action(`服务操作: ${action}`);
    serviceLogger.info(`正在执行: ${action}_service`);
    setActionLoading(action);
    try {
      const result = await invoke(`${action}_service`);
      serviceLogger.info(`✅ ${action} 操作成功`, result);
    } catch (e) {
      serviceLogger.error(`❌ ${action} 操作失败`, e);
      alert(`操作失败: ${e}`);
    } finally {
      setActionLoading(null);
    }
  };

  return (
    <div className="h-full flex flex-col overflow-hidden">
      {/* 操作按钮栏 */}
      <div className="flex items-center gap-4">
        <div className="flex items-center gap-2">
          <button
            onClick={() => handleAction('start')}
            disabled={actionLoading !== null}
            className={clsx(
              'flex items-center gap-2 px-4 py-2 rounded-lg font-medium transition-all',
              'bg-green-500/20 text-green-400 border border-green-500/30',
              'hover:bg-green-500/30 disabled:opacity-50'
            )}
          >
            {actionLoading === 'start' ? (
              <Loader2 size={16} className="animate-spin" />
            ) : (
              <Play size={16} />
            )}
            启动
          </button>

          <button
            onClick={() => handleAction('stop')}
            disabled={actionLoading !== null}
            className={clsx(
              'flex items-center gap-2 px-4 py-2 rounded-lg font-medium transition-all',
              'bg-red-500/20 text-red-400 border border-red-500/30',
              'hover:bg-red-500/30 disabled:opacity-50'
            )}
          >
            {actionLoading === 'stop' ? (
              <Loader2 size={16} className="animate-spin" />
            ) : (
              <Square size={16} />
            )}
            停止
          </button>

          <button
            onClick={() => handleAction('restart')}
            disabled={actionLoading !== null}
            className={clsx(
              'flex items-center gap-2 px-4 py-2 rounded-lg font-medium transition-all',
              'bg-amber-500/20 text-amber-400 border border-amber-500/30',
              'hover:bg-amber-500/30 disabled:opacity-50'
            )}
          >
            {actionLoading === 'restart' ? (
              <Loader2 size={16} className="animate-spin" />
            ) : (
              <RotateCcw size={16} />
            )}
            重启
          </button>
        </div>
      </div>
    </div>
  );
}
