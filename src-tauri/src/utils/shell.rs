use crate::utils::file;
use crate::utils::platform;
use log::{debug, info, warn};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

/// Windows CREATE_NO_WINDOW 标志，用于隐藏控制台窗口
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

#[cfg(windows)]
const WINDOWS_RUNTIME_DIR: &str = "OpenClawManager\\runtime";

#[cfg(windows)]
const NODE_RESOURCE_RELATIVE_PATH: &str = "nodejs\\node-windows-x64.zip";

#[cfg(windows)]
const OPENCLAW_RESOURCE_RELATIVE_PATH: &str = "openclaw\\openclaw-zh.tgz";

#[cfg(windows)]
const GIT_RESOURCE_CANDIDATES: &[&str] = &[
    "git\\git-windows-x64.zip",
    "git\\git-portable.zip",
    "git\\PortableGit.zip",
];

#[derive(Debug, Clone)]
pub struct WindowsOfflineRuntime {
    pub node_dir: PathBuf,
    pub npm_prefix: PathBuf,
    pub openclaw_cmd: PathBuf,
    pub openclaw_package: PathBuf,
    pub git_exe: Option<PathBuf>,
}

/// 获取扩展的 PATH 环境变量
/// GUI 应用启动时可能没有继承用户 shell 的 PATH，需要手动添加常见路径
pub fn get_extended_path() -> String {
    let mut paths: Vec<String> = Vec::new();

    #[cfg(windows)]
    {
        if let Ok(runtime) = get_windows_offline_runtime() {
            paths.push(runtime.node_dir.display().to_string());
            paths.push(runtime.npm_prefix.display().to_string());
            if let Some(git_exe) = runtime.git_exe {
                if let Some(parent) = git_exe.parent() {
                    paths.push(parent.display().to_string());
                }
            }
        }
    }

    #[cfg(not(windows))]
    {
        // 添加常见的可执行文件路径
        paths.push("/opt/homebrew/bin".to_string()); // Homebrew on Apple Silicon
        paths.push("/usr/local/bin".to_string()); // Homebrew on Intel / 常规安装
        paths.push("/usr/bin".to_string());
        paths.push("/bin".to_string());

        if let Some(home) = dirs::home_dir() {
            let home_str = home.display().to_string();

            // nvm 路径（尝试获取当前版本）
            let nvm_default = format!("{}/.nvm/alias/default", home_str);
            if let Ok(version) = std::fs::read_to_string(&nvm_default) {
                let version = version.trim();
                if !version.is_empty() {
                    paths.insert(
                        0,
                        format!("{}/.nvm/versions/node/v{}/bin", home_str, version),
                    );
                }
            }
            // 也添加常见 nvm 版本路径
            for version in ["v22.22.0", "v22.12.0", "v22.11.0", "v22.0.0", "v23.0.0"] {
                let nvm_bin = format!("{}/.nvm/versions/node/{}/bin", home_str, version);
                if std::path::Path::new(&nvm_bin).exists() {
                    paths.insert(0, nvm_bin);
                    break; // 只添加第一个存在的
                }
            }

            // fnm
            paths.push(format!("{}/.fnm/aliases/default/bin", home_str));

            // volta
            paths.push(format!("{}/.volta/bin", home_str));

            // asdf
            paths.push(format!("{}/.asdf/shims", home_str));

            // mise
            paths.push(format!("{}/.local/share/mise/shims", home_str));
        }
    }

    let current_path = std::env::var("PATH").unwrap_or_default();
    if !current_path.is_empty() {
        paths.push(current_path);
    }

    #[cfg(windows)]
    let path_sep = ";";
    #[cfg(not(windows))]
    let path_sep = ":";

    paths.join(path_sep)
}

#[cfg(windows)]
fn zip_err_to_io(err: zip::result::ZipError) -> io::Error {
    io::Error::new(io::ErrorKind::Other, err.to_string())
}

#[cfg(windows)]
fn get_windows_runtime_root() -> PathBuf {
    if let Some(local) = dirs::data_local_dir() {
        return local.join(WINDOWS_RUNTIME_DIR);
    }

    if let Some(home) = dirs::home_dir() {
        return home.join("AppData\\Local").join(WINDOWS_RUNTIME_DIR);
    }

    PathBuf::from("C:\\OpenClawManager\\runtime")
}

