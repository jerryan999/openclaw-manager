import { useState, useEffect, useMemo } from 'react';
import DOMPurify from 'dompurify';
import { motion } from 'framer-motion';
import { Copy, Check, Download, Coins, MessageSquare, Loader2, Search } from 'lucide-react';
import clsx from 'clsx';
import { LEARNING_CASES_URL } from '../../lib/appConfig';

/** description 在页面上以 HTML 渲染（支持 <a>、<br/>、<strong> 等），经 DOMPurify 白名单后输出，避免 XSS */
const DESCRIPTION_ALLOWED = {
  ALLOWED_TAGS: ['a', 'br', 'strong', 'em', 'b', 'i'],
  ALLOWED_ATTR: ['href'],
};

function sanitizeDescription(html: string): string {
  const withBr = html.replace(/\n/g, '<br/>');
  const out = DOMPurify.sanitize(withBr, {
    ...DESCRIPTION_ALLOWED,
    ADD_ATTR: ['target', 'rel'],
  });
  // 为链接统一加上 target="_blank" 与 rel（DOMPurify 可能不保留，用后处理保证）
  return out.replace(/<a\s+/g, '<a target="_blank" rel="noopener noreferrer" ');
}

export interface LearningCase {
  id: string;
  title: string;
  description: string;
  prompt: string;
  icon: React.ElementType;
  tags?: string[];
}

/** 从 GitHub 拉取的案例项（icon 为字符串） */
interface LearningCaseRaw {
  id: string;
  title: string;
  description: string;
  prompt: string;
  icon: string;
  tags?: string[];
}

/** 学习页可选标签：新手、模型、技能、成本 */
const LEARNING_TAGS = ['新手', '模型', '技能', '成本'] as const;

const ICON_MAP: Record<string, React.ElementType> = {
  messageSquare: MessageSquare,
  download: Download,
  coins: Coins,
};

/** 内置默认案例（网络失败或解析失败时使用） */
const DEFAULT_LEARNING_CASES: LearningCase[] = [
  {
    id: 'new-session-tips',
    title: '新会话快速上手',
    description: '复制到 OpenClaw 新会话中发送，获取当前环境下的入门指引与可做事项。',
    prompt: `我是 OpenClaw 用户，刚打开一个新会话。请基于「当前是 OpenClaw 对话环境」简要说明：
1. 在这个会话里你能帮我做哪些事（如执行技能、查状态、配置建议等）；
2. 我想安装技能或修改提示词时，该怎么说、怎么操作；
3. 给一句简短的最佳实践。`,
    icon: MessageSquare,
    tags: ['新手'],
  },
  {
    id: 'download-skill-prompt',
    title: '添加技能',
    description: '安装、检查已安装技能，并设置或优化系统提示词，适合初次配置或扩展能力。',
    prompt: `请帮我做这些 OpenClaw 相关设置：
1. 检查并列出当前已安装的技能与插件；
2. 若有官方推荐技能（如 @openclaw/download 等），请指导或协助我安装；
3. 帮我设置或优化系统提示词（system prompt），让回复更符合我的使用场景。`,
    icon: Download,
    tags: ['技能'],
  },
  {
    id: 'save-token',
    title: '节约成本 / 省 Token',
    description: '在保证效果的前提下，减少 API 调用与上下文长度，涉及模型选择与用法建议。',
    prompt: `请以「节约 Token、控制成本」为目标，给出一套 OpenClaw 使用建议：
1. 系统提示词如何写得简短但有效；
2. 对话时如何避免重复发送过长上下文；
3. 模型选择与 max_tokens 设置建议；
4. 是否有摘要、分步等技巧可以少用 Token。`,
    icon: Coins,
    tags: ['成本', '模型'],
  },
];

function parseCasesFromRemote(raw: { cases?: LearningCaseRaw[] }): LearningCase[] {
  if (!raw?.cases || !Array.isArray(raw.cases)) return [];
  return raw.cases
    .filter((c) => c?.id && c?.title && c?.prompt)
    .map((c) => ({
      id: c.id,
      title: c.title,
      description: c.description ?? '',
      prompt: c.prompt,
      icon: ICON_MAP[c.icon?.toLowerCase()] ?? MessageSquare,
      tags: Array.isArray(c.tags) ? c.tags : undefined,
    }));
}

const ALL_TAGS_ID = '__all__';

