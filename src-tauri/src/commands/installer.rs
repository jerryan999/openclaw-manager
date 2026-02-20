use crate::utils::{platform, shell, bundled};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use tauri::command;

/// 环境检查结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentStatus {
    /// Node.js 是否安装
    pub node_installed: bool,
    /// Node.js 版本
    pub node_version: Option<String>,
    /// Node.js 版本是否满足要求 (>=22)
    pub node_version_ok: bool,
    /// 是否有打包的 Node.js
    pub has_bundled_nodejs: bool,
    /// Git 是否安装
    pub git_installed: bool,
    /// Git 版本
    pub git_version: Option<String>,
    /// 是否有离线安装包
    pub has_offline_package: bool,
    /// OpenClaw 是否安装
    pub openclaw_installed: bool,
    /// OpenClaw 版本
    pub openclaw_version: Option<String>,
    /// 配置目录是否存在
    pub config_dir_exists: bool,
    /// 是否全部就绪
    pub ready: bool,
    /// 操作系统
    pub os: String,
}

/// 安装进度
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallProgress {
    pub step: String,
    pub progress: u8,
    pub message: String,
    pub error: Option<String>,
}

/// 安装结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallResult {
    pub success: bool,
    pub message: String,
    pub error: Option<String>,
}

/// 检查环境状态
#[command]
pub async fn check_environment(app: tauri::AppHandle) -> Result<EnvironmentStatus, String> {
    info!("[环境检查] 开始检查系统环境...");

    let os = platform::get_os();
    info!("[环境检查] 操作系统: {}", os);

    // 检查 Node.js
    info!("[环境检查] 检查 Node.js...");
    let node_path = get_preferred_node_path();
    let node_version = get_node_version_from_path(node_path.as_deref());
    let node_installed = node_version.is_some();
    let node_version_ok = check_node_version_requirement(&node_version);
    info!(
        "[环境检查] Node.js: installed={}, version={:?}, version_ok={}, path={:?}",
        node_installed, node_version, node_version_ok, node_path
    );

    // 检查打包的 Node.js
    let has_bundled_nodejs = bundled::has_bundled_nodejs(&app);
    info!("[环境检查] 打包的 Node.js: {}", if has_bundled_nodejs { "存在" } else { "不存在" });

    // 检查 Git
    info!("[环境检查] 检查 Git...");
    let git_version = get_git_version();
    let git_installed = git_version.is_some();
    let git_path = if git_installed {
        get_command_path("git")
    } else {
        None
    };
    info!(
        "[环境检查] Git: installed={}, version={:?}, path={:?}",
        git_installed, git_version, git_path
    );

    // 检查 OpenClaw
    info!("[环境检查] 检查 OpenClaw...");
    let openclaw_version = get_openclaw_version();
    let openclaw_installed = openclaw_version.is_some();
    let openclaw_path = if openclaw_installed {
        shell::get_openclaw_path()
    } else {
        None
    };
    info!(
        "[环境检查] OpenClaw: installed={}, version={:?}, path={:?}",
        openclaw_installed, openclaw_version, openclaw_path
    );

    // 检查配置目录
    let config_dir = platform::get_config_dir();
    let config_dir_exists = std::path::Path::new(&config_dir).exists();
    info!(
        "[环境检查] 配置目录: {}, exists={}",
        config_dir, config_dir_exists
    );

    // 检查是否有离线安装包
    let has_offline_package = bundled::get_bundled_openclaw_package(&app).is_some();
    info!("[环境检查] 离线安装包: {}", if has_offline_package { "存在" } else { "不存在" });

    // 环境就绪判断：
    // 1. OpenClaw 已安装 → 就绪
    // 2. 有打包的 Node.js 和离线包 → 就绪（完全离线，无需任何依赖）
    // 3. 有离线包 + 已安装 Node.js → 就绪
    // 4. 无离线包：需要 Node.js + (Windows需要Git)
    let ready = if openclaw_installed {
        true
    } else if has_bundled_nodejs && has_offline_package {
        true
    } else if has_offline_package {
        node_installed && node_version_ok
    } else if platform::is_windows() {
        node_installed && node_version_ok && git_installed
    } else {
        node_installed && node_version_ok
    };
    info!(
        "[环境检查] 环境就绪状态: ready={}, 打包Node={}, 离线包={}, Windows={}",
        ready, has_bundled_nodejs, has_offline_package, platform::is_windows()
    );

    Ok(EnvironmentStatus {
        node_installed,
        node_version,
        node_version_ok,
        has_bundled_nodejs,
        git_installed,
        git_version,
        has_offline_package,
        openclaw_installed,
        openclaw_version,
        config_dir_exists,
        ready,
        os,
    })
}

/// 获取 Node.js 版本
/// 检测多个可能的安装路径，因为 GUI 应用不继承用户 shell 的 PATH
fn get_node_version_from_path(node_path: Option<&str>) -> Option<String> {
    let Some(path) = node_path else {
        return None;
    };
    if let Ok(v) = shell::run_command_output(path, &["--version"]) {
        let version = v.trim().to_string();
        if !version.is_empty() && version.starts_with('v') {
            return Some(version);
        }
    }
    None
}

fn get_preferred_node_path() -> Option<String> {
    if platform::is_windows() {
        if let Ok(runtime) = shell::get_windows_offline_runtime() {
            let node_exe = runtime.node_dir.join("node.exe");
            if node_exe.exists() {
                return Some(node_exe.display().to_string());
            }
        }

        for path in get_windows_node_paths() {
            if std::path::Path::new(&path).exists() {
                return Some(path);
            }
        }

        if let Ok(output) = shell::run_cmd_output("where node") {
            if let Some(path) = output.lines().map(str::trim).find(|line| !line.is_empty()) {
                return Some(path.to_string());
            }
        }
        None
    } else {
        let mut runtime_candidates: Vec<String> = Vec::new();
        if let Some(home) = dirs::home_dir() {
            let home_str = home.display().to_string();
            runtime_candidates.push(format!("{}/.openclaw-manager/runtime/node/bin/node", home_str));
            runtime_candidates.push(format!("{}/.openclaw-manager/runtime/node/node", home_str));
        }
        for path in runtime_candidates {
            if std::path::Path::new(&path).exists() {
                return Some(path);
            }
        }

        for path in get_unix_node_paths() {
            if std::path::Path::new(&path).exists() {
                return Some(path);
            }
        }

        if let Ok(path) = shell::run_bash_output(
            "source ~/.zshrc 2>/dev/null || source ~/.bashrc 2>/dev/null; command -v node 2>/dev/null",
        ) {
            let path = path.trim();
            if !path.is_empty() {
                return Some(path.to_string());
            }
        }
        None
    }
}