#[cfg(windows)]
fn get_windows_resource_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            roots.push(exe_dir.join("resources"));
            roots.push(exe_dir.join("..").join("resources"));
        }
    }
    roots
}

#[cfg(windows)]
fn find_windows_resource_file(relative: &str) -> Option<PathBuf> {
    for root in get_windows_resource_roots() {
        let path = root.join(relative);
        if path.exists() {
            return Some(path);
        }
    }
    None
}

#[cfg(windows)]
fn extract_zip(zip_path: &Path, output_dir: &Path) -> io::Result<()> {
    fs::create_dir_all(output_dir)?;
    let file = fs::File::open(zip_path)?;
    let mut archive = zip::ZipArchive::new(file).map_err(zip_err_to_io)?;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(zip_err_to_io)?;
        let out_path = output_dir.join(entry.mangled_name());
        if entry.name().ends_with('/') {
            fs::create_dir_all(&out_path)?;
            continue;
        }

        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut out = fs::File::create(&out_path)?;
        io::copy(&mut entry, &mut out)?;
    }
    Ok(())
}

#[cfg(windows)]
fn find_parent_dir_with_file(root: &Path, file_name: &str) -> Option<PathBuf> {
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let target_file = dir.join(file_name);
        if target_file.exists() {
            return Some(dir);
        }

        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_dir() {
                        stack.push(entry.path());
                    }
                }
            }
        }
    }
    None
}

#[cfg(windows)]
fn find_top_level_dir_containing_file(root: &Path, file_name: &str) -> Option<PathBuf> {
    if root.join(file_name).exists() {
        return Some(root.to_path_buf());
    }

    let entries = fs::read_dir(root).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if !entry.file_type().ok()?.is_dir() {
            continue;
        }
        if find_parent_dir_with_file(&path, file_name).is_some() {
            return Some(path);
        }
    }
    None
}

#[cfg(windows)]
fn copy_dir_recursive(src: &Path, dst: &Path) -> io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

#[cfg(windows)]
fn move_or_copy_dir(src: &Path, dst: &Path) -> io::Result<()> {
    if dst.exists() {
        fs::remove_dir_all(dst)?;
    }
    match fs::rename(src, dst) {
        Ok(_) => Ok(()),
        Err(_) => {
            copy_dir_recursive(src, dst)?;
            fs::remove_dir_all(src)?;
            Ok(())
        }
    }
}

#[cfg(windows)]
fn ensure_windows_node_runtime(runtime_root: &Path) -> io::Result<PathBuf> {
    let node_dir = runtime_root.join("node");
    if node_dir.join("node.exe").exists() && node_dir.join("npm.cmd").exists() {
        return Ok(node_dir);
    }

    let node_zip = find_windows_resource_file(NODE_RESOURCE_RELATIVE_PATH).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "找不到内置 Node.js 资源: resources/nodejs/node-windows-x64.zip",
        )
    })?;

    let extract_root = runtime_root.join("tmp-node-extract");
    if extract_root.exists() {
        fs::remove_dir_all(&extract_root)?;
    }
    extract_zip(&node_zip, &extract_root)?;

    let extracted_node_dir = find_top_level_dir_containing_file(&extract_root, "node.exe")
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Node.js 压缩包中未找到 node.exe"))?;
    move_or_copy_dir(&extracted_node_dir, &node_dir)?;

    if extract_root.exists() {
        let _ = fs::remove_dir_all(extract_root);
    }

    Ok(node_dir)
}

#[cfg(windows)]
fn ensure_windows_openclaw_package(runtime_root: &Path) -> io::Result<PathBuf> {
    let packages_dir = runtime_root.join("packages");
    fs::create_dir_all(&packages_dir)?;

    let target = packages_dir.join("openclaw-zh.tgz");
    if target.exists() {
        return Ok(target);
    }

    let source = find_windows_resource_file(OPENCLAW_RESOURCE_RELATIVE_PATH).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "找不到内置 OpenClaw 资源: resources/openclaw/openclaw-zh.tgz",
        )
    })?;
    fs::copy(source, &target)?;
    Ok(target)
}

#[cfg(windows)]
fn find_git_resource_zip() -> Option<PathBuf> {
    for rel in GIT_RESOURCE_CANDIDATES {
        if let Some(path) = find_windows_resource_file(rel) {
            return Some(path);
        }
    }
    None
}

