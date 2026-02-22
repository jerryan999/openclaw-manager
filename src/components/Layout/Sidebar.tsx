import { motion } from 'framer-motion';
import {
  LayoutDashboard,
  BookOpen,
  Bot,
  MessageSquare,
  FlaskConical,
  ScrollText,
  Settings,
  MessageCircle,
} from 'lucide-react';
import { PageType } from '../../App';
import clsx from 'clsx';
import { open } from '@tauri-apps/plugin-shell';
import { WECHAT_QR_URL } from '../../lib/appConfig';

interface SidebarProps {
  currentPage: PageType;
  onNavigate: (page: PageType) => void;
}

const menuItems: { id: PageType; label: string; icon: React.ElementType }[] = [
  { id: 'dashboard', label: '概览', icon: LayoutDashboard },
  { id: 'ai', label: 'AI 配置', icon: Bot },
  { id: 'channels', label: '消息渠道', icon: MessageSquare },
  { id: 'testing', label: '测试诊断', icon: FlaskConical },
  { id: 'logs', label: '应用日志', icon: ScrollText },
  { id: 'settings', label: '设置', icon: Settings },
];

export function Sidebar({ currentPage, onNavigate }: SidebarProps) {
  return (
    <aside className="w-64 bg-dark-800 border-r border-dark-600 flex flex-col">
      {/* Logo 区域（macOS 标题栏拖拽） */}
      <div className="h-14 flex items-center px-6 titlebar-drag border-b border-dark-600">
        <div className="flex items-center gap-3 titlebar-no-drag">
          <div className="w-8 h-8 rounded-lg bg-gradient-to-br from-claw-400 to-claw-600 flex items-center justify-center">
            <span className="text-lg">🦞</span>
          </div>
          <div>
            <h1 className="text-sm font-semibold text-white">OpenClaw</h1>
            <p className="text-xs text-gray-500">Manager</p>
          </div>
        </div>
      </div>

      {/* 导航菜单 */}
      <nav className="flex-1 py-4 px-3">
        <ul className="space-y-1">
          {menuItems.map((item) => {
            const isActive = currentPage === item.id;
            const Icon = item.icon;
            
            return (
              <li key={item.id}>
                <button
                  onClick={() => onNavigate(item.id)}
                  className={clsx(
                    'w-full flex items-center gap-3 px-4 py-2.5 rounded-lg text-sm font-medium transition-all relative',
                    isActive
                      ? 'text-white bg-dark-600'
                      : 'text-gray-400 hover:text-white hover:bg-dark-700'
                  )}
                >
                  {isActive && (
                    <motion.div
                      layoutId="activeIndicator"
                      className="absolute left-0 top-1/2 -translate-y-1/2 w-1 h-6 bg-claw-500 rounded-r-full"
                      transition={{ type: 'spring', stiffness: 300, damping: 30 }}
                    />
                  )}
                  <Icon size={18} className={isActive ? 'text-claw-400' : ''} />
                  <span>{item.label}</span>
                </button>
              </li>
            );
          })}
        </ul>
      </nav>

      {/* 底部：交流群入口 + 学习天地 */}
      <div className="p-4 border-t border-dark-600 space-y-2">
        <button
          type="button"
          onClick={() => open(WECHAT_QR_URL).catch(() => {})}
          className="w-full px-4 py-3 flex items-center justify-center gap-2 rounded-xl border border-green-500/25 bg-green-500/10 hover:bg-green-500/20 transition-colors text-green-400/90 hover:text-green-300"
        >
          <MessageCircle size={18} />
          <span className="text-sm font-medium">扫码加入交流群</span>
        </button>
        <button
          type="button"
          onClick={() => onNavigate('learning')}
          className={clsx(
            'w-full flex items-center justify-center gap-2 px-4 py-3 rounded-xl text-sm font-medium transition-all relative',
            currentPage === 'learning'
              ? 'text-white bg-claw-500/20 border border-claw-500/40'
              : 'border border-dark-500 bg-dark-600 hover:bg-dark-500 text-gray-300 hover:text-white'
          )}
        >
          {currentPage === 'learning' && (
            <motion.div
              layoutId="activeIndicatorBottom"
              className="absolute left-0 top-1/2 -translate-y-1/2 w-1 h-6 bg-claw-500 rounded-r-full"
              transition={{ type: 'spring', stiffness: 300, damping: 30 }}
            />
          )}
          <BookOpen size={18} />
          <span>学习天地</span>
        </button>
      </div>
    </aside>
  );
}