/// 获取 Git 版本
fn get_git_version() -> Option<String> {
    if platform::is_windows() {
        // Windows: 先尝试直接调用
        if let Ok(v) = shell::run_cmd_output("git --version") {
            if !v.is_empty() {
                info!("[环境检查] 通过 cmd 找到 Git: {}", v.trim());
                return Some(v.trim().to_string());
            }
        }
        
        if let Ok(v) = shell::run_powershell_output("git --version") {
            if !v.is_empty() {
                info!("[环境检查] 通过 PowerShell 找到 Git: {}", v.trim());
                return Some(v.trim().to_string());
            }
        }

        // 先尝试单独解压打包的 Git（不依赖完整 offline runtime，避免因无 Node zip 导致从不解压）
        if let Some(ref git_exe) = shell::ensure_windows_git_if_bundled() {
            if git_exe.exists() {
                let cmd = format!("\"{}\" --version", git_exe.display());
                if let Ok(v) = shell::run_cmd_output(&cmd) {
                    if !v.is_empty() {
                        info!("[环境检查] 使用离线 runtime 中的 Git: {}", v.trim());
                        return Some(v.trim().to_string());
                    }
                }
            }
        }

        // 离线 runtime 中的 Git（若完整 runtime 已就绪）
        if let Ok(runtime) = shell::get_windows_offline_runtime() {
            if let Some(ref git_exe) = runtime.git_exe {
                if git_exe.exists() {
                    let cmd = format!("\"{}\" --version", git_exe.display());
                    if let Ok(v) = shell::run_cmd_output(&cmd) {
                        if !v.is_empty() {
                            info!("[环境检查] 使用离线 runtime 中的 Git: {}", v.trim());
                            return Some(v.trim().to_string());
                        }
                    }
                }
            }
        }

        // 检查常见的 Git 安装路径
        let common_paths = vec![
            "C:\\Program Files\\Git\\cmd\\git.exe",
            "C:\\Program Files (x86)\\Git\\cmd\\git.exe",
            "C:\\Program Files\\Git\\bin\\git.exe",
            "C:\\Program Files (x86)\\Git\\bin\\git.exe",
        ];
        
        if let Some(home) = dirs::home_dir() {
            let user_local = format!("{}\\AppData\\Local\\Programs\\Git\\cmd\\git.exe", home.display());
            let user_git = format!("{}\\scoop\\apps\\git\\current\\cmd\\git.exe", home.display());
            
            for path in vec![user_local, user_git] {
                if std::path::Path::new(&path).exists() {
                    // 尝试执行找到的 git
                    let cmd = format!("\"{}\" --version", path);
                    if let Ok(v) = shell::run_cmd_output(&cmd) {
                        if !v.is_empty() {
                            info!("[环境检查] 在 {} 找到 Git: {}", path, v.trim());
                            return Some(v.trim().to_string());
                        }
                    }
                }
            }
        }
        
        for path in common_paths {
            if std::path::Path::new(path).exists() {
                let cmd = format!("\"{}\" --version", path);
                if let Ok(v) = shell::run_cmd_output(&cmd) {
                    if !v.is_empty() {
                        info!("[环境检查] 在 {} 找到 Git: {}", path, v.trim());
                        return Some(v.trim().to_string());
                    }
                }
            }
        }
        
        let reason = match shell::get_git_bundled_failure_reason() {
            Some(r) => format!("详情: {}", r),
            None => {
                let roots = shell::get_windows_resource_roots_for_diagnostic();
                format!(
                    "详情: 已找到 resources/git 下的 .zip 但解压或运行未成功。查找时 roots: {:?}",
                    roots
                )
            }
        };
        warn!(
            "[环境检查] 未找到 Git。请安装 Git 或确保 src-tauri/resources/git/ 下有 MinGit 的 .zip（如 git-windows-x64.zip）。{}",
            reason
        );
        None
    } else {
        // Unix: 直接调用
        if let Ok(v) = shell::run_command_output("git", &["--version"]) {
            if !v.is_empty() {
                info!("[环境检查] 找到 Git: {}", v.trim());
                return Some(v.trim().to_string());
            }
        }
        
        None
    }
}

/// 获取 Unix 系统上可能的 Node.js 路径
fn get_unix_node_paths() -> Vec<String> {
    let mut paths = Vec::new();

    if let Some(home) = dirs::home_dir() {
        let home_str = home.display().to_string();
        paths.push(format!("{}/.openclaw-manager/runtime/node/bin/node", home_str));
        paths.push(format!("{}/.openclaw-manager/runtime/node/node", home_str));
    }

    // Homebrew (macOS)
    paths.push("/opt/homebrew/bin/node".to_string()); // Apple Silicon
    paths.push("/usr/local/bin/node".to_string()); // Intel Mac

    // 系统安装
    paths.push("/usr/bin/node".to_string());

    // nvm (检查常见版本)
    if let Some(home) = dirs::home_dir() {
        let home_str = home.display().to_string();

        // nvm 默认版本
        paths.push(format!("{}/.nvm/versions/node/v22.0.0/bin/node", home_str));
        paths.push(format!("{}/.nvm/versions/node/v22.1.0/bin/node", home_str));
        paths.push(format!("{}/.nvm/versions/node/v22.2.0/bin/node", home_str));
        paths.push(format!("{}/.nvm/versions/node/v22.11.0/bin/node", home_str));
        paths.push(format!("{}/.nvm/versions/node/v22.12.0/bin/node", home_str));
        paths.push(format!("{}/.nvm/versions/node/v23.0.0/bin/node", home_str));

        // 尝试 nvm alias default（读取 nvm 的 default alias）
        let nvm_default = format!("{}/.nvm/alias/default", home_str);
        if let Ok(version) = std::fs::read_to_string(&nvm_default) {
            let version = version.trim();
            if !version.is_empty() {
                paths.insert(
                    0,
                    format!("{}/.nvm/versions/node/v{}/bin/node", home_str, version),
                );
            }
        }

        // fnm
        paths.push(format!("{}/.fnm/aliases/default/bin/node", home_str));

        // volta
        paths.push(format!("{}/.volta/bin/node", home_str));

        // asdf
        paths.push(format!("{}/.asdf/shims/node", home_str));

        // mise (formerly rtx)
        paths.push(format!("{}/.local/share/mise/shims/node", home_str));
    }

    paths
}

/// 获取 Windows 系统上可能的 Node.js 路径
fn get_windows_node_paths() -> Vec<String> {
    let mut paths = Vec::new();

    // 1. 标准安装路径 (Program Files)
    paths.push("C:\\Program Files\\nodejs\\node.exe".to_string());
    paths.push("C:\\Program Files (x86)\\nodejs\\node.exe".to_string());

    // 2. nvm for Windows (nvm4w) - 常见安装位置
    paths.push("C:\\nvm4w\\nodejs\\node.exe".to_string());

    // 3. 用户目录下的各种安装
    if let Some(home) = dirs::home_dir() {
        let home_str = home.display().to_string();

        // nvm for Windows 用户安装
        paths.push(format!(
            "{}\\AppData\\Roaming\\nvm\\current\\node.exe",
            home_str
        ));

        // fnm (Fast Node Manager) for Windows
        paths.push(format!(
            "{}\\AppData\\Roaming\\fnm\\aliases\\default\\node.exe",
            home_str
        ));
        paths.push(format!(
            "{}\\AppData\\Local\\fnm\\aliases\\default\\node.exe",
            home_str
        ));
        paths.push(format!("{}\\.fnm\\aliases\\default\\node.exe", home_str));

        // volta
        paths.push(format!(
            "{}\\AppData\\Local\\Volta\\bin\\node.exe",
            home_str
        ));
        // volta 通过 shim 调用，检查 bin 目录即可

        // scoop 安装
        paths.push(format!(
            "{}\\scoop\\apps\\nodejs\\current\\node.exe",
            home_str
        ));
        paths.push(format!(
            "{}\\scoop\\apps\\nodejs-lts\\current\\node.exe",
            home_str
        ));

        // chocolatey 安装
        paths.push("C:\\ProgramData\\chocolatey\\lib\\nodejs\\tools\\node.exe".to_string());
    }

    // 4. 从注册表读取的安装路径（通过环境变量间接获取）
    if let Ok(program_files) = std::env::var("ProgramFiles") {
        paths.push(format!("{}\\nodejs\\node.exe", program_files));
    }
    if let Ok(program_files_x86) = std::env::var("ProgramFiles(x86)") {
        paths.push(format!("{}\\nodejs\\node.exe", program_files_x86));
    }

    // 5. nvm-windows 的符号链接路径（NVM_SYMLINK 环境变量）
    if let Ok(nvm_symlink) = std::env::var("NVM_SYMLINK") {
        paths.insert(0, format!("{}\\node.exe", nvm_symlink));
    }

    // 6. nvm-windows 的 NVM_HOME 路径下的当前版本
    if let Ok(nvm_home) = std::env::var("NVM_HOME") {
        // 尝试读取当前激活的版本
        let settings_path = format!("{}\\settings.txt", nvm_home);
        if let Ok(content) = std::fs::read_to_string(&settings_path) {
            for line in content.lines() {
                if line.starts_with("current:") {
                    if let Some(version) = line.strip_prefix("current:") {
                        let version = version.trim();
                        if !version.is_empty() {
                            paths.insert(0, format!("{}\\v{}\\node.exe", nvm_home, version));
                        }
                    }
                }
            }
        }
    }

    paths
}

/// 获取 OpenClaw 版本
fn get_openclaw_version() -> Option<String> {
    // 使用 run_openclaw 统一处理各平台
    shell::run_openclaw(&["--version"])
        .ok()
        .map(|v| v.trim().to_string())
}

/// 检查 Node.js 版本是否 >= 22
fn check_node_version_requirement(version: &Option<String>) -> bool {
    if let Some(v) = version {
        // 解析版本号 "v22.1.0" -> 22
        let major = v
            .trim_start_matches('v')
            .split('.')
            .next()
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(0);
        major >= 22
    } else {
        false
    }
}