#[cfg(windows)]
fn find_git_executable(root: &Path) -> Option<PathBuf> {
    find_parent_dir_with_file(root, "git.exe").map(|p| p.join("git.exe"))
}

#[cfg(windows)]
fn ensure_windows_git_runtime(runtime_root: &Path) -> io::Result<Option<PathBuf>> {
    let git_root = runtime_root.join("git");
    if let Some(exe) = find_git_executable(&git_root) {
        return Ok(Some(exe));
    }

    let Some(git_zip) = find_git_resource_zip() else {
        return Ok(None);
    };

    let extract_root = runtime_root.join("tmp-git-extract");
    if extract_root.exists() {
        fs::remove_dir_all(&extract_root)?;
    }
    extract_zip(&git_zip, &extract_root)?;

    let extracted_git_root = find_top_level_dir_containing_file(&extract_root, "git.exe")
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Git 压缩包中未找到 git.exe"))?;
    move_or_copy_dir(&extracted_git_root, &git_root)?;

    if extract_root.exists() {
        let _ = fs::remove_dir_all(extract_root);
    }

    Ok(find_git_executable(&git_root))
}

#[cfg(windows)]
pub fn get_windows_offline_runtime() -> Result<WindowsOfflineRuntime, String> {
    let runtime_root = get_windows_runtime_root();
    fs::create_dir_all(&runtime_root).map_err(|e| format!("创建离线运行时目录失败: {}", e))?;

    let node_dir = ensure_windows_node_runtime(&runtime_root)
        .map_err(|e| format!("准备离线 Node.js 失败: {}", e))?;
    let openclaw_package = ensure_windows_openclaw_package(&runtime_root)
        .map_err(|e| format!("准备离线 OpenClaw 包失败: {}", e))?;

    let npm_prefix = runtime_root.join("npm-global");
    fs::create_dir_all(&npm_prefix).map_err(|e| format!("创建 npm 全局目录失败: {}", e))?;

    let git_exe = ensure_windows_git_runtime(&runtime_root)
        .map_err(|e| format!("准备离线 Git 失败: {}", e))?;

    let openclaw_cmd = npm_prefix.join("openclaw.cmd");
    Ok(WindowsOfflineRuntime {
        node_dir,
        npm_prefix,
        openclaw_cmd,
        openclaw_package,
        git_exe,
    })
}

/// 执行 Shell 命令（带扩展 PATH）
pub fn run_command(cmd: &str, args: &[&str]) -> io::Result<Output> {
    let mut command = Command::new(cmd);
    command.args(args);
    command.env("PATH", get_extended_path());

    #[cfg(windows)]
    command.creation_flags(CREATE_NO_WINDOW);

    command.output()
}

/// 执行 Shell 命令并获取输出字符串
pub fn run_command_output(cmd: &str, args: &[&str]) -> Result<String, String> {
    match run_command(cmd, args) {
        Ok(output) => {
            if output.status.success() {
                Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
            } else {
                Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
            }
        }
        Err(e) => Err(e.to_string()),
    }
}

/// 执行 Bash 命令（带扩展 PATH）
pub fn run_bash(script: &str) -> io::Result<Output> {
    let mut command = Command::new("bash");
    command.arg("-c").arg(script);

    // 在非 Windows 系统上使用扩展的 PATH
    #[cfg(not(windows))]
    {
        let extended_path = get_extended_path();
        command.env("PATH", extended_path);
    }

    #[cfg(windows)]
    command.creation_flags(CREATE_NO_WINDOW);

    command.output()
}

/// 执行 Bash 命令并获取输出
pub fn run_bash_output(script: &str) -> Result<String, String> {
    match run_bash(script) {
        Ok(output) => {
            if output.status.success() {
                Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                if stderr.is_empty() {
                    Err(format!(
                        "Command failed with exit code: {:?}",
                        output.status.code()
                    ))
                } else {
                    Err(stderr)
                }
            }
        }
        Err(e) => Err(e.to_string()),
    }
}

/// 执行 cmd.exe 命令（Windows）- 避免 PowerShell 执行策略问题
pub fn run_cmd(script: &str) -> io::Result<Output> {
    let mut cmd = Command::new("cmd");
    cmd.args(["/c", script]);
    cmd.env("PATH", get_extended_path());

    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);

    cmd.output()
}

