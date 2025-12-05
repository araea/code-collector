use chrono::Local;
use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};

// ═══════════════════════════════════════════════════════════════════════════
// 常量定义
// ═══════════════════════════════════════════════════════════════════════════

const VERSION: &str = "0.1.1";

const BINARY_EXTS: &[&str] = &[
    "exe", "dll", "so", "dylib", "png", "jpg", "jpeg", "gif", "bmp", "pdf", "zip", "tar", "gz",
    "7z", "mp3", "mp4", "mov", "avi", "mkv", "doc", "docx", "xls", "xlsx", "ppt", "pptx", "bin",
    "o", "a", "lib", "class", "jar", "war", "ear", "swf", "dat", "db", "sqlite", "db3", "dbf",
    "mdb", "accdb", "ttf", "otf", "woff", "woff2", "eot", "ico", "cur", "svgz", "psd", "ai", "eps",
    "ps", "tiff", "tif", "raw", "cr2", "nef", "orf", "sr2", "dng", "heic", "heif", "webp", "avif",
    "apng", "xcf", "kra", "blend", "max", "c4d", "ma", "mb", "fbx", "obj", "stl", "gcode", "dwg",
    "dxf", "step", "stp", "iges", "igs", "3dm", "skp", "rvt", "ifc", "dwf", "dwfx", "vsd", "vdx",
    "vsdx", "vsdm", "vss", "vssm", "vst", "vstm", "vtx", "emf", "wmf", "pcx", "tga", "ppm", "pgm",
    "pbm", "pnm", "hdr", "exr", "lock", "sum",
];

// 默认忽略的目录名
const IGNORED_DIRS: &[&str] = &[
    ".git",
    ".svn",
    ".hg",
    "node_modules",
    "target",
    "build",
    "dist",
    "out",
    "__pycache__",
    ".idea",
    ".vscode",
    "vendor",
    ".cache",
    "coverage",
];

// ═══════════════════════════════════════════════════════════════════════════
// 配置结构
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Clone)]
struct Config {
    path: PathBuf,
    outfile: PathBuf,
    max_bytes: u64,
    skip_exts: HashSet<String>,
    include_exts: Option<HashSet<String>>, // 白名单过滤
    ignore_dirs: HashSet<String>,          // 忽略的目录名
    ignore_files: HashSet<String>,         // 忽略的文件名
    show_tree: bool,
    show_toc: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            path: PathBuf::new(),
            outfile: PathBuf::from("all-in-one.md"),
            max_bytes: 1024 * 1024, // 1MB
            skip_exts: HashSet::new(),
            include_exts: None,
            ignore_dirs: HashSet::new(),
            ignore_files: HashSet::new(),
            show_tree: true,
            show_toc: true,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 统计信息
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Default)]
struct Stats {
    files_processed: usize,
    files_skipped_binary: usize,
    files_skipped_size: usize,
    files_skipped_encoding: usize,
    files_skipped_filter: usize,
    total_bytes: u64,
    total_lines: usize,
    dirs_count: usize,
    file_types: HashMap<String, usize>,
}

// ═══════════════════════════════════════════════════════════════════════════
// 文件条目（用于目录树和 TOC）
// ═══════════════════════════════════════════════════════════════════════════

struct FileEntry {
    relative_path: String,
    extension: String,
    size: u64,
    line_count: usize,
}

// ═══════════════════════════════════════════════════════════════════════════
// 扩展名到语言映射
// ═══════════════════════════════════════════════════════════════════════════