/// 安装 Node.js
#[command]
pub async fn install_nodejs() -> Result<InstallResult, String> {
    info!("[安装Node.js] 开始安装 Node.js...");
    let os = platform::get_os();
    info!("[安装Node.js] 检测到操作系统: {}", os);

    let result = match os.as_str() {
        "windows" => {
            info!("[安装Node.js] 使用 Windows 安装方式...");
            install_nodejs_windows().await
        }
        "macos" => {
            info!("[安装Node.js] 使用 macOS 安装方式 (Homebrew)...");
            install_nodejs_macos().await
        }
        "linux" => {
            info!("[安装Node.js] 使用 Linux 安装方式...");
            install_nodejs_linux().await
        }
        _ => {
            error!("[安装Node.js] 不支持的操作系统: {}", os);
            Ok(InstallResult {
                success: false,
                message: "不支持的操作系统".to_string(),
                error: Some(format!("不支持的操作系统: {}", os)),
            })
        }
    };

    match &result {
        Ok(r) if r.success => info!("[安装Node.js] ✓ 安装成功"),
        Ok(r) => warn!("[安装Node.js] ✗ 安装失败: {}", r.message),
        Err(e) => error!("[安装Node.js] ✗ 安装错误: {}", e),
    }

    result
}

/// Windows 安装 Node.js
async fn install_nodejs_windows() -> Result<InstallResult, String> {
    #[cfg(windows)]
    {
        match shell::get_windows_offline_runtime() {
            Ok(runtime) => {
                let node_exe = runtime.node_dir.join("node.exe");
                if !node_exe.exists() {
                    return Ok(InstallResult {
                        success: false,
                        message: "Node.js 安装失败".to_string(),
                        error: Some("离线 Node.js 运行时不存在".to_string()),
                    });
                }

                let version_cmd = format!("\"{}\" --version", node_exe.display());
                match shell::run_cmd_output(&version_cmd) {
                    Ok(version) => Ok(InstallResult {
                        success: true,
                        message: format!("Node.js 离线运行时已就绪: {}", version.trim()),
                        error: None,
                    }),
                    Err(e) => Ok(InstallResult {
                        success: false,
                        message: "Node.js 安装失败".to_string(),
                        error: Some(e),
                    }),
                }
            }
            Err(e) => Ok(InstallResult {
                success: false,
                message: "Node.js 安装失败".to_string(),
                error: Some(e),
            }),
        }
    }

    #[cfg(not(windows))]
    {
        Ok(InstallResult {
            success: false,
            message: "不支持的操作系统".to_string(),
            error: Some("install_nodejs_windows 仅支持 Windows".to_string()),
        })
    }
}

/// macOS 安装 Node.js
async fn install_nodejs_macos() -> Result<InstallResult, String> {
    // 使用 Homebrew 安装
    let script = r#"
# 检查 Homebrew
if ! command -v brew &> /dev/null; then
    echo "安装 Homebrew..."
    /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
    
    # 配置 PATH
    if [[ -f /opt/homebrew/bin/brew ]]; then
        eval "$(/opt/homebrew/bin/brew shellenv)"
    elif [[ -f /usr/local/bin/brew ]]; then
        eval "$(/usr/local/bin/brew shellenv)"
    fi
fi

echo "安装 Node.js 22..."
brew install node@22
brew link --overwrite node@22

# 验证安装
node --version
"#;

    match shell::run_bash_output(script) {
        Ok(output) => Ok(InstallResult {
            success: true,
            message: format!("Node.js 安装成功！{}", output),
            error: None,
        }),
        Err(e) => Ok(InstallResult {
            success: false,
            message: "Node.js 安装失败".to_string(),
            error: Some(e),
        }),
    }
}

/// Linux 安装 Node.js
async fn install_nodejs_linux() -> Result<InstallResult, String> {
    // 使用 NodeSource 仓库安装
    let script = r#"
# 检测包管理器
if command -v apt-get &> /dev/null; then
    echo "检测到 apt，使用 NodeSource 仓库..."
    curl -fsSL https://deb.nodesource.com/setup_22.x | sudo -E bash -
    sudo apt-get install -y nodejs
elif command -v dnf &> /dev/null; then
    echo "检测到 dnf，使用 NodeSource 仓库..."
    curl -fsSL https://rpm.nodesource.com/setup_22.x | sudo bash -
    sudo dnf install -y nodejs
elif command -v yum &> /dev/null; then
    echo "检测到 yum，使用 NodeSource 仓库..."
    curl -fsSL https://rpm.nodesource.com/setup_22.x | sudo bash -
    sudo yum install -y nodejs
elif command -v pacman &> /dev/null; then
    echo "检测到 pacman..."
    sudo pacman -S nodejs npm --noconfirm
else
    echo "无法检测到支持的包管理器"
    exit 1
fi

# 验证安装
node --version
"#;

    match shell::run_bash_output(script) {
        Ok(output) => Ok(InstallResult {
            success: true,
            message: format!("Node.js 安装成功！{}", output),
            error: None,
        }),
        Err(e) => Ok(InstallResult {
            success: false,
            message: "Node.js 安装失败".to_string(),
            error: Some(e),
        }),
    }
}

/// 安装 OpenClaw
#[command]
pub async fn install_openclaw(app: tauri::AppHandle) -> Result<InstallResult, String> {
    info!("[安装OpenClaw] 开始安装 OpenClaw...");
    let os = platform::get_os();
    info!("[安装OpenClaw] 检测到操作系统: {}", os);

    let result = match os.as_str() {
        "windows" => {
            info!("[安装OpenClaw] 使用 Windows 安装方式...");
            install_openclaw_windows(&app).await
        }
        _ => {
            info!("[安装OpenClaw] 使用 Unix 安装方式 (npm)...");
            install_openclaw_unix(&app).await
        }
    };

    match &result {
        Ok(r) if r.success => info!("[安装OpenClaw] ✓ 安装成功"),
        Ok(r) => warn!("[安装OpenClaw] ✗ 安装失败: {}", r.message),
        Err(e) => error!("[安装OpenClaw] ✗ 安装错误: {}", e),
    }

    result
}

/// 检查是否有打包的离线 OpenClaw 包（使用 AppHandle 获取资源路径）
fn get_bundled_openclaw_package_with_app(app: &tauri::AppHandle) -> Option<String> {
    // 优先使用 Tauri 的资源目录
    if let Some(path) = bundled::get_bundled_openclaw_package(app) {
        let path_str = path.display().to_string();
        info!("[安装OpenClaw] 通过 AppHandle 找到离线包: {}", path_str);
        return Some(path_str);
    }
    
    // 回退：在本地目录中查找（用于开发模式）
    let resource_dirs = vec![
        "resources/openclaw",
        "../resources/openclaw",
        "src-tauri/resources/openclaw",
        "openclaw",
    ];
    
    for dir in resource_dirs {
        let path = std::path::Path::new(dir).join("openclaw-zh.tgz");
        if path.is_file() {
            let path_str = path.display().to_string();
            info!("[安装OpenClaw] 找到本地离线包: {}", path_str);
            return Some(path_str);
        }
    }
    
    debug!("[安装OpenClaw] 未找到离线包，将使用在线安装");
    None
}