/// 执行 cmd.exe 命令并获取输出（Windows）
pub fn run_cmd_output(script: &str) -> Result<String, String> {
    match run_cmd(script) {
        Ok(output) => {
            if output.status.success() {
                Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                if stderr.is_empty() {
                    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if stdout.is_empty() {
                        Err(format!(
                            "Command failed with exit code: {:?}",
                            output.status.code()
                        ))
                    } else {
                        Err(stdout)
                    }
                } else {
                    Err(stderr)
                }
            }
        }
        Err(e) => Err(e.to_string()),
    }
}

/// 执行 PowerShell 命令（Windows）- 仅在需要 PowerShell 特定功能时使用
/// 注意：某些 Windows 系统的 PowerShell 执行策略可能禁止运行脚本
pub fn run_powershell(script: &str) -> io::Result<Output> {
    let mut cmd = Command::new("powershell");
    // 使用 -ExecutionPolicy Bypass 绕过执行策略限制
    cmd.args([
        "-NoProfile",
        "-NonInteractive",
        "-ExecutionPolicy",
        "Bypass",
        "-Command",
        script,
    ]);
    cmd.env("PATH", get_extended_path());

    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);

    cmd.output()
}

/// 执行 PowerShell 命令并获取输出（Windows）
pub fn run_powershell_output(script: &str) -> Result<String, String> {
    match run_powershell(script) {
        Ok(output) => {
            if output.status.success() {
                Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                if stderr.is_empty() {
                    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if stdout.is_empty() {
                        Err(format!(
                            "Command failed with exit code: {:?}",
                            output.status.code()
                        ))
                    } else {
                        Err(stdout)
                    }
                } else {
                    Err(stderr)
                }
            }
        }
        Err(e) => Err(e.to_string()),
    }
}

/// 跨平台执行脚本命令
/// Windows 上使用 cmd.exe（避免 PowerShell 执行策略问题）
pub fn run_script_output(script: &str) -> Result<String, String> {
    if platform::is_windows() {
        run_cmd_output(script)
    } else {
        run_bash_output(script)
    }
}

/// 后台执行命令（不等待结果）
pub fn spawn_background(script: &str) -> io::Result<()> {
    if platform::is_windows() {
        let mut cmd = Command::new("cmd");
        cmd.args(["/c", script]);
        cmd.env("PATH", get_extended_path());

        #[cfg(windows)]
        cmd.creation_flags(CREATE_NO_WINDOW);

        cmd.spawn()?;
    } else {
        Command::new("bash").arg("-c").arg(script).spawn()?;
    }
    Ok(())
}

/// 获取 openclaw 可执行文件路径
/// 检测多个可能的安装路径，因为 GUI 应用不继承用户 shell 的 PATH
pub fn get_openclaw_path() -> Option<String> {
    // Windows: 检查常见的 npm 全局安装路径
    if platform::is_windows() {
        #[cfg(windows)]
        if let Ok(runtime) = get_windows_offline_runtime() {
            if runtime.openclaw_cmd.exists() {
                info!(
                    "[Shell] 使用离线 runtime 中的 openclaw: {}",
                    runtime.openclaw_cmd.display()
                );
                return Some(runtime.openclaw_cmd.display().to_string());
            }
        }

        let possible_paths = get_windows_openclaw_paths();
        for path in possible_paths {
            if std::path::Path::new(&path).exists() {
                info!("[Shell] 在 {} 找到 openclaw", path);
                return Some(path);
            }
        }
    } else {
        // Unix: 检查常见的 npm 全局安装路径
        let possible_paths = get_unix_openclaw_paths();
        for path in possible_paths {
            if std::path::Path::new(&path).exists() {
                info!("[Shell] 在 {} 找到 openclaw", path);
                return Some(path);
            }
        }
    }

    // 回退：检查是否在 PATH 中
    if command_exists("openclaw") {
        return Some("openclaw".to_string());
    }

    // 最后尝试：通过用户 shell 查找
    if !platform::is_windows() {
        if let Ok(path) = run_bash_output("source ~/.zshrc 2>/dev/null || source ~/.bashrc 2>/dev/null; which openclaw 2>/dev/null") {
            if !path.is_empty() && std::path::Path::new(&path).exists() {
                info!("[Shell] 通过用户 shell 找到 openclaw: {}", path);
                return Some(path);
            }
        }
    }

    None
}