fn get_language(ext: &str) -> &'static str {
    match ext {
        "rs" => "rust",
        "py" | "pyw" | "pyi" => "python",
        "js" | "mjs" | "cjs" => "javascript",
        "ts" | "mts" | "cts" => "typescript",
        "tsx" => "tsx",
        "jsx" => "jsx",
        "html" | "htm" => "html",
        "css" | "scss" | "sass" | "less" => "css",
        "md" | "markdown" => "markdown",
        "json" | "jsonc" => "json",
        "xml" | "svg" | "xsl" | "xslt" => "xml",
        "yml" | "yaml" => "yaml",
        "toml" => "toml",
        "ini" | "cfg" | "conf" => "ini",
        "sh" | "bash" | "zsh" => "bash",
        "bat" | "cmd" => "batch",
        "ps1" | "psm1" | "psd1" => "powershell",
        "java" => "java",
        "c" | "h" => "c",
        "cpp" | "cc" | "cxx" | "hpp" | "hxx" | "hh" => "cpp",
        "cs" => "csharp",
        "go" => "go",
        "rb" | "erb" => "ruby",
        "php" => "php",
        "swift" => "swift",
        "kt" | "kts" => "kotlin",
        "scala" | "sc" => "scala",
        "groovy" | "gradle" => "groovy",
        "clj" | "cljs" | "cljc" | "edn" => "clojure",
        "lua" => "lua",
        "pl" | "pm" => "perl",
        "r" | "R" => "r",
        "sql" => "sql",
        "dart" => "dart",
        "vue" => "vue",
        "svelte" => "svelte",
        "elm" => "elm",
        "ex" | "exs" => "elixir",
        "erl" | "hrl" => "erlang",
        "hs" | "lhs" => "haskell",
        "ml" | "mli" => "ocaml",
        "fs" | "fsi" | "fsx" => "fsharp",
        "v" => "verilog",
        "vhd" | "vhdl" => "vhdl",
        "zig" => "zig",
        "nim" => "nim",
        "dockerfile" => "dockerfile",
        "makefile" | "mk" => "makefile",
        "cmake" => "cmake",
        "tf" | "tfvars" => "hcl",
        "proto" => "protobuf",
        "graphql" | "gql" => "graphql",
        _ => "plaintext",
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 辅助函数
// ═══════════════════════════════════════════════════════════════════════════

/// 计算内容中最长的连续反引号数量
fn count_max_backticks(content: &str) -> usize {
    let mut max_count = 0;
    let mut current_count = 0;

    for ch in content.chars() {
        if ch == '`' {
            current_count += 1;
            max_count = max_count.max(current_count);
        } else {
            current_count = 0;
        }
    }

    max_count
}

fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

fn print_colored(color: &str, symbol: &str, message: &str) {
    let color_code = match color {
        "green" => "\x1b[32m",
        "yellow" => "\x1b[33m",
        "red" => "\x1b[31m",
        "blue" => "\x1b[34m",
        "cyan" => "\x1b[36m",
        "gray" => "\x1b[90m",
        _ => "\x1b[0m",
    };
    println!("{}{}\x1b[0m {}", color_code, symbol, message);
}

fn print_progress(current: usize, message: &str) {
    print!("\r\x1b[K\x1b[36m[{:>4}]\x1b[0m {}", current, message);
    io::stdout().flush().ok();
}

// ═══════════════════════════════════════════════════════════════════════════
// 帮助信息
// ═══════════════════════════════════════════════════════════════════════════

fn print_help() {
    println!(
        r#"
📦 Code Collector v{}
将项目代码整合为单个 Markdown 文件，便于 AI 分析

用法:
  code_collector                           # 交互模式
  code_collector -Path <目录> [选项]       # 命令行模式

选项:
  -Path <路径>         目标目录路径（必需）
  -OutFile <文件>      输出文件名（默认: all-in-one.md）
  -MaxBytes <大小>     最大文件大小（默认: 1048576 = 1MB）
  -SkipExts <扩展名>   跳过的扩展名（空格分隔）
  -IncludeExts <扩展名> 只包含的扩展名（空格分隔，白名单模式）
  -IgnoreDirs <名称>   忽略的特定目录名（空格分隔，如: tests docs）
  -IgnoreFiles <名称>  忽略的特定文件名（空格分隔，如: package-lock.json）
  -NoTree              不生成目录树
  -NoToc               不生成目录索引
  -h, --help           显示帮助信息

示例:
  code_collector -Path ./my_project
  code_collector -Path ./src -OutFile code.md -MaxBytes 512000
  code_collector -Path . -IncludeExts "rs toml md"
  code_collector -Path ./project -IgnoreDirs "tests examples" -IgnoreFiles "cargo.lock"
"#,
        VERSION
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// 交互式输入
// ═══════════════════════════════════════════════════════════════════════════

fn interactive_input() -> Result<Config, Box<dyn std::error::Error>> {
    println!("\n\x1b[36m╔══════════════════════════════════════════╗\x1b[0m");
    println!(
        "\x1b[36m║\x1b[0m   📦 Code Collector v{}                \x1b[36m║\x1b[0m",
        VERSION
    );
    println!("\x1b[36m╚══════════════════════════════════════════╝\x1b[0m\n");

    let mut config = Config::default();

    // 输入目录路径
    print!("\x1b[33m?\x1b[0m 请输入目标目录路径: ");
    io::stdout().flush()?;
    let mut path = String::new();
    io::stdin().read_line(&mut path)?;
    config.path = PathBuf::from(path.trim());

    // 是否使用默认选项
    print!("\x1b[33m?\x1b[0m 使用默认选项？[Y/n]: ");
    io::stdout().flush()?;
    let mut use_default = String::new();
    io::stdin().read_line(&mut use_default)?;
    let use_default = use_default.trim().to_lowercase();

    if !use_default.is_empty() && use_default != "y" && use_default != "yes" {
        // 输出文件名
        print!("\x1b[33m?\x1b[0m 输出文件名 [all-in-one.md]: ");
        io::stdout().flush()?;
        let mut outfile = String::new();
        io::stdin().read_line(&mut outfile)?;
        if !outfile.trim().is_empty() {
            config.outfile = PathBuf::from(outfile.trim());
        }

        // 最大文件大小
        print!("\x1b[33m?\x1b[0m 最大文件大小（字节）[1048576]: ");
        io::stdout().flush()?;
        let mut max_bytes = String::new();
        io::stdin().read_line(&mut max_bytes)?;
        if !max_bytes.trim().is_empty() {
            config.max_bytes = max_bytes.trim().parse()?;
        }

        // 只包含的扩展名
        print!("\x1b[33m?\x1b[0m 只包含的扩展名（空格分隔，留空表示全部）: ");
        io::stdout().flush()?;
        let mut include_exts = String::new();
        io::stdin().read_line(&mut include_exts)?;
        if !include_exts.trim().is_empty() {
            config.include_exts = Some(
                include_exts
                    .split_whitespace()
                    .map(|s| s.to_lowercase())
                    .collect(),
            );
        }

        // 跳过的扩展名
        print!("\x1b[33m?\x1b[0m 额外跳过的扩展名（空格分隔）: ");
        io::stdout().flush()?;
        let mut skip_exts = String::new();
        io::stdin().read_line(&mut skip_exts)?;
        config.skip_exts = skip_exts
            .split_whitespace()
            .map(|s| s.to_lowercase())
            .collect();

        // 忽略的目录名
        print!("\x1b[33m?\x1b[0m 忽略的目录名（空格分隔，如: tests docs）: ");
        io::stdout().flush()?;
        let mut ignore_dirs = String::new();
        io::stdin().read_line(&mut ignore_dirs)?;
        config.ignore_dirs = ignore_dirs
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();

        // 忽略的文件名
        print!("\x1b[33m?\x1b[0m 忽略的文件名（空格分隔，如: package-lock.json）: ");
        io::stdout().flush()?;
        let mut ignore_files = String::new();
        io::stdin().read_line(&mut ignore_files)?;
        config.ignore_files = ignore_files
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();

        // 是否生成目录树
        print!("\x1b[33m?\x1b[0m 生成目录树？[Y/n]: ");
        io::stdout().flush()?;
        let mut show_tree = String::new();
        io::stdin().read_line(&mut show_tree)?;
        config.show_tree = show_tree.trim().is_empty() || show_tree.trim().to_lowercase() == "y";

        // 是否生成 TOC
        print!("\x1b[33m?\x1b[0m 生成文件目录？[Y/n]: ");
        io::stdout().flush()?;
        let mut show_toc = String::new();
        io::stdin().read_line(&mut show_toc)?;
        config.show_toc = show_toc.trim().is_empty() || show_toc.trim().to_lowercase() == "y";
    }

    println!();
    print_colored(
        "green",
        "✓",
        &format!("目标目录: {}", config.path.display()),
    );
    print_colored(
        "green",
        "✓",
        &format!("输出文件: {}", config.outfile.display()),
    );
    println!();

    Ok(config)
}

// ═══════════════════════════════════════════════════════════════════════════
// 命令行参数解析
// ═══════════════════════════════════════════════════════════════════════════

fn parse_args() -> Result<Option<Config>, Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() == 1 {
        return Ok(None); // 交互模式
    }

    // 检查帮助
    if args.iter().any(|a| a == "-h" || a == "--help") {
        print_help();
        std::process::exit(0);
    }

    let mut config = Config::default();
    let mut i = 1;

    while i < args.len() {
        match args[i].as_str() {
            "-Path" => {
                i += 1;
                config.path = PathBuf::from(args.get(i).ok_or("缺少 -Path 的值")?);
            }
            "-OutFile" => {
                i += 1;
                config.outfile = PathBuf::from(args.get(i).ok_or("缺少 -OutFile 的值")?);
            }
            "-MaxBytes" => {
                i += 1;
                config.max_bytes = args.get(i).ok_or("缺少 -MaxBytes 的值")?.parse()?;
            }
            "-SkipExts" => {
                i += 1;
                config.skip_exts = args
                    .get(i)
                    .ok_or("缺少 -SkipExts 的值")?
                    .split_whitespace()
                    .map(|s| s.to_lowercase())
                    .collect();
            }
            "-IncludeExts" => {
                i += 1;
                config.include_exts = Some(
                    args.get(i)
                        .ok_or("缺少 -IncludeExts 的值")?
                        .split_whitespace()
                        .map(|s| s.to_lowercase())
                        .collect(),
                );
            }
            "-IgnoreDirs" => {
                i += 1;
                config.ignore_dirs = args
                    .get(i)
                    .ok_or("缺少 -IgnoreDirs 的值")?
                    .split_whitespace()
                    .map(|s| s.to_string())
                    .collect();
            }
            "-IgnoreFiles" => {
                i += 1;
                config.ignore_files = args
                    .get(i)
                    .ok_or("缺少 -IgnoreFiles 的值")?
                    .split_whitespace()
                    .map(|s| s.to_string())
                    .collect();
            }
            "-NoTree" => config.show_tree = false,
            "-NoToc" => config.show_toc = false,
            arg => return Err(format!("未知参数: {}", arg).into()),
        }
        i += 1;
    }

    if config.path.as_os_str().is_empty() {
        return Err("必须指定 -Path 参数".into());
    }

    Ok(Some(config))
}

// ═══════════════════════════════════════════════════════════════════════════
// 主运行函数
// ═══════════════════════════════════════════════════════════════════════════

fn main() {
    if let Err(e) = run() {
        print_colored("red", "✗", &format!("错误: {}", e));
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let config = match parse_args()? {
        Some(c) => c,
        None => interactive_input()?,
    };

    // 验证目录
    if !config.path.exists() {
        return Err(format!("路径不存在: {}", config.path.display()).into());
    }
    if !config.path.is_dir() {
        return Err(format!("路径不是目录: {}", config.path.display()).into());
    }

    // 构建跳过扩展名集合
    let mut skip_set: HashSet<String> = BINARY_EXTS.iter().map(|&s| s.to_string()).collect();
    skip_set.extend(config.skip_exts.clone());

    // 第一遍：收集文件信息
    print_colored("blue", "→", "正在扫描文件...");
    let mut stats = Stats::default();
    let mut entries: Vec<FileEntry> = Vec::new();
    let mut dir_tree: Vec<String> = Vec::new();

    collect_files(
        &config.path,
        &config.path,
        &skip_set,
        &config.ignore_dirs,
        &config.ignore_files,
        &config.include_exts,
        config.max_bytes,
        &mut stats,
        &mut entries,
        &mut dir_tree,
        0,
    )?;

    println!();
    print_colored(
        "green",
        "✓",
        &format!("扫描完成，共 {} 个文件", entries.len()),
    );

    // 确定输出路径
    let outfile_path = if config.outfile.is_relative() {
        std::env::current_dir()?.join(&config.outfile)
    } else {
        config.outfile.clone()
    };

    // 创建输出文件（覆盖模式）
    if let Some(parent) = outfile_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let file = File::create(&outfile_path)?;
    let mut writer = BufWriter::new(file);

    // 写入头部信息
    write_header(&mut writer, &config, &stats, &entries)?;

    // 写入目录树
    if config.show_tree && !dir_tree.is_empty() {
        write_tree(&mut writer, &dir_tree)?;
    }

    // 写入文件目录（TOC）
    if config.show_toc && !entries.is_empty() {
        write_toc(&mut writer, &entries)?;
    }

    // 写入文件内容
    print_colored("blue", "→", "正在写入文件内容...");
    write_files(&mut writer, &config.path, &entries)?;

    // 写入统计信息
    write_stats(&mut writer, &stats)?;

    writer.flush()?;

    // 输出完成信息
    println!();
    print_colored("green", "✓", &format!("已生成: {}", outfile_path.display()));
    println!();
    println!("  📄 文件数: {}", stats.files_processed);
    println!("  📁 目录数: {}", stats.dirs_count);
    println!("  📏 总行数: {}", stats.total_lines);
    println!("  💾 总大小: {}", format_size(stats.total_bytes));
    if stats.files_skipped_size > 0 {
        print_colored(
            "yellow",
            "  ⚠",
            &format!("跳过（过大）: {}", stats.files_skipped_size),
        );
    }
    if stats.files_skipped_binary > 0 {
        print_colored(
            "gray",
            "  ○",
            &format!("跳过（二进制）: {}", stats.files_skipped_binary),
        );
    }
    println!();

    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════
// 文件收集
// ═══════════════════════════════════════════════════════════════════════════

fn collect_files(
    dir: &Path,
    base_path: &Path,
    skip_set: &HashSet<String>,
    ignore_dirs: &HashSet<String>,
    ignore_files: &HashSet<String>,
    include_exts: &Option<HashSet<String>>,
    max_bytes: u64,
    stats: &mut Stats,
    entries: &mut Vec<FileEntry>,
    tree: &mut Vec<String>,
    depth: usize,
) -> io::Result<()> {
    let dir_name = dir
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| ".".to_string());

    // 跳过忽略的目录（默认列表 + 用户自定义列表）
    if IGNORED_DIRS.contains(&dir_name.as_str()) || ignore_dirs.contains(&dir_name) {
        return Ok(());
    }

    stats.dirs_count += 1;

    // 添加到目录树
    let prefix = if depth == 0 {
        String::new()
    } else {
        "│   ".repeat(depth - 1) + "├── "
    };
    tree.push(format!("{}{}/", prefix, dir_name));

    let mut items: Vec<_> = fs::read_dir(dir)?.filter_map(|e| e.ok()).collect();
    items.sort_by_key(|e| (e.path().is_file(), e.file_name()));

    for entry in items {
        let path = entry.path();
        let file_name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        if path.is_dir() {
            collect_files(
                &path,
                base_path,
                skip_set,
                ignore_dirs,
                ignore_files,
                include_exts,
                max_bytes,
                stats,
                entries,
                tree,
                depth + 1,
            )?;
        } else {
            // 检查特定文件名忽略
            if ignore_files.contains(&file_name) {
                stats.files_skipped_filter += 1;
                continue;
            }

            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_lowercase())
                .unwrap_or_default();

            // 检查白名单
            if let Some(include) = include_exts
                && !include.contains(&ext)
            {
                stats.files_skipped_filter += 1;
                continue;
            }

            // 检查黑名单
            if skip_set.contains(&ext) {
                stats.files_skipped_binary += 1;
                tree.push(format!(
                    "{}│   ├── {} (binary)",
                    "│   ".repeat(depth),
                    file_name
                ));
                continue;
            }

            // 检查文件大小
            let metadata = fs::metadata(&path)?;
            if metadata.len() > max_bytes {
                stats.files_skipped_size += 1;
                tree.push(format!(
                    "{}│   ├── {} ({})",
                    "│   ".repeat(depth),
                    file_name,
                    format_size(metadata.len())
                ));
                continue;
            }

            // 尝试读取文件
            match fs::read_to_string(&path) {
                Ok(content) => {
                    let line_count = content.lines().count();
                    let relative_path = path
                        .strip_prefix(base_path)
                        .map(|p| p.to_string_lossy().replace('\\', "/"))
                        .unwrap_or_else(|_| path.to_string_lossy().to_string());

                    print_progress(entries.len() + 1, &relative_path);

                    entries.push(FileEntry {
                        relative_path,
                        extension: ext.clone(),
                        size: metadata.len(),
                        line_count,
                    });

                    stats.files_processed += 1;
                    stats.total_bytes += metadata.len();
                    stats.total_lines += line_count;
                    *stats.file_types.entry(ext).or_insert(0) += 1;

                    tree.push(format!("{}│   ├── {}", "│   ".repeat(depth), file_name));
                }
                Err(_) => {
                    stats.files_skipped_encoding += 1;
                }
            }
        }
    }

    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════
// Markdown 输出
// ═══════════════════════════════════════════════════════════════════════════

fn write_header(
    writer: &mut BufWriter<File>,
    config: &Config,
    _stats: &Stats,
    entries: &[FileEntry],
) -> io::Result<()> {
    let now = Local::now();
    let project_name = config
        .path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "Project".to_string());

    writeln!(writer, "# 📦 {} - Code Collection", project_name)?;
    writeln!(writer)?;
    writeln!(writer, "> 由 Code Collector v{} 自动生成", VERSION)?;
    writeln!(writer)?;
    writeln!(writer, "| 属性 | 值 |")?;
    writeln!(writer, "|------|-----|")?;
    writeln!(
        writer,
        "| 📅 生成时间 | {} |",
        now.format("%Y-%m-%d %H:%M:%S")
    )?;
    writeln!(writer, "| 📁 源目录 | `{}` |", config.path.display())?;
    writeln!(writer, "| 📄 文件数量 | {} |", entries.len())?;
    writeln!(writer)?;
    writeln!(writer, "---")?;
    writeln!(writer)?;

    Ok(())
}

fn write_tree(writer: &mut BufWriter<File>, tree: &[String]) -> io::Result<()> {
    writeln!(writer, "## 📂 目录结构")?;
    writeln!(writer)?;
    writeln!(writer, "```")?;
    for line in tree {
        writeln!(writer, "{}", line)?;
    }
    writeln!(writer, "```")?;
    writeln!(writer)?;
    writeln!(writer, "---")?;
    writeln!(writer)?;

    Ok(())
}

fn write_toc(writer: &mut BufWriter<File>, entries: &[FileEntry]) -> io::Result<()> {
    writeln!(writer, "## 📑 文件目录")?;
    writeln!(writer)?;
    writeln!(writer, "| # | 文件 | 类型 | 行数 | 大小 |")?;
    writeln!(writer, "|---|------|------|------|------|")?;

    for (i, entry) in entries.iter().enumerate() {
        let anchor = entry
            .relative_path
            .replace(['/', '.', ' '], "-")
            .to_lowercase();
        writeln!(
            writer,
            "| {} | [{}](#{}) | {} | {} | {} |",
            i + 1,
            entry.relative_path,
            anchor,
            entry.extension,
            entry.line_count,
            format_size(entry.size)
        )?;
    }

    writeln!(writer)?;
    writeln!(writer, "---")?;
    writeln!(writer)?;

    Ok(())
}

fn write_files(
    writer: &mut BufWriter<File>,
    base_path: &Path,
    entries: &[FileEntry],
) -> io::Result<()> {
    writeln!(writer, "## 📄 文件内容")?;
    writeln!(writer)?;

    for (i, entry) in entries.iter().enumerate() {
        print_progress(i + 1, &entry.relative_path);

        let file_path = base_path.join(&entry.relative_path);
        let content = fs::read_to_string(&file_path)?;
        let lang = get_language(&entry.extension);

        // 动态计算需要的反引号数量，确保比内容中最长的反引号序列多
        let max_backticks = count_max_backticks(&content);
        let fence_count = if max_backticks >= 3 {
            max_backticks + 1
        } else {
            3
        };
        let fence: String = "`".repeat(fence_count);

        writeln!(writer, "### {}", entry.relative_path)?;
        writeln!(writer)?;
        writeln!(
            writer,
            "> 📏 {} 行 | 💾 {}",
            entry.line_count,
            format_size(entry.size)
        )?;
        writeln!(writer)?;
        writeln!(writer, "{}{}", fence, lang)?;
        write!(writer, "{}", content)?;
        if !content.ends_with('\n') {
            writeln!(writer)?;
        }
        writeln!(writer, "{}", fence)?;
        writeln!(writer)?;
    }

    println!(); // 清除进度行
    Ok(())
}

fn write_stats(writer: &mut BufWriter<File>, stats: &Stats) -> io::Result<()> {
    writeln!(writer, "---")?;
    writeln!(writer)?;
    writeln!(writer, "## 📊 统计信息")?;
    writeln!(writer)?;
    writeln!(writer, "### 文件类型分布")?;
    writeln!(writer)?;
    writeln!(writer, "| 扩展名 | 文件数 |")?;
    writeln!(writer, "|--------|--------|")?;

    let mut types: Vec<_> = stats.file_types.iter().collect();
    types.sort_by(|a, b| b.1.cmp(a.1));

    for (ext, count) in types {
        writeln!(writer, "| .{} | {} |", ext, count)?;
    }

    writeln!(writer)?;
    writeln!(writer, "### 汇总")?;
    writeln!(writer)?;
    writeln!(writer, "- **处理文件**: {}", stats.files_processed)?;
    writeln!(writer, "- **总代码行数**: {}", stats.total_lines)?;
    writeln!(writer, "- **总大小**: {}", format_size(stats.total_bytes))?;
    writeln!(writer, "- **跳过（过大）**: {}", stats.files_skipped_size)?;
    writeln!(
        writer,
        "- **跳过（二进制）**: {}",
        stats.files_skipped_binary
    )?;
    writeln!(
        writer,
        "- **跳过（编码问题）**: {}",
        stats.files_skipped_encoding
    )?;
    writeln!(writer)?;

    Ok(())
}