/// Windows 安装 OpenClaw
#[cfg(windows)]
async fn install_openclaw_windows(app: &tauri::AppHandle) -> Result<InstallResult, String> {
    // 优先使用 Windows 离线运行时（打包的 Node + OpenClaw 包）
    if let Ok(runtime) = shell::get_windows_offline_runtime() {
        if runtime.openclaw_cmd.exists() {
            return Ok(InstallResult {
                success: true,
                message: "OpenClaw 离线运行时已就绪。".to_string(),
                error: None,
            });
        }

        let npm_cmd = runtime.node_dir.join("npm.cmd");
        if !npm_cmd.exists() {
            return Ok(InstallResult {
                success: false,
                message: "OpenClaw 安装失败".to_string(),
                error: Some("离线 Node.js 运行时不完整：缺少 npm.cmd".to_string()),
            });
        }
        let install_cmd = format!(
            "\"{}\" install -g \"{}\" --prefix \"{}\" --no-audit --fund=false --loglevel=error",
            npm_cmd.display(),
            runtime.openclaw_package.display(),
            runtime.npm_prefix.display()
        );
        match shell::run_cmd_output(&install_cmd) {
            Ok(_) => {
                if runtime.openclaw_cmd.exists() || get_openclaw_version().is_some() {
                    return Ok(InstallResult {
                        success: true,
                        message: "OpenClaw 离线安装成功！".to_string(),
                        error: None,
                    });
                }
                return Ok(InstallResult {
                    success: false,
                    message: "OpenClaw 安装失败".to_string(),
                    error: Some("安装命令执行成功，但未找到 openclaw.cmd".to_string()),
                });
            }
            Err(e) => {
                return Ok(InstallResult {
                    success: false,
                    message: "OpenClaw 安装失败".to_string(),
                    error: Some(e),
                });
            }
        }
    }

    // 回退：使用打包的离线包或在线安装（需 Git）
    let bundled_package = get_bundled_openclaw_package_with_app(app);
    if bundled_package.is_none() && get_git_version().is_none() {
        return Ok(InstallResult {
            success: false,
            message: "Git 未安装".to_string(),
            error: Some(
                "在线安装 OpenClaw 需要 Git。\n\n两个解决方案：\n\
                方案1（推荐）：使用打包离线包\n  \
                - 下载包含 OpenClaw 离线包的完整版本\n  \
                - 无需 Git，安装更快更可靠\n\n\
                方案2：安装 Git\n  \
                1. 访问 https://git-scm.com/download/win\n  \
                2. 下载并安装 Git for Windows\n  \
                3. 安装完成后重启本应用\n  \
                或使用: winget install --id Git.Git -e --source winget"
                    .to_string(),
            ),
        });
    }

    let script = if let Some(package_path) = bundled_package {
        info!("[安装OpenClaw] 使用离线包: {}", package_path);
        format!(
            r#"
$ErrorActionPreference = 'Stop'

# 检查 Node.js
$nodeVersion = node --version 2>$null
if (-not $nodeVersion) {{
    Write-Host "错误：请先安装 Node.js"
    exit 1
}}

Write-Host "使用离线包安装 OpenClaw（无需 Git）..."
Write-Host "包路径: {}"
npm install -g "{}" --unsafe-perm

# 刷新 PATH
$npmPrefix = npm prefix -g
$env:Path = "$env:Path;$npmPrefix"

# 验证安装
$openclawVersion = openclaw --version 2>$null
if ($openclawVersion) {{
    Write-Host "OpenClaw 安装成功: $openclawVersion"
    exit 0
}} else {{
    Write-Host "OpenClaw 安装失败"
    exit 1
}}
"#,
            package_path, package_path
        )
    } else {
        info!("[安装OpenClaw] 使用在线安装，需要 Git");
        r#"
$ErrorActionPreference = 'Stop'

# 检查 Node.js
$nodeVersion = node --version 2>$null
if (-not $nodeVersion) {
    Write-Host "错误：请先安装 Node.js"
    exit 1
}

# 检查 Git（在线安装需要）
$gitVersion = git --version 2>$null
if (-not $gitVersion) {
    Write-Host "错误：在线安装需要 Git"
    Write-Host "下载地址: https://git-scm.com/download/win"
    exit 1
}

Write-Host "使用 npm 在线安装 OpenClaw..."
npm install -g @jerryan999/openclaw-zh --unsafe-perm

# 刷新 PATH
$npmPrefix = npm prefix -g
$env:Path = "$env:Path;$npmPrefix"

# 验证安装
$openclawVersion = openclaw --version 2>$null
if ($openclawVersion) {
    Write-Host "OpenClaw 安装成功: $openclawVersion"
    exit 0
} else {
    Write-Host "OpenClaw 安装失败"
    exit 1
}
"#
        .to_string()
    };

    match shell::run_powershell_output(&script) {
        Ok(_) => {
            if get_openclaw_version().is_some() {
                Ok(InstallResult {
                    success: true,
                    message: "OpenClaw 安装成功！".to_string(),
                    error: None,
                })
            } else {
                Ok(InstallResult {
                    success: false,
                    message: "OpenClaw 安装失败".to_string(),
                    error: Some("安装命令执行成功，但 openclaw 未在 PATH 中".to_string()),
                })
            }
        }
        Err(e) => Ok(InstallResult {
            success: false,
            message: "OpenClaw 安装失败".to_string(),
            error: Some(e),
        }),
    }
}

#[cfg(not(windows))]
async fn install_openclaw_windows(_app: &tauri::AppHandle) -> Result<InstallResult, String> {
    Ok(InstallResult {
        success: false,
        message: "不支持的操作系统".to_string(),
        error: Some("install_openclaw_windows 仅支持 Windows".to_string()),
    })
}

/// Unix 系统安装 OpenClaw
async fn install_openclaw_unix(app: &tauri::AppHandle) -> Result<InstallResult, String> {
    // 检查是否有打包的离线包
    let bundled_package = get_bundled_openclaw_package_with_app(app);
    
    let script = if let Some(package_path) = bundled_package {
        info!("[安装OpenClaw] 使用离线包: {}", package_path);
        format!(
            r#"
# 检查 Node.js
if ! command -v node &> /dev/null; then
    echo "错误：请先安装 Node.js"
    exit 1
fi

echo "使用离线包安装 OpenClaw（无需 Git，更快更可靠）..."
echo "包路径: {}"
npm install -g "{}"

# 刷新命令缓存
hash -r 2>/dev/null || true
export PATH="$PATH:$(npm prefix -g)/bin"

# 验证安装
openclaw --version
"#,
            package_path, package_path
        )
    } else {
        info!("[安装OpenClaw] 使用在线安装");
        r#"
# 检查 Node.js
if ! command -v node &> /dev/null; then
    echo "错误：请先安装 Node.js"
    exit 1
fi

echo "使用 npm 在线安装 OpenClaw..."
npm install -g @jerryan999/openclaw-zh --unsafe-perm

# 刷新命令缓存
hash -r 2>/dev/null || true
export PATH="$PATH:$(npm prefix -g)/bin"

# 验证安装
openclaw --version
"#
        .to_string()
    };

    match shell::run_bash_output(&script) {
        Ok(output) => Ok(InstallResult {
            success: true,
            message: format!("OpenClaw 安装成功！{}", output),
            error: None,
        }),
        Err(e) => Ok(InstallResult {
            success: false,
            message: "OpenClaw 安装失败".to_string(),
            error: Some(e),
        }),
    }
}

/// 初始化 OpenClaw 配置
#[command]
pub async fn init_openclaw_config() -> Result<InstallResult, String> {
    info!("[初始化配置] 开始初始化 OpenClaw 配置...");

    let config_dir = platform::get_config_dir();
    info!("[初始化配置] 配置目录: {}", config_dir);

    // 创建配置目录
    info!("[初始化配置] 创建配置目录...");
    if let Err(e) = std::fs::create_dir_all(&config_dir) {
        error!("[初始化配置] ✗ 创建配置目录失败: {}", e);
        return Ok(InstallResult {
            success: false,
            message: "创建配置目录失败".to_string(),
            error: Some(e.to_string()),
        });
    }

    // 创建子目录
    let subdirs = ["agents/main/sessions", "agents/main/agent", "credentials"];
    for subdir in subdirs {
        let path = format!("{}/{}", config_dir, subdir);
        info!("[初始化配置] 创建子目录: {}", subdir);
        if let Err(e) = std::fs::create_dir_all(&path) {
            error!("[初始化配置] ✗ 创建目录失败: {} - {}", subdir, e);
            return Ok(InstallResult {
                success: false,
                message: format!("创建目录失败: {}", subdir),
                error: Some(e.to_string()),
            });
        }
    }

    // 设置配置目录权限为 700（与 shell 脚本 chmod 700 一致）
    // 仅在 Unix 系统上执行
    #[cfg(unix)]
    {
        info!("[初始化配置] 设置目录权限为 700...");
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = std::fs::metadata(&config_dir) {
            let mut perms = metadata.permissions();
            perms.set_mode(0o700);
            if let Err(e) = std::fs::set_permissions(&config_dir, perms) {
                warn!("[初始化配置] 设置权限失败: {}", e);
            } else {
                info!("[初始化配置] ✓ 权限设置成功");
            }
        }
    }

    // 设置 gateway mode 为 local
    info!("[初始化配置] 执行: openclaw config set gateway.mode local");
    let result = shell::run_openclaw(&["config", "set", "gateway.mode", "local"]);

    match result {
        Ok(output) => {
            info!("[初始化配置] ✓ 配置初始化成功");
            debug!("[初始化配置] 命令输出: {}", output);
            Ok(InstallResult {
                success: true,
                message: "配置初始化成功！".to_string(),
                error: None,
            })
        }
        Err(e) => {
            error!("[初始化配置] ✗ 配置初始化失败: {}", e);
            Ok(InstallResult {
                success: false,
                message: "配置初始化失败".to_string(),
                error: Some(e),
            })
        }
    }
}

/// 打开终端执行安装脚本（用于需要管理员权限的场景）
#[command]
pub async fn open_install_terminal(install_type: String) -> Result<String, String> {
    match install_type.as_str() {
        "nodejs" => open_nodejs_install_terminal().await,
        "openclaw" => open_openclaw_install_terminal().await,
        _ => Err(format!("未知的安装类型: {}", install_type)),
    }
}