/// 获取 Unix 系统上可能的 openclaw 安装路径
fn get_unix_openclaw_paths() -> Vec<String> {
    let mut paths = Vec::new();

    // npm 全局安装路径
    paths.push("/usr/local/bin/openclaw".to_string());
    paths.push("/opt/homebrew/bin/openclaw".to_string()); // Homebrew on Apple Silicon
    paths.push("/usr/bin/openclaw".to_string());

    if let Some(home) = dirs::home_dir() {
        let home_str = home.display().to_string();

        // npm 全局安装到用户目录
        paths.push(format!("{}/.npm-global/bin/openclaw", home_str));

        // nvm 安装的 npm 全局包（需要找到正确的 node 版本目录）
        // 先检查常见版本
        for version in [
            "v22.0.0", "v22.1.0", "v22.2.0", "v22.11.0", "v22.12.0", "v23.0.0",
        ] {
            paths.push(format!(
                "{}/.nvm/versions/node/{}/bin/openclaw",
                home_str, version
            ));
        }

        // 检查 nvm current（尝试读取 .nvmrc 或 default）
        let nvm_default = format!("{}/.nvm/alias/default", home_str);
        if let Ok(version) = std::fs::read_to_string(&nvm_default) {
            let version = version.trim();
            if !version.is_empty() {
                paths.insert(
                    0,
                    format!("{}/.nvm/versions/node/v{}/bin/openclaw", home_str, version),
                );
            }
        }

        // fnm
        paths.push(format!("{}/.fnm/aliases/default/bin/openclaw", home_str));

        // volta
        paths.push(format!("{}/.volta/bin/openclaw", home_str));

        // pnpm 全局安装
        paths.push(format!("{}/.pnpm/bin/openclaw", home_str));
        paths.push(format!("{}/Library/pnpm/openclaw", home_str)); // macOS pnpm 默认路径

        // asdf
        paths.push(format!("{}/.asdf/shims/openclaw", home_str));

        // mise (formerly rtx)
        paths.push(format!("{}/.local/share/mise/shims/openclaw", home_str));

        // yarn 全局安装
        paths.push(format!("{}/.yarn/bin/openclaw", home_str));
        paths.push(format!(
            "{}/.config/yarn/global/node_modules/.bin/openclaw",
            home_str
        ));
    }

    paths
}

/// 获取 Windows 上可能的 openclaw 安装路径
fn get_windows_openclaw_paths() -> Vec<String> {
    let mut paths = Vec::new();

    // 1. nvm4w 安装路径
    paths.push("C:\\nvm4w\\nodejs\\openclaw.cmd".to_string());

    // 2. 用户目录下的 npm 全局路径
    if let Some(home) = dirs::home_dir() {
        let npm_path = format!("{}\\AppData\\Roaming\\npm\\openclaw.cmd", home.display());
        paths.push(npm_path);
    }

    // 3. Program Files 下的 nodejs
    paths.push("C:\\Program Files\\nodejs\\openclaw.cmd".to_string());

    paths
}

/// openclaw agent 可能要求的工作区模板（若再报 Missing workspace template: XXX.md，在此追加 "XXX.md"）
const AGENT_TEST_TEMPLATES: &[&str] = &[
    "AGENTS.md",
    "SOUL.md",
    "TOOLS.md",
    "IDENTITY.md",
    "USER.md",
    "HEARTBEAT.md",
    "BOOTSTRAP.md",
];

/// 创建用于 agent 测试的临时工作区（包含 docs/reference/templates 下所需模板），
/// 避免在 Manager 项目目录下执行 agent 时报 Missing workspace template。
pub fn create_agent_test_workspace() -> Result<PathBuf, String> {
    let name = format!(
        "openclaw-manager-agent-test-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0)
    );
    let root = std::env::temp_dir().join(name);
    let templates_dir = root.join("docs").join("reference").join("templates");
    fs::create_dir_all(&templates_dir).map_err(|e| format!("创建临时工作区失败: {}", e))?;

    let placeholder = b"# Placeholder\nMinimal template for OpenClaw Manager AI connection test.\n";
    for name in AGENT_TEST_TEMPLATES {
        fs::write(templates_dir.join(name), placeholder)
            .map_err(|e| format!("写入 {} 失败: {}", name, e))?;
    }

    debug!(
        "[Shell] 临时 agent 工作区: {} (templates: {:?})",
        root.display(),
        AGENT_TEST_TEMPLATES
    );
    Ok(root)
}

