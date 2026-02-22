import { useEffect, useState } from 'react';
import { Monitor, Folder, CheckCircle, XCircle, Smartphone, ExternalLink } from 'lucide-react';
import { api, SystemInfo as SystemInfoType, ManagerUpdateInfo, isTauri } from '../../lib/tauri';
import { GITHUB_REPO, GITHUB_RELEASES_URL } from '../../lib/appConfig';

interface SystemInfoProps {
  refreshToken?: string;
}

export function SystemInfo({ refreshToken }: SystemInfoProps) {
  const [info, setInfo] = useState<SystemInfoType | null>(null);
  const [appVersion, setAppVersion] = useState<string | null>(null);
  const [managerUpdate, setManagerUpdate] = useState<ManagerUpdateInfo | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const fetchInfo = async () => {
      setLoading(true);
      if (!isTauri()) {
        setLoading(false);
        return;
      }
      try {
        const [systemInfo, version] = await Promise.all([
          api.getSystemInfo(),
          api.getAppVersion(),
        ]);
        setInfo(systemInfo);
        setAppVersion(version);
        setManagerUpdate(null);
      } catch {
        // 静默处理
      } finally {
        setLoading(false);
      }
    };
    fetchInfo();
  }, [refreshToken]);

  // Manager 自身更新检测（延迟请求，不阻塞首屏）
  useEffect(() => {
    if (!isTauri() || !appVersion) return;
    const controller = new AbortController();
    const timer = setTimeout(async () => {
      try {
        const res = await fetch(
          `https://api.github.com/repos/${GITHUB_REPO}/releases/latest`,
          { signal: controller.signal }
        );
        if (!res.ok) return;
        const data = await res.json();
        const tag = data?.tag_name;
        if (typeof tag !== 'string') return;
        const result = await api.checkManagerUpdateFromLatest(tag);
        setManagerUpdate(result);
      } catch {
        // 网络或解析失败静默忽略
      }
    }, 1500);
    return () => {
      clearTimeout(timer);
      controller.abort();
    };
  }, [appVersion]);

  const getOSLabel = (os: string) => {
    switch (os) {
      case 'macos':
        return 'macOS';
      case 'windows':
        return 'Windows';
      case 'linux':
        return 'Linux';
      default:
        return os;
    }
  };

  if (loading) {
    return (
      <div className="bg-dark-700 rounded-2xl p-6 border border-dark-500">
        <h3 className="text-lg font-semibold text-white mb-4">系统信息</h3>
        <div className="animate-pulse space-y-3">
          <div className="h-4 bg-dark-500 rounded w-1/2"></div>
          <div className="h-4 bg-dark-500 rounded w-2/3"></div>
          <div className="h-4 bg-dark-500 rounded w-1/3"></div>
        </div>
      </div>
    );
  }

  return (
    <div className="bg-dark-700 rounded-2xl p-6 border border-dark-500">
      <h3 className="text-lg font-semibold text-white mb-4">系统信息</h3>

      <div className="space-y-4">
        {/* 操作系统 */}
        <div className="flex items-center gap-3">
          <div className="w-8 h-8 rounded-lg bg-dark-500 flex items-center justify-center">
            <Monitor size={16} className="text-gray-400" />
          </div>
          <div className="flex-1">
            <p className="text-xs text-gray-500">操作系统</p>
            <p className="text-sm text-white">
              {info ? `${getOSLabel(info.os)} ${info.os_version}` : '--'}{' '}
              <span className="text-gray-500">({info?.arch})</span>
            </p>
          </div>
        </div>

        {/* OpenClaw Manager 本应用版本 */}
        {appVersion && (
          <div className="flex items-center gap-3">
            <div className="w-8 h-8 rounded-lg bg-dark-500 flex items-center justify-center">
              <Smartphone size={16} className="text-gray-400" />
            </div>
            <div className="flex-1 min-w-0">
              <p className="text-xs text-gray-500">OpenClaw Manager</p>
              <p className="text-sm text-white flex items-center gap-1.5 flex-wrap">
                <span>v{appVersion}</span>
                {managerUpdate?.update_available && (
                  <a
                    href={GITHUB_RELEASES_URL}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="inline-flex items-center gap-1 text-amber-400 hover:text-amber-300 text-xs"
                  >
                    <ExternalLink size={12} />
                    有新版本 {managerUpdate.latest_version}
                  </a>
                )}
              </p>
            </div>
          </div>
        )}

        {/* OpenClaw */}
        <div className="flex items-center gap-3">
          <div className="w-8 h-8 rounded-lg bg-dark-500 flex items-center justify-center">
            {info?.openclaw_installed ? (
              <CheckCircle size={16} className="text-green-400" />
            ) : (
              <XCircle size={16} className="text-red-400" />
            )}
          </div>
          <div className="flex-1">
            <p className="text-xs text-gray-500">OpenClaw</p>
            <p className="text-sm text-white">
              {info?.openclaw_installed
                ? info.openclaw_version || '已安装'
                : '未安装'}
            </p>
          </div>
        </div>

        {/* 配置目录 */}
        <div className="flex items-center gap-3">
          <div className="w-8 h-8 rounded-lg bg-dark-500 flex items-center justify-center">
            <Folder size={16} className="text-amber-400" />
          </div>
          <div className="flex-1">
            <p className="text-xs text-gray-500">配置目录</p>
            <p className="text-sm text-white font-mono text-xs truncate">
              {info?.config_dir || '--'}
            </p>
          </div>
        </div>
      </div>
    </div>
  );
}