/// 打开诊断终端（用于手动排查环境问题）
#[command]
pub async fn open_debug_terminal() -> Result<String, String> {
    if platform::is_windows() {
        let rt_path = dirs::data_local_dir()
            .map(|d| d.join("OpenClawManager").join("runtime"))
            .unwrap_or_else(|| std::path::PathBuf::from("C:\\OpenClawManager\\runtime"))
            .display()
            .to_string();
        let script_body = r#"$rt = "__RUNTIME_PATH__"
$p1 = [IO.Path]::Combine($rt,'node'); $p2 = [IO.Path]::Combine($rt,'npm-global'); $p3 = [IO.Path]::Combine($rt,'git','cmd'); $p4 = [IO.Path]::Combine($rt,'git','mingw64','bin'); $env:PATH = "$p1;$p2;$p3;$p4;$env:PATH"
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "    OpenClaw 诊断终端" -ForegroundColor White
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "[当前运行时]" -ForegroundColor Cyan
Write-Host "  优先离线 runtime；若离线不存在则回退系统/外部环境"
Write-Host ""
Write-Host "[离线 Runtime 路径]" -ForegroundColor Yellow
Write-Host "runtime: $rt"
Write-Host ("node(runtime): " + [IO.Path]::Combine($rt,'node','node.exe'))
Write-Host ("npm(runtime): " + [IO.Path]::Combine($rt,'node','npm.cmd'))
Write-Host ("openclaw(runtime): " + [IO.Path]::Combine($rt,'npm-global','openclaw.cmd'))
Write-Host ("git(runtime): " + [IO.Path]::Combine($rt,'git','cmd','git.exe'))
Write-Host ("node(runtime exists): " + (Test-Path ([IO.Path]::Combine($rt,'node','node.exe'))))
Write-Host ("npm(runtime exists): " + (Test-Path ([IO.Path]::Combine($rt,'node','npm.cmd'))))
Write-Host ("openclaw(runtime exists): " + (Test-Path ([IO.Path]::Combine($rt,'npm-global','openclaw.cmd'))))
Write-Host ("git(runtime exists): " + (Test-Path ([IO.Path]::Combine($rt,'git','cmd','git.exe'))))
Write-Host ""
function Show-Version($name) {
  $cmd = Get-Command $name -ErrorAction SilentlyContinue
  if (-not $cmd) {
    Write-Host ("$name: NOT FOUND")
    return
  }
  $path = $cmd.Source
  $source = if ($path.StartsWith($rt, [System.StringComparison]::OrdinalIgnoreCase)) { "离线 runtime" } else { "系统/外部" }
  $version = (& $path --version 2>&1 | Select-Object -First 1)
  if (-not $version) { $version = "(无版本输出)" }
  Write-Host ("$name: $version")
  Write-Host ("  path: $path")
  Write-Host ("  source: $source")
}
Write-Host "[版本检测]" -ForegroundColor Yellow
Show-Version "node"
Show-Version "npm"
Show-Version "git"
Show-Version "openclaw"
Write-Host ""
Write-Host "[路径检测]" -ForegroundColor Yellow
Get-Command node -All -ErrorAction SilentlyContinue | ForEach-Object { $_.Source }
Get-Command npm -All -ErrorAction SilentlyContinue | ForEach-Object { $_.Source }
Get-Command git -All -ErrorAction SilentlyContinue | ForEach-Object { $_.Source }
Get-Command openclaw -All -ErrorAction SilentlyContinue | ForEach-Object { $_.Source }
Write-Host ""
Write-Host "[提示] 可继续手动执行命令排查问题。" -ForegroundColor Green
"#
        .replace("__RUNTIME_PATH__", &rt_path.replace('"', "`\""));
        let tmp = std::env::temp_dir().join("openclaw_debug_terminal.ps1");
        std::fs::write(&tmp, &script_body).map_err(|e| format!("写入脚本失败: {}", e))?;
        // 直接用 Command 启动 powershell -File <path>，避免通过 -Command 传参导致的路径/引号解析问题
        std::process::Command::new("powershell")
            .args(["-NoExit", "-ExecutionPolicy", "Bypass", "-File"])
            .arg(&tmp)
            .spawn()
            .map_err(|e| format!("启动诊断终端失败: {}", e))?;
        Ok("已打开诊断终端".to_string())
    } else if platform::is_macos() {
        let script_content = r#"#!/bin/bash
clear
RUNTIME="${HOME}/.openclaw-manager/runtime"
prepend_path_if_dir() {
  [ -d "$1" ] && PATH="$1:$PATH"
}
apply_runtime_priority_path() {
  prepend_path_if_dir "$RUNTIME/node/bin"
  prepend_path_if_dir "$RUNTIME/node"
  prepend_path_if_dir "$RUNTIME/npm-global/bin"
  prepend_path_if_dir "$RUNTIME/npm-global"
  prepend_path_if_dir "$RUNTIME/git/bin"
  prepend_path_if_dir "$RUNTIME/git/cmd"
  prepend_path_if_dir "$RUNTIME/git/mingw64/bin"
}
apply_runtime_priority_path
[ -f "$HOME/.zshrc" ] && source "$HOME/.zshrc" >/dev/null 2>&1 || true
[ -f "$HOME/.bashrc" ] && source "$HOME/.bashrc" >/dev/null 2>&1 || true
apply_runtime_priority_path
export PATH
echo "========================================"
echo "    OpenClaw 诊断终端"
echo "========================================"
echo ""
echo "[当前运行时]"
echo "  优先离线 runtime；若离线不存在则回退系统/用户 shell 环境"
echo ""
echo "[离线 Runtime 路径]"
echo "runtime: $RUNTIME"
echo "node(runtime): $RUNTIME/node/bin/node"
echo "npm(runtime): $RUNTIME/node/bin/npm"
echo "openclaw(runtime): $RUNTIME/npm-global/bin/openclaw"
echo "git(runtime): $RUNTIME/git/bin/git"
echo "node(runtime exists): $(test -x "$RUNTIME/node/bin/node" && echo yes || echo no)"
echo "npm(runtime exists): $(test -x "$RUNTIME/node/bin/npm" && echo yes || echo no)"
echo "openclaw(runtime exists): $(test -x "$RUNTIME/npm-global/bin/openclaw" && echo yes || echo no)"
echo "git(runtime exists): $(test -x "$RUNTIME/git/bin/git" && echo yes || echo no)"
echo ""
show_version() {
  local name="$1"
  local path source version shell_path shell_dir candidate
  path="$(command -v "$name" 2>/dev/null || true)"
  source="系统/外部"
  case "$path" in
    "$RUNTIME"/*) source="离线 runtime" ;;
  esac
  if [ -z "$path" ]; then
    for candidate in "$HOME/.nvm/versions/node/"*/bin/"$name" "$HOME/.npm-global/bin/$name" "/opt/homebrew/bin/$name" "/usr/local/bin/$name" "/usr/bin/$name"; do
      if [ -x "$candidate" ]; then
        path="$candidate"
        shell_dir="$(dirname "$candidate")"
        PATH="$shell_dir:$PATH"
        export PATH
        source="系统路径兜底"
        break
      fi
    done
  fi
  if [ -z "$path" ]; then
    shell_path="$(zsh -lc "command -v $name" 2>/dev/null | head -n 1 || true)"
    [ -z "$shell_path" ] && shell_path="$(bash -lc "command -v $name" 2>/dev/null | head -n 1 || true)"
    if [ -n "$shell_path" ] && [ -x "$shell_path" ]; then
      path="$shell_path"
      shell_dir="$(dirname "$shell_path")"
      PATH="$shell_dir:$PATH"
      export PATH
      source="用户 shell 兜底"
    else
      echo "$name: NOT FOUND (当前诊断终端 PATH)"
      if [ -n "$shell_path" ]; then
        echo "  user-shell-path: $shell_path"
      fi
      return
    fi
  fi
  if [ "$name" = "npm" ]; then
    version="$("$path" -v 2>/dev/null | head -n 1)"
  else
    version="$("$path" --version 2>&1 | head -n 1)"
  fi
  [ -z "$version" ] && version="(无版本输出)"
  echo "$name: $version"
  echo "  path: $path"
  echo "  source: $source"
}
echo "[版本检测]"
show_version node
show_version npm
show_version git
show_version openclaw
echo ""
echo "[路径检测]"
type -a node 2>/dev/null || true
type -a npm 2>/dev/null || true
type -a git 2>/dev/null || true
type -a openclaw 2>/dev/null || true
echo ""
echo "[提示] 可继续手动执行命令排查问题。"
echo ""
unset npm_config_prefix NPM_CONFIG_PREFIX
export NO_COLOR=1
enter_shell_keep_path() {
  local sh="${SHELL:-/bin/bash}"
  case "$(basename "$sh")" in
    zsh) exec "$sh" -f -i ;;
    bash) exec "$sh" --noprofile --norc -i ;;
    *) exec "$sh" -i ;;
  esac
}
enter_shell_keep_path
"#;

        let script_path = "/tmp/openclaw_debug_terminal.command";
        std::fs::write(script_path, script_content).map_err(|e| format!("创建脚本失败: {}", e))?;

        std::process::Command::new("chmod")
            .args(["+x", script_path])
            .output()
            .map_err(|e| format!("设置权限失败: {}", e))?;

        std::process::Command::new("open")
            .arg(script_path)
            .spawn()
            .map_err(|e| format!("启动终端失败: {}", e))?;

        Ok("已打开诊断终端".to_string())
    } else {
        let script_content = r#"#!/bin/bash
clear
RUNTIME="${HOME}/.openclaw-manager/runtime"
prepend_path_if_dir() {
  [ -d "$1" ] && PATH="$1:$PATH"
}
apply_runtime_priority_path() {
  prepend_path_if_dir "$RUNTIME/node/bin"
  prepend_path_if_dir "$RUNTIME/node"
  prepend_path_if_dir "$RUNTIME/npm-global/bin"
  prepend_path_if_dir "$RUNTIME/npm-global"
  prepend_path_if_dir "$RUNTIME/git/bin"
  prepend_path_if_dir "$RUNTIME/git/cmd"
  prepend_path_if_dir "$RUNTIME/git/mingw64/bin"
}
apply_runtime_priority_path
[ -f "$HOME/.zshrc" ] && source "$HOME/.zshrc" >/dev/null 2>&1 || true
[ -f "$HOME/.bashrc" ] && source "$HOME/.bashrc" >/dev/null 2>&1 || true
apply_runtime_priority_path
export PATH
echo "========================================"
echo "    OpenClaw 诊断终端"
echo "========================================"
echo ""
echo "[当前运行时]"
echo "  优先离线 runtime；若离线不存在则回退系统/用户 shell 环境"
echo ""
echo "[离线 Runtime 路径]"
echo "runtime: $RUNTIME"
echo "node(runtime): $RUNTIME/node/bin/node"
echo "npm(runtime): $RUNTIME/node/bin/npm"
echo "openclaw(runtime): $RUNTIME/npm-global/bin/openclaw"
echo "git(runtime): $RUNTIME/git/bin/git"
echo "node(runtime exists): $(test -x "$RUNTIME/node/bin/node" && echo yes || echo no)"
echo "npm(runtime exists): $(test -x "$RUNTIME/node/bin/npm" && echo yes || echo no)"
echo "openclaw(runtime exists): $(test -x "$RUNTIME/npm-global/bin/openclaw" && echo yes || echo no)"
echo "git(runtime exists): $(test -x "$RUNTIME/git/bin/git" && echo yes || echo no)"
echo ""
show_version() {
  local name="$1"
  local path source version shell_path shell_dir candidate
  path="$(command -v "$name" 2>/dev/null || true)"
  source="系统/外部"
  case "$path" in
    "$RUNTIME"/*) source="离线 runtime" ;;
  esac
  if [ -z "$path" ]; then
    for candidate in "$HOME/.nvm/versions/node/"*/bin/"$name" "$HOME/.npm-global/bin/$name" "/opt/homebrew/bin/$name" "/usr/local/bin/$name" "/usr/bin/$name"; do
      if [ -x "$candidate" ]; then
        path="$candidate"
        shell_dir="$(dirname "$candidate")"
        PATH="$shell_dir:$PATH"
        export PATH
        source="系统路径兜底"
        break
      fi
    done
  fi
  if [ -z "$path" ]; then
    shell_path="$(zsh -lc "command -v $name" 2>/dev/null | head -n 1 || true)"
    [ -z "$shell_path" ] && shell_path="$(bash -lc "command -v $name" 2>/dev/null | head -n 1 || true)"
    if [ -n "$shell_path" ] && [ -x "$shell_path" ]; then
      path="$shell_path"
      shell_dir="$(dirname "$shell_path")"
      PATH="$shell_dir:$PATH"
      export PATH
      source="用户 shell 兜底"
    else
      echo "$name: NOT FOUND (当前诊断终端 PATH)"
      if [ -n "$shell_path" ]; then
        echo "  user-shell-path: $shell_path"
      fi
      return
    fi
  fi
  if [ "$name" = "npm" ]; then
    version="$("$path" -v 2>/dev/null | head -n 1)"
  else
    version="$("$path" --version 2>&1 | head -n 1)"
  fi
  [ -z "$version" ] && version="(无版本输出)"
  echo "$name: $version"
  echo "  path: $path"
  echo "  source: $source"
}
echo "[版本检测]"
show_version node
show_version npm
show_version git
show_version openclaw
echo ""
echo "[路径检测]"
type -a node 2>/dev/null || true
type -a npm 2>/dev/null || true
type -a git 2>/dev/null || true
type -a openclaw 2>/dev/null || true
echo ""
echo "[提示] 可继续手动执行命令排查问题。"
echo ""
unset npm_config_prefix NPM_CONFIG_PREFIX
export NO_COLOR=1
enter_shell_keep_path() {
  local sh="${SHELL:-/bin/bash}"
  case "$(basename "$sh")" in
    zsh) exec "$sh" -f -i ;;
    bash) exec "$sh" --noprofile --norc -i ;;
    *) exec "$sh" -i ;;
  esac
}
enter_shell_keep_path
"#;

        let script_path = "/tmp/openclaw_debug_terminal.sh";
        std::fs::write(script_path, script_content).map_err(|e| format!("创建脚本失败: {}", e))?;

        std::process::Command::new("chmod")
            .args(["+x", script_path])
            .output()
            .map_err(|e| format!("设置权限失败: {}", e))?;

        let terminals = ["gnome-terminal", "xfce4-terminal", "konsole", "xterm"];
        for term in terminals {
            if std::process::Command::new(term)
                .args(["--", script_path])
                .spawn()
                .is_ok()
            {
                return Ok("已打开诊断终端".to_string());
            }
        }

        Err("无法启动终端，请手动在终端执行 node --version / openclaw --version".to_string())
    }
}

