/**
 * 前端日志工具
 * 统一管理所有前端日志输出，方便调试和追踪
 */

type LogLevel = 'debug' | 'info' | 'warn' | 'error';

// 日志条目
export interface LogEntry {
  id: number;
  timestamp: Date;
  level: LogLevel;
  module: string;
  message: string;
  args: unknown[];
}

// 日志级别权重
const LOG_LEVELS: Record<LogLevel, number> = {
  debug: 0,
  info: 1,
  warn: 2,
  error: 3,
};

// 日志存储
class LogStore {
  private logs: LogEntry[] = [];
  private maxLogs = 500;
  private idCounter = 0;
  private listeners: Set<() => void> = new Set();

  add(entry: Omit<LogEntry, 'id'>) {
    const newEntry: LogEntry = {
      ...entry,
      id: ++this.idCounter,
    };
    this.logs.push(newEntry);
    
    // 限制日志数量
    if (this.logs.length > this.maxLogs) {
      this.logs = this.logs.slice(-this.maxLogs);
    }
    
    // 通知监听者
    this.listeners.forEach(listener => listener());
  }

  getAll(): LogEntry[] {
    return [...this.logs];
  }

  clear() {
    this.logs = [];
    this.listeners.forEach(listener => listener());
  }

  subscribe(listener: () => void): () => void {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  }
}

// 全局日志存储实例
export const logStore = new LogStore();

// 当前日志级别（可通过 localStorage 设置）
const getCurrentLevel = (): LogLevel => {
  if (typeof window !== 'undefined') {
    const level = localStorage.getItem('LOG_LEVEL') as LogLevel;
    if (level && LOG_LEVELS[level] !== undefined) {
      return level;
    }
  }
  // 默认 debug 级别（开发时显示所有日志）
  return 'debug';
};

// 日志样式
const STYLES: Record<LogLevel, string> = {
  debug: 'color: #888; font-weight: normal',
  info: 'color: #4ade80; font-weight: normal',
  warn: 'color: #facc15; font-weight: bold',
  error: 'color: #f87171; font-weight: bold',
};

// 模块颜色（为不同模块分配不同颜色）
const MODULE_COLORS: Record<string, string> = {
  App: '#a78bfa',
  Service: '#60a5fa',
  Config: '#34d399',
  AI: '#f472b6',
  Channel: '#fb923c',
  Setup: '#22d3ee',
  Dashboard: '#a3e635',
  Testing: '#e879f9',
  API: '#fbbf24',
};

const getModuleColor = (module: string): string => {
  return MODULE_COLORS[module] || '#94a3b8';
};

class Logger {
  private module: string;

  constructor(module: string) {
    this.module = module;
  }

  private shouldLog(level: LogLevel): boolean {
    return LOG_LEVELS[level] >= LOG_LEVELS[getCurrentLevel()];
  }

  private formatMessage(level: LogLevel, message: string, ...args: unknown[]): void {
    if (!this.shouldLog(level)) return;

    const now = new Date();
    const timestamp = now.toLocaleTimeString('zh-CN', { 
      hour12: false,
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit',
    }) + '.' + String(now.getMilliseconds()).padStart(3, '0');
    
    const moduleColor = getModuleColor(this.module);
    const prefix = `%c${timestamp} %c[${this.module}]%c`;
    
    const consoleMethod = level === 'error' ? 'error' : level === 'warn' ? 'warn' : 'log';
    
    console[consoleMethod](
      prefix + ` %c${message}`,
      'color: #666',
      `color: ${moduleColor}; font-weight: bold`,
      '',
      STYLES[level],
      ...args
    );

    // 存储日志
    logStore.add({
      timestamp: now,
      level,
      module: this.module,
      message,
      args,
    });
  }

  debug(message: string, ...args: unknown[]): void {
    this.formatMessage('debug', message, ...args);
  }

  info(message: string, ...args: unknown[]): void {
    this.formatMessage('info', message, ...args);
  }

  warn(message: string, ...args: unknown[]): void {
    this.formatMessage('warn', message, ...args);
  }

  error(message: string, ...args: unknown[]): void {
    this.formatMessage('error', message, ...args);
  }

  // 记录 API 调用
  apiCall(method: string, ...args: unknown[]): void {
    this.debug(`📡 调用 API: ${method}`, ...args);
  }

  // 记录 API 响应
  apiResponse(method: string, result: unknown): void {
    this.debug(`✅ API 响应: ${method}`, result);
  }

  // 记录 API 错误
  apiError(method: string, error: unknown): void {
    this.error(`❌ API 错误: ${method}`, error);
  }

  // 记录用户操作
  action(action: string, ...args: unknown[]): void {
    this.info(`👆 用户操作: ${action}`, ...args);
  }

  // 记录状态变化
  state(description: string, state: unknown): void {
    this.debug(`📊 状态变化: ${description}`, state);
  }
}

// 创建模块 logger 的工厂函数
export function createLogger(module: string): Logger {
  return new Logger(module);
}

// 全局设置日志级别
export function setLogLevel(level: LogLevel): void {
  localStorage.setItem('LOG_LEVEL', level);
  console.log(`%c日志级别已设置为: ${level}`, 'color: #4ade80; font-weight: bold');
}

// 导出预创建的常用 logger
export const appLogger = createLogger('App');
export const serviceLogger = createLogger('Service');
export const configLogger = createLogger('Config');
export const aiLogger = createLogger('AI');
export const channelLogger = createLogger('Channel');
export const setupLogger = createLogger('Setup');
export const dashboardLogger = createLogger('Dashboard');
export const testingLogger = createLogger('Testing');
export const apiLogger = createLogger('API');

// 在控制台暴露日志控制函数
if (typeof window !== 'undefined') {
  (window as unknown as Record<string, unknown>).setLogLevel = setLogLevel;
  (window as unknown as Record<string, unknown>).logStore = logStore;
  console.log(
    '%c🦞 OpenClaw Manager 日志已启用\n' +
    '%c使用 setLogLevel("debug"|"info"|"warn"|"error") 设置日志级别',
    'color: #a78bfa; font-weight: bold; font-size: 14px',
    'color: #888; font-size: 12px'
  );
}