function LearningCard({
  item,
  onCopy,
  copiedId,
}: {
  item: LearningCase;
  onCopy: (id: string, text: string) => void;
  copiedId: string | null;
}) {
  const [showPrompt, setShowPrompt] = useState(false);
  const Icon = item.icon;
  const isCopied = copiedId === item.id;

  return (
    <motion.div
      layout
      initial={{ opacity: 0, y: 16 }}
      animate={{ opacity: 1, y: 0 }}
      className="bg-dark-700 rounded-2xl border border-dark-500 overflow-hidden"
    >
      <div className="p-5">
        <div className="flex items-start gap-4">
          <div className="w-10 h-10 rounded-xl bg-claw-500/20 flex items-center justify-center shrink-0">
            <Icon className="text-claw-400" size={20} />
          </div>
          <div className="flex-1 min-w-0">
            <h3 className="text-base font-semibold text-white mb-1">{item.title}</h3>
            <div
              className="text-sm text-gray-400 mb-2 whitespace-pre-line [&_a]:text-claw-400 [&_a:hover]:text-claw-300 [&_a]:underline [&_a]:break-all"
              dangerouslySetInnerHTML={{ __html: sanitizeDescription(item.description) }}
            />
            {item.tags && item.tags.length > 0 && (
              <div className="flex flex-wrap gap-1.5 mb-3">
                {item.tags.map((tag) => (
                  <span
                    key={tag}
                    className="px-2 py-0.5 rounded-md bg-dark-600 text-gray-400 text-xs"
                  >
                    {tag}
                  </span>
                ))}
              </div>
            )}
            <button
              type="button"
              onClick={() => setShowPrompt(!showPrompt)}
              className="text-xs text-claw-400 hover:text-claw-300 transition-colors"
            >
              {showPrompt ? '收起提示词' : '展开查看提示词'}
            </button>
            {showPrompt && (
              <pre className="mt-2 p-3 rounded-lg bg-dark-800 border border-dark-600 text-xs text-gray-300 whitespace-pre-wrap font-sans">
                {item.prompt}
              </pre>
            )}
          </div>
        </div>
        <div className="flex flex-wrap gap-2 mt-4 pt-4 border-t border-dark-600">
          <button
            type="button"
            onClick={() => onCopy(item.id, item.prompt)}
            className={clsx(
              'flex items-center gap-2 px-3 py-2 rounded-lg text-sm font-medium transition-colors',
              isCopied
                ? 'bg-green-500/20 text-green-400 border border-green-500/40'
                : 'bg-dark-600 hover:bg-dark-500 text-gray-300 hover:text-white border border-dark-500'
            )}
          >
            {isCopied ? <Check size={14} /> : <Copy size={14} />}
            {isCopied ? '已复制' : '复制提示词'}
          </button>
        </div>
      </div>
    </motion.div>
  );
}

export function Learning() {
  const [copiedId, setCopiedId] = useState<string | null>(null);
  const [selectedTag, setSelectedTag] = useState<string>(ALL_TAGS_ID);
  const [cases, setCases] = useState<LearningCase[]>(DEFAULT_LEARNING_CASES);
  const [loading, setLoading] = useState(true);
  const [searchKeyword, setSearchKeyword] = useState('');

  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    fetch(LEARNING_CASES_URL)
      .then((res) => (res.ok ? res.json() : Promise.reject(new Error('fetch failed'))))
      .then((data) => {
        if (cancelled) return;
        const parsed = parseCasesFromRemote(data);
        if (parsed.length > 0) setCases(parsed);
      })
      .catch(() => {
        if (!cancelled) setCases(DEFAULT_LEARNING_CASES);
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, []);

  const filteredCases = useMemo(() => {
    let list =
      selectedTag === ALL_TAGS_ID
        ? cases
        : cases.filter((c) => c.tags?.includes(selectedTag));
    const kw = searchKeyword.trim().toLowerCase();
    if (kw) {
      list = list.filter(
        (c) =>
          c.title.toLowerCase().includes(kw) ||
          (c.description && c.description.toLowerCase().includes(kw)) ||
          c.prompt.toLowerCase().includes(kw)
      );
    }
    return list;
  }, [cases, selectedTag, searchKeyword]);

  const handleCopy = async (id: string, text: string) => {
    try {
      await navigator.clipboard.writeText(text);
      setCopiedId(id);
      setTimeout(() => setCopiedId(null), 2000);
    } catch {
      // ignore
    }
  };

  return (
    <div className="h-full overflow-y-auto scroll-container pr-2">
      <motion.div
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        className="space-y-6"
      >
        {/* 顶部搜索 */}
        <div className="flex items-center gap-2">
          <Search size={18} className="text-gray-500 shrink-0" />
          <input
            type="text"
            value={searchKeyword}
            onChange={(e) => setSearchKeyword(e.target.value)}
            placeholder="搜索案例（标题、描述、提示词）"
            className="flex-1 min-w-0 px-3 py-2 rounded-lg bg-dark-700 border border-dark-500 text-sm text-white placeholder-gray-500 focus:outline-none focus:border-claw-500/50"
          />
        </div>

        {/* 标签筛选：单选 */}
        <div className="flex flex-wrap gap-2">
          <button
            type="button"
            onClick={() => setSelectedTag(ALL_TAGS_ID)}
            className={clsx(
              'px-3 py-1.5 rounded-lg text-sm font-medium transition-colors',
              selectedTag === ALL_TAGS_ID
                ? 'bg-claw-500/30 text-claw-300 border border-claw-500/50'
                : 'bg-dark-600 text-gray-400 hover:text-white border border-dark-500 hover:border-dark-400'
            )}
          >
            全部
          </button>
          {LEARNING_TAGS.map((tag) => (
            <button
              key={tag}
              type="button"
              onClick={() => setSelectedTag(tag)}
              className={clsx(
                'px-3 py-1.5 rounded-lg text-sm font-medium transition-colors',
                selectedTag === tag
                  ? 'bg-claw-500/30 text-claw-300 border border-claw-500/50'
                  : 'bg-dark-600 text-gray-400 hover:text-white border border-dark-500 hover:border-dark-400'
              )}
            >
              {tag}
            </button>
          ))}
        </div>

        <div className="grid gap-4 sm:grid-cols-1">
          {loading ? (
            <div className="flex items-center justify-center py-12 text-gray-500">
              <Loader2 size={24} className="animate-spin" />
            </div>
          ) : (
            filteredCases.map((item) => (
              <LearningCard
                key={item.id}
                item={item}
                onCopy={handleCopy}
                copiedId={copiedId}
              />
            ))
          )}
        </div>
      </motion.div>
    </div>
  );
}