/// 打开终端安装 Node.js
async fn open_nodejs_install_terminal() -> Result<String, String> {
    if platform::is_windows() {
        // Windows: 打开 PowerShell 执行安装
        let script = r#"
Start-Process powershell -ArgumentList '-NoExit', '-Command', '
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "    Node.js 安装向导" -ForegroundColor White
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# 检查 winget
$hasWinget = Get-Command winget -ErrorAction SilentlyContinue
if ($hasWinget) {
    Write-Host "正在使用 winget 安装 Node.js 22..." -ForegroundColor Yellow
    winget install --id OpenJS.NodeJS.LTS --accept-source-agreements --accept-package-agreements
} else {
    Write-Host "请从以下地址下载安装 Node.js:" -ForegroundColor Yellow
    Write-Host "https://nodejs.org/en/download" -ForegroundColor Green
    Write-Host ""
    Start-Process "https://nodejs.org/en/download"
}

Write-Host ""
Write-Host "安装完成后请重启 OpenClaw Manager" -ForegroundColor Green
Write-Host ""
Read-Host "按回车键关闭此窗口"
' -Verb RunAs
"#;
        shell::run_powershell_output(script)?;
        Ok("已打开安装终端".to_string())
    } else if platform::is_macos() {
        // macOS: 打开 Terminal.app
        let script_content = r#"#!/bin/bash
clear
echo "========================================"
echo "    Node.js 安装向导"
echo "========================================"
echo ""

# 检查 Homebrew
if ! command -v brew &> /dev/null; then
    echo "正在安装 Homebrew..."
    /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
    
    if [[ -f /opt/homebrew/bin/brew ]]; then
        eval "$(/opt/homebrew/bin/brew shellenv)"
    elif [[ -f /usr/local/bin/brew ]]; then
        eval "$(/usr/local/bin/brew shellenv)"
    fi
fi

echo "正在安装 Node.js 22..."
brew install node@22
brew link --overwrite node@22

echo ""
echo "安装完成！"
node --version
echo ""
read -p "按回车键关闭此窗口..."
"#;

        let script_path = "/tmp/openclaw_install_nodejs.command";
        std::fs::write(script_path, script_content).map_err(|e| format!("创建脚本失败: {}", e))?;

        std::process::Command::new("chmod")
            .args(["+x", script_path])
            .output()
            .map_err(|e| format!("设置权限失败: {}", e))?;

        std::process::Command::new("open")
            .arg(script_path)
            .spawn()
            .map_err(|e| format!("启动终端失败: {}", e))?;

        Ok("已打开安装终端".to_string())
    } else {
        Err("请手动安装 Node.js: https://nodejs.org/".to_string())
    }
}