/// 在指定工作目录下执行 openclaw 命令（用于 agent 测试，避免 Missing workspace template）
pub fn run_openclaw_with_cwd(args: &[&str], cwd: &Path) -> Result<String, String> {
    debug!(
        "[Shell] 执行 openclaw 命令 (cwd={}): {:?}",
        cwd.display(),
        args
    );

    let openclaw_path = get_openclaw_path().ok_or_else(|| {
        warn!("[Shell] 找不到 openclaw 命令");
        "找不到 openclaw 命令，请确保已通过 npm install -g @jerryan999/openclaw-zh 安装".to_string()
    })?;

    let extended_path = get_extended_path();

    let output = if openclaw_path.ends_with(".cmd") {
        let mut cmd_args = vec!["/c", &openclaw_path];
        cmd_args.extend(args);
        let mut cmd = Command::new("cmd");
        cmd.args(&cmd_args)
            .current_dir(cwd)
            .env("OPENCLAW_GATEWAY_TOKEN", DEFAULT_GATEWAY_TOKEN)
            .env("PATH", &extended_path);
        #[cfg(windows)]
        cmd.creation_flags(CREATE_NO_WINDOW);
        cmd.output()
    } else {
        let mut cmd = Command::new(&openclaw_path);
        cmd.args(args)
            .current_dir(cwd)
            .env("OPENCLAW_GATEWAY_TOKEN", DEFAULT_GATEWAY_TOKEN)
            .env("PATH", &extended_path);
        #[cfg(windows)]
        cmd.creation_flags(CREATE_NO_WINDOW);
        cmd.output()
    };

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout).to_string();
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();
            if out.status.success() {
                Ok(stdout)
            } else {
                Err(format!("{}\n{}", stdout, stderr).trim().to_string())
            }
        }
        Err(e) => Err(format!("执行 openclaw 失败: {}", e)),
    }
}

/// 执行 openclaw 命令并获取输出
pub fn run_openclaw(args: &[&str]) -> Result<String, String> {
    debug!("[Shell] 执行 openclaw 命令: {:?}", args);

    let openclaw_path = get_openclaw_path().ok_or_else(|| {
        warn!("[Shell] 找不到 openclaw 命令");
        "找不到 openclaw 命令，请确保已通过 npm install -g @jerryan999/openclaw-zh 安装".to_string()
    })?;

    debug!("[Shell] openclaw 路径: {}", openclaw_path);

    // 获取扩展的 PATH，确保能找到 node
    let extended_path = get_extended_path();
    debug!("[Shell] 扩展 PATH: {}", extended_path);

    let output = if openclaw_path.ends_with(".cmd") {
        // Windows: .cmd 文件需要通过 cmd /c 执行
        let mut cmd_args = vec!["/c", &openclaw_path];
        cmd_args.extend(args);
        let mut cmd = Command::new("cmd");
        cmd.args(&cmd_args)
            .env("OPENCLAW_GATEWAY_TOKEN", DEFAULT_GATEWAY_TOKEN)
            .env("PATH", &extended_path);

        #[cfg(windows)]
        cmd.creation_flags(CREATE_NO_WINDOW);

        cmd.output()
    } else {
        let mut cmd = Command::new(&openclaw_path);
        cmd.args(args)
            .env("OPENCLAW_GATEWAY_TOKEN", DEFAULT_GATEWAY_TOKEN)
            .env("PATH", &extended_path);

        #[cfg(windows)]
        cmd.creation_flags(CREATE_NO_WINDOW);

        cmd.output()
    };

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout).to_string();
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();
            debug!("[Shell] 命令退出码: {:?}", out.status.code());
            if out.status.success() {
                debug!("[Shell] 命令执行成功, stdout 长度: {}", stdout.len());
                Ok(stdout)
            } else {
                debug!("[Shell] 命令执行失败, stderr: {}", stderr);
                Err(format!("{}\n{}", stdout, stderr).trim().to_string())
            }
        }
        Err(e) => {
            warn!("[Shell] 执行 openclaw 失败: {}", e);
            Err(format!("执行 openclaw 失败: {}", e))
        }
    }
}

