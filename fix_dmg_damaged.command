#!/bin/bash
# 兼容 zsh 与 bash，可用 bash 或 zsh 直接运行
set -e
export PATH="/usr/bin:/bin:/usr/sbin:/sbin:/usr/local/bin:$PATH"

# zsh：使用 0-based 数组，与 bash 一致
[[ -n "$ZSH_VERSION" ]] && setopt KSH_ARRAYS 2>/dev/null

# 兼容的带提示的 read（zsh 用 read "var?prompt"，bash 用 read -p "prompt" var）
read_prompt() {
  local varname="$1"
  local prompt="$2"
  if [[ -n "$ZSH_VERSION" ]]; then
    eval "read \"${varname}?${prompt}\""
  else
    read -p "$prompt" "$varname"
  fi
}

# 双击运行时先切到脚本所在目录
cd "$(dirname "$0")"

clear

echo "==============================================="
echo " macOS 安装包修复工具（清除 quarantine 标记）"
echo "==============================================="
echo ""

TARGET="$1"

if [[ -z "$TARGET" ]]; then
  echo "正在扫描桌面、下载目录和已挂载的磁盘映像..."
  echo ""

  candidates=()
  for dir in "$HOME/Desktop" "$HOME/Downloads"; do
    [[ -d "$dir" ]] || continue
    while IFS= read -r path; do
      candidates+=("$path")
    done < <(/usr/bin/find "$dir" -maxdepth 2 \( -name "*.dmg" -o -name "*.app" \) -print 2>/dev/null | /usr/bin/sort)
  done
  # 已挂载的 DMG 里的 .app（如 /Volumes/OpenClaw Manager/OpenClaw Manager.app）
  for vol in /Volumes/*/; do
    [[ -d "$vol" ]] || continue
    while IFS= read -r path; do
      candidates+=("$path")
    done < <(/usr/bin/find "$vol" -maxdepth 2 \( -name "*.dmg" -o -name "*.app" \) -print 2>/dev/null | /usr/bin/sort)
  done

  if (( ${#candidates[@]} == 0 )); then
    echo "没找到 .dmg 或 .app。"
    echo "可先挂载 DMG，或运行：$(basename "$0") /路径/到/应用.app"
    echo ""
    read_prompt TARGET "请输入 .dmg 或 .app 的完整路径： "
  elif (( ${#candidates[@]} == 1 )); then
    echo "找到 1 个文件："
    echo "1) ${candidates[0]}"
    echo ""
    read_prompt CONFIRM "是否处理这个文件？(y/n，默认 y): "
    if [[ -z "$CONFIRM" || "$CONFIRM" == "y" || "$CONFIRM" == "Y" ]]; then
      TARGET="${candidates[0]}"
    else
      read_prompt TARGET "请输入 .dmg 或 .app 的完整路径： "
    fi
  else
    echo "找到 ${#candidates[@]} 个文件："
    i=1
    for item in "${candidates[@]}"; do
      echo "$i) $item"
      ((i++))
    done
    echo ""
    read_prompt IDX "请输入要处理的编号（1-${#candidates[@]}）: "
    if [[ "$IDX" =~ ^[0-9]+$ ]] && (( IDX >= 1 && IDX <= ${#candidates[@]} )); then
      TARGET="${candidates[$((IDX-1))]}"
    else
      echo "[错误] 编号无效。"
      read_prompt _ "按回车键退出..."
      exit 1
    fi
  fi
fi

# 去掉可能的引号
TARGET="${TARGET#\"}"
TARGET="${TARGET%\"}"

if [[ ! -e "$TARGET" ]]; then
  echo ""
  echo "[错误] 文件不存在：$TARGET"
  read_prompt _ "按回车键退出..."
  exit 1
fi

# 挂载的 DMG 是只读的；若「应用程序」里已有同名 app，直接处理那个
if [[ "$TARGET" == /Volumes/* ]]; then
  app_name="$(basename "$TARGET")"
  applications_copy="/Applications/$app_name"
  if [[ -e "$applications_copy" ]]; then
    echo ""
    echo "[提示] 挂载盘为只读，将改为处理「应用程序」中的副本："
    echo "  $applications_copy"
    echo ""
    TARGET="$applications_copy"
  else
    echo ""
    echo "[提示] 该应用在已挂载的磁盘映像里，卷为只读，无法直接修改。"
    echo "且未在「应用程序」中找到「$app_name」。"
    echo ""
    echo "请先将该应用拖到「应用程序」文件夹，再重新运行本脚本。"
    echo ""
    read_prompt _ "按回车键退出..."
    exit 1
  fi
fi

echo ""
echo "正在处理：$TARGET"

echo "执行完整命令："
echo "  /usr/bin/xattr -rd com.apple.quarantine \"$TARGET\""
/usr/bin/xattr -rd com.apple.quarantine "$TARGET"

echo ""
echo "[完成] 已清除隔离标记。"
echo "现在请重新打开安装包或应用。"
echo ""
read_prompt _ "按回车键退出..."