/// 打开终端安装 OpenClaw
async fn open_openclaw_install_terminal() -> Result<String, String> {
    if platform::is_windows() {
        let script = r#"
Start-Process powershell -ArgumentList '-NoExit', '-Command', '
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "    OpenClaw 安装向导" -ForegroundColor White
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

Write-Host "正在安装 OpenClaw 中文版（无广告版）..." -ForegroundColor Yellow
npm install -g @jerryan999/openclaw-zh

Write-Host ""
Write-Host "初始化配置..."
openclaw config set gateway.mode local

Write-Host ""
Write-Host "安装完成！" -ForegroundColor Green
openclaw --version
Write-Host ""
Read-Host "按回车键关闭此窗口"
'
"#;
        shell::run_powershell_output(script)?;
        Ok("已打开安装终端".to_string())
    } else if platform::is_macos() {
        let script_content = r#"#!/bin/bash
clear
echo "========================================"
echo "    OpenClaw 安装向导"
echo "========================================"
echo ""

echo "正在安装 OpenClaw 中文版（无广告版）..."
npm install -g @jerryan999/openclaw-zh

# 刷新命令缓存，确保能找到新安装的 openclaw 命令
hash -r 2>/dev/null || true
export PATH="$PATH:$(npm prefix -g)/bin"

echo ""
echo "初始化配置..."
openclaw config set gateway.mode local 2>/dev/null || true

mkdir -p ~/.openclaw/agents/main/sessions
mkdir -p ~/.openclaw/agents/main/agent
mkdir -p ~/.openclaw/credentials

echo ""
echo "安装完成！"
openclaw --version
echo ""
read -p "按回车键关闭此窗口..."
"#;

        let script_path = "/tmp/openclaw_install_openclaw.command";
        std::fs::write(script_path, script_content).map_err(|e| format!("创建脚本失败: {}", e))?;

        std::process::Command::new("chmod")
            .args(["+x", script_path])
            .output()
            .map_err(|e| format!("设置权限失败: {}", e))?;

        std::process::Command::new("open")
            .arg(script_path)
            .spawn()
            .map_err(|e| format!("启动终端失败: {}", e))?;

        Ok("已打开安装终端".to_string())
    } else {
        // Linux
        let script_content = r#"#!/bin/bash
clear
echo "========================================"
echo "    OpenClaw 安装向导"
echo "========================================"
echo ""

echo "正在安装 OpenClaw 中文版（无广告版）..."
npm install -g @jerryan999/openclaw-zh

# 刷新命令缓存，确保能找到新安装的 openclaw 命令
hash -r 2>/dev/null || true
export PATH="$PATH:$(npm prefix -g)/bin"

echo ""
echo "初始化配置..."
openclaw config set gateway.mode local 2>/dev/null || true

mkdir -p ~/.openclaw/agents/main/sessions
mkdir -p ~/.openclaw/agents/main/agent
mkdir -p ~/.openclaw/credentials

echo ""
echo "安装完成！"
openclaw --version
echo ""
read -p "按回车键关闭..."
"#;

        let script_path = "/tmp/openclaw_install_openclaw.sh";
        std::fs::write(script_path, script_content).map_err(|e| format!("创建脚本失败: {}", e))?;

        std::process::Command::new("chmod")
            .args(["+x", script_path])
            .output()
            .map_err(|e| format!("设置权限失败: {}", e))?;

        // 尝试不同的终端
        let terminals = ["gnome-terminal", "xfce4-terminal", "konsole", "xterm"];
        for term in terminals {
            if std::process::Command::new(term)
                .args(["--", script_path])
                .spawn()
                .is_ok()
            {
                return Ok("已打开安装终端".to_string());
            }
        }

        Err("无法启动终端，请手动运行: npm install -g @jerryan999/openclaw-zh".to_string())
    }
}

/// 卸载 OpenClaw
#[command]
pub async fn uninstall_openclaw() -> Result<InstallResult, String> {
    info!("[卸载OpenClaw] 开始卸载 OpenClaw...");
    let os = platform::get_os();
    info!("[卸载OpenClaw] 检测到操作系统: {}", os);

    // 先停止服务
    info!("[卸载OpenClaw] 尝试停止服务...");
    let _ = shell::run_openclaw(&["gateway", "stop"]);
    std::thread::sleep(std::time::Duration::from_millis(500));

    let result = match os.as_str() {
        "windows" => {
            info!("[卸载OpenClaw] 使用 Windows 卸载方式...");
            uninstall_openclaw_windows().await
        }
        _ => {
            info!("[卸载OpenClaw] 使用 Unix 卸载方式 (npm)...");
            uninstall_openclaw_unix().await
        }
    };

    match &result {
        Ok(r) if r.success => info!("[卸载OpenClaw] ✓ 卸载成功"),
        Ok(r) => warn!("[卸载OpenClaw] ✗ 卸载失败: {}", r.message),
        Err(e) => error!("[卸载OpenClaw] ✗ 卸载错误: {}", e),
    }

    result
}

/// Windows 卸载 OpenClaw
async fn uninstall_openclaw_windows() -> Result<InstallResult, String> {
    // 使用 cmd.exe 执行 npm uninstall，避免 PowerShell 执行策略问题
    info!("[卸载OpenClaw] 执行 npm uninstall -g @jerryan999/openclaw-zh...");

    match shell::run_cmd_output("npm uninstall -g @jerryan999/openclaw-zh") {
        Ok(output) => {
            info!("[卸载OpenClaw] npm 输出: {}", output);

            // 验证卸载是否成功
            std::thread::sleep(std::time::Duration::from_millis(500));
            if get_openclaw_version().is_none() {
                Ok(InstallResult {
                    success: true,
                    message: "OpenClaw 已成功卸载！".to_string(),
                    error: None,
                })
            } else {
                Ok(InstallResult {
                    success: false,
                    message: "卸载命令已执行，但 OpenClaw 仍然存在，请尝试手动卸载".to_string(),
                    error: Some(output),
                })
            }
        }
        Err(e) => {
            warn!("[卸载OpenClaw] npm uninstall 失败: {}", e);
            Ok(InstallResult {
                success: false,
                message: "OpenClaw 卸载失败".to_string(),
                error: Some(e),
            })
        }
    }
}

/// Unix 系统卸载 OpenClaw（Windows 下为占位，不应被调用）
#[cfg(windows)]
async fn uninstall_openclaw_unix() -> Result<InstallResult, String> {
    Err("Unix uninstall is not available on Windows".to_string())
}

/// Unix 系统卸载 OpenClaw
/// 1) 在登录 shell 中执行，保证 nvm/fnm 等环境与用户终端一致
/// 2) 使用与 openclaw 同目录的 npm，确保从正确的全局空间卸载
#[cfg(not(windows))]
async fn uninstall_openclaw_unix() -> Result<InstallResult, String> {
    let npm_cmd = shell::get_openclaw_path()
        .and_then(|p| {
            std::path::Path::new(&p)
                .parent()
                .map(|dir| dir.join("npm").display().to_string())
        })
        .filter(|npm_path| std::path::Path::new(npm_path).exists());

    let script = if let Some(ref npm) = npm_cmd {
        info!("[卸载OpenClaw] 使用与 openclaw 同目录的 npm 执行卸载: {}", npm);
        // 使用与 openclaw 同目录的 npm，确保从正确的全局空间卸载（如 nvm 安装的包）
        format!(
            r#"echo "卸载 OpenClaw..."
'{}' uninstall -g @jerryan999/openclaw-zh

# 验证卸载
if command -v openclaw &> /dev/null; then
    echo "警告：openclaw 命令仍然存在"
    exit 1
else
    echo "OpenClaw 已成功卸载"
    exit 0
fi"#,
            npm
        )
    } else {
        info!("[卸载OpenClaw] 在登录 shell 中执行 npm uninstall（使用用户环境）");
        r#"echo "卸载 OpenClaw..."
npm uninstall -g @jerryan999/openclaw-zh

# 验证卸载
if command -v openclaw &> /dev/null; then
    echo "警告：openclaw 命令仍然存在"
    exit 1
else
    echo "OpenClaw 已成功卸载"
    exit 0
fi
"#
        .to_string()
    };

    // 在登录 shell 中执行，使 nvm/fnm 等与用户终端环境一致，避免用错 npm/node
    match shell::run_login_shell_output(&script) {
        Ok(output) => Ok(InstallResult {
            success: true,
            message: format!("OpenClaw 已成功卸载！{}", output),
            error: None,
        }),
        Err(e) => {
            warn!("[卸载OpenClaw] 卸载脚本执行失败: {}", e);
            Ok(InstallResult {
                success: false,
                message: "OpenClaw 卸载失败".to_string(),
                error: Some(e),
            })
        }
    }
}