/// 默认的 Gateway Token
pub const DEFAULT_GATEWAY_TOKEN: &str = "openclaw-manager-local-token";

/// 从 ~/.openclaw/env 文件读取所有环境变量
/// 与 shell 脚本 `source ~/.openclaw/env` 行为一致
fn load_openclaw_env_vars() -> HashMap<String, String> {
    let mut env_vars = HashMap::new();
    let env_path = platform::get_env_file_path();

    if let Ok(content) = file::read_file(&env_path) {
        for line in content.lines() {
            let line = line.trim();
            // 跳过注释和空行
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            // 解析 export KEY=VALUE 或 KEY=VALUE 格式
            let line = line.strip_prefix("export ").unwrap_or(line);
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                // 去除值周围的引号
                let value = value.trim().trim_matches('"').trim_matches('\'');
                env_vars.insert(key.to_string(), value.to_string());
            }
        }
    }

    env_vars
}

/// 后台启动 openclaw gateway
/// 与 shell 脚本行为一致：先加载 env 文件，再启动 gateway
pub fn spawn_openclaw_gateway() -> io::Result<()> {
    info!("[Shell] 后台启动 openclaw gateway...");

    let openclaw_path = get_openclaw_path().ok_or_else(|| {
        warn!("[Shell] 找不到 openclaw 命令");
        io::Error::new(
            io::ErrorKind::NotFound,
            "找不到 openclaw 命令，请确保已通过 npm install -g @jerryan999/openclaw-zh 安装",
        )
    })?;

    info!("[Shell] openclaw 路径: {}", openclaw_path);

    // 加载用户的 env 文件环境变量（与 shell 脚本 source ~/.openclaw/env 一致）
    info!("[Shell] 加载用户环境变量...");
    let user_env_vars = load_openclaw_env_vars();
    info!("[Shell] 已加载 {} 个环境变量", user_env_vars.len());
    for key in user_env_vars.keys() {
        debug!("[Shell] - 环境变量: {}", key);
    }

    // 获取扩展的 PATH，确保能找到 node
    let extended_path = get_extended_path();
    info!("[Shell] 扩展 PATH: {}", extended_path);

    // Windows 上 .cmd 文件需要通过 cmd /c 来执行
    // 设置环境变量 OPENCLAW_GATEWAY_TOKEN，这样所有子命令都能自动使用
    let mut cmd = if openclaw_path.ends_with(".cmd") {
        info!("[Shell] Windows 模式: 使用 cmd /c 执行");
        let mut c = Command::new("cmd");
        c.args(["/c", &openclaw_path, "gateway", "--port", "18789"]);
        c
    } else {
        info!("[Shell] Unix 模式: 直接执行");
        let mut c = Command::new(&openclaw_path);
        c.args(["gateway", "--port", "18789"]);
        c
    };

    // 注入用户的环境变量（如 ANTHROPIC_API_KEY, OPENAI_API_KEY 等）
    for (key, value) in &user_env_vars {
        cmd.env(key, value);
    }

    // 设置 PATH 和 gateway token
    cmd.env("PATH", &extended_path);
    cmd.env("OPENCLAW_GATEWAY_TOKEN", DEFAULT_GATEWAY_TOKEN);

    // Windows: 隐藏控制台窗口
    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);

    info!("[Shell] 启动 gateway 进程...");
    let child = cmd.spawn();

    match child {
        Ok(c) => {
            info!("[Shell] ✓ Gateway 进程已启动, PID: {}", c.id());
            Ok(())
        }
        Err(e) => {
            warn!("[Shell] ✗ Gateway 启动失败: {}", e);
            Err(io::Error::new(
                e.kind(),
                format!("启动失败 (路径: {}): {}", openclaw_path, e),
            ))
        }
    }
}

/// 检查命令是否存在
pub fn command_exists(cmd: &str) -> bool {
    if platform::is_windows() {
        // Windows: 使用 where 命令
        let mut command = Command::new("where");
        command.arg(cmd);
        command.env("PATH", get_extended_path());

        #[cfg(windows)]
        command.creation_flags(CREATE_NO_WINDOW);

        command
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    } else {
        // Unix: 使用 which 命令
        let mut command = Command::new("which");
        command.arg(cmd);
        command.env("PATH", get_extended_path());
        command
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}