/// 版本更新信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    /// 是否有更新可用
    pub update_available: bool,
    /// 当前版本
    pub current_version: Option<String>,
    /// 最新版本
    pub latest_version: Option<String>,
    /// 错误信息
    pub error: Option<String>,
}

/// 检查 OpenClaw 更新
#[command]
pub async fn check_openclaw_update() -> Result<UpdateInfo, String> {
    info!("[版本检查] 开始检查 OpenClaw 更新...");

    // 获取当前版本
    let current_version = get_openclaw_version();
    info!("[版本检查] 当前版本: {:?}", current_version);

    if current_version.is_none() {
        info!("[版本检查] OpenClaw 未安装");
        return Ok(UpdateInfo {
            update_available: false,
            current_version: None,
            latest_version: None,
            error: Some("OpenClaw 未安装".to_string()),
        });
    }

    // 获取最新版本
    let latest_version = get_latest_openclaw_version();
    info!("[版本检查] 最新版本: {:?}", latest_version);

    if latest_version.is_none() {
        return Ok(UpdateInfo {
            update_available: false,
            current_version,
            latest_version: None,
            error: Some("无法获取最新版本信息".to_string()),
        });
    }

    // 比较版本
    let current = current_version.clone().unwrap();
    let latest = latest_version.clone().unwrap();
    let update_available = compare_versions(&current, &latest);

    info!("[版本检查] 是否有更新: {}", update_available);

    Ok(UpdateInfo {
        update_available,
        current_version,
        latest_version,
        error: None,
    })
}

/// 获取 npm registry 上的最新版本
fn get_latest_openclaw_version() -> Option<String> {
    // 使用 npm view 获取最新版本
    let result = if platform::is_windows() {
        shell::run_cmd_output("npm view @jerryan999/openclaw-zh version")
    } else {
        shell::run_bash_output("npm view @jerryan999/openclaw-zh version 2>/dev/null")
    };

    match result {
        Ok(version) => {
            let v = version.trim().to_string();
            if v.is_empty() {
                None
            } else {
                Some(v)
            }
        }
        Err(e) => {
            warn!("[版本检查] 获取最新版本失败: {}", e);
            None
        }
    }
}

fn get_command_path(cmd: &str) -> Option<String> {
    if platform::is_windows() {
        let query = format!("where {}", cmd);
        if let Ok(output) = shell::run_cmd_output(&query) {
            return output
                .lines()
                .map(str::trim)
                .find(|line| !line.is_empty())
                .map(|line| line.to_string());
        }
        None
    } else {
        shell::run_command_output("which", &[cmd])
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    }
}

/// 比较版本号，返回是否有更新可用
/// current: 当前版本 (如 "1.0.0" 或 "v1.0.0")
/// latest: 最新版本 (如 "1.0.1")
fn compare_versions(current: &str, latest: &str) -> bool {
    let current_parts = extract_numeric_parts(current);
    let latest_parts = extract_numeric_parts(latest);
    let max_len = current_parts.len().max(latest_parts.len());

    for i in 0..max_len {
        let c = current_parts.get(i).unwrap_or(&0);
        let l = latest_parts.get(i).unwrap_or(&0);
        if l > c {
            return true;
        } else if l < c {
            return false;
        }
    }

    false
}

fn extract_numeric_parts(version: &str) -> Vec<u32> {
    let mut parts = Vec::new();
    let mut buf = String::new();
    let version = version.trim().trim_start_matches('v');
    for ch in version.chars() {
        if ch.is_ascii_digit() {
            buf.push(ch);
        } else if !buf.is_empty() {
            if let Ok(n) = buf.parse::<u32>() {
                parts.push(n);
            }
            buf.clear();
        }
    }
    if !buf.is_empty() {
        if let Ok(n) = buf.parse::<u32>() {
            parts.push(n);
        }
    }
    parts
}

fn openclaw_channel_file() -> std::path::PathBuf {
    if let Some(local) = dirs::data_local_dir() {
        let dir = local.join("OpenClawManager");
        let _ = std::fs::create_dir_all(&dir);
        return dir.join("openclaw-channel.txt");
    }
    if let Some(home) = dirs::home_dir() {
        return home.join(".openclaw-manager-channel.txt");
    }
    std::path::PathBuf::from("openclaw-channel.txt")
}

/// 获取 OpenClaw 安装渠道（latest / nightly）
#[command]
pub fn get_openclaw_channel() -> Result<String, String> {
    let path = openclaw_channel_file();
    match std::fs::read_to_string(&path) {
        Ok(s) => {
            let ch = s.trim().to_lowercase();
            if ch == "nightly" { Ok("nightly".to_string()) } else { Ok("latest".to_string()) }
        }
        _ => Ok("latest".to_string()),
    }
}

/// 设置 OpenClaw 安装渠道（latest / nightly）
#[command]
pub fn set_openclaw_channel(channel: String) -> Result<String, String> {
    let ch = channel.trim().to_lowercase();
    let valid = ch == "latest" || ch == "nightly";
    let path = openclaw_channel_file();
    if let Err(e) = std::fs::write(&path, if valid { ch.as_str() } else { "latest" }) {
        return Err(format!("写入渠道配置失败: {}", e));
    }
    Ok(if valid { ch } else { "latest".to_string() })
}

/// 更新 OpenClaw
#[command]
pub async fn update_openclaw() -> Result<InstallResult, String> {
    info!("[更新OpenClaw] 开始更新 OpenClaw...");
    let os = platform::get_os();

    // 先停止服务
    info!("[更新OpenClaw] 尝试停止服务...");
    let _ = shell::run_openclaw(&["gateway", "stop"]);
    std::thread::sleep(std::time::Duration::from_millis(500));

    let result = match os.as_str() {
        "windows" => {
            info!("[更新OpenClaw] 使用 Windows 更新方式...");
            update_openclaw_windows().await
        }
        _ => {
            info!("[更新OpenClaw] 使用 Unix 更新方式 (npm)...");
            update_openclaw_unix().await
        }
    };

    match &result {
        Ok(r) if r.success => info!("[更新OpenClaw] ✓ 更新成功"),
        Ok(r) => warn!("[更新OpenClaw] ✗ 更新失败: {}", r.message),
        Err(e) => error!("[更新OpenClaw] ✗ 更新错误: {}", e),
    }

    result
}

/// Windows 更新 OpenClaw
async fn update_openclaw_windows() -> Result<InstallResult, String> {
    info!("[更新OpenClaw] 执行 npm install -g @jerryan999/openclaw-zh...");

    match shell::run_cmd_output("npm install -g @jerryan999/openclaw-zh") {
        Ok(output) => {
            info!("[更新OpenClaw] npm 输出: {}", output);

            // 获取新版本
            let new_version = get_openclaw_version();

            Ok(InstallResult {
                success: true,
                message: format!(
                    "OpenClaw 已更新到 {}",
                    new_version.unwrap_or("最新版本".to_string())
                ),
                error: None,
            })
        }
        Err(e) => {
            warn!("[更新OpenClaw] npm install 失败: {}", e);
            Ok(InstallResult {
                success: false,
                message: "OpenClaw 更新失败".to_string(),
                error: Some(e),
            })
        }
    }
}

/// Unix 系统更新 OpenClaw
async fn update_openclaw_unix() -> Result<InstallResult, String> {
    let script = r#"
echo "更新 OpenClaw..."
npm install -g @jerryan999/openclaw-zh

# 验证更新
openclaw --version
"#;

    match shell::run_bash_output(script) {
        Ok(output) => Ok(InstallResult {
            success: true,
            message: format!("OpenClaw 已更新！{}", output),
            error: None,
        }),
        Err(e) => Ok(InstallResult {
            success: false,
            message: "OpenClaw 更新失败".to_string(),
            error: Some(e),
        }),
    }
}
