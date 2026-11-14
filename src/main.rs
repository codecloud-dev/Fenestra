//! Hollo CLI —— 用来验证「壳」本身，不是浏览器成品。
//!
//! 这个二进制里的 StubEngine 只是演示用的插件替身；
//! 内核（library）里没有任何引擎实现。

use std::env;
use std::fs;
use std::process::ExitCode;

use hollo::{
    from_json_manifest, to_hep, Capability, EngineError, Frame, Grant, RenderEngine, RenderRequest,
    Shell, Target, KERNEL_VERSION,
};

use hollo::manifest::{EXAMPLE_ENGINE, EXAMPLE_EXT, EXAMPLE_UI};

/// 演示用引擎插件：证明"引擎由插件提供"这件事在类型层面成立。
struct StubEngine {
    label: String,
}

impl RenderEngine for StubEngine {
    fn name(&self) -> &str {
        &self.label
    }

    fn render(&self, req: &RenderRequest) -> Result<Frame, EngineError> {
        Ok(Frame {
            engine: self.label.clone(),
            bytes: req.url.len() + req.origin.len(),
        })
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    let cmd = args.get(1).map(String::as_str).unwrap_or("help");
    match cmd {
        "verify" => cmd_verify(args.get(2)),
        "compat" => cmd_compat(args.get(2)),
        "demo" => {
            cmd_demo();
            ExitCode::SUCCESS
        }
        "policy" => {
            cmd_policy();
            ExitCode::SUCCESS
        }
        "targets" => {
            cmd_targets();
            ExitCode::SUCCESS
        }
        "--version" | "-V" => {
            println!("hollo {KERNEL_VERSION} (HEP v{})", hollo::HEP_VERSION);
            ExitCode::SUCCESS
        }
        "help" | "--help" | "-h" => {
            print_help();
            ExitCode::SUCCESS
        }
        other => {
            eprintln!("未知命令: {other}\n");
            print_help();
            ExitCode::FAILURE
        }
    }
}

fn read(path: Option<&String>) -> Result<String, ExitCode> {
    let Some(path) = path else {
        eprintln!("缺少文件路径");
        return Err(ExitCode::FAILURE);
    };
    match fs::read_to_string(path) {
        Ok(s) => Ok(s),
        Err(e) => {
            eprintln!("读取 {path} 失败: {e}");
            Err(ExitCode::FAILURE)
        }
    }
}

fn cmd_verify(path: Option<&String>) -> ExitCode {
    let text = match read(path) {
        Ok(t) => t,
        Err(code) => return code,
    };
    match Shell::verify(&text) {
        Ok(m) => {
            println!("✓ 清单合法");
            println!("  id       : {}", m.id);
            println!("  name     : {}", m.name);
            println!("  version  : {}", m.version);
            println!("  kind     : {}", m.kind.as_str());
            println!("  entry    : {}", m.entry);
            println!(
                "  target   : {}",
                m.target.map(|t| t.as_str()).unwrap_or("any")
            );
            println!(
                "  能力     : {:?}",
                m.capabilities
                    .iter()
                    .map(|c| c.as_str())
                    .collect::<Vec<_>>()
            );
            println!("  origins  : {:?}", m.origins);
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("✗ 清单非法: {e}");
            ExitCode::FAILURE
        }
    }
}

fn cmd_compat(path: Option<&String>) -> ExitCode {
    let text = match read(path) {
        Ok(t) => t,
        Err(code) => return code,
    };
    match from_json_manifest(&text) {
        Ok(r) => {
            println!(
                "✓ 归一成功: {} (MV{})",
                r.flavor.as_str(),
                r.manifest_version
            );
            println!("  entry   : {}", r.entry);
            println!(
                "  能力    : {:?}",
                r.manifest
                    .capabilities
                    .iter()
                    .map(|c| c.as_str())
                    .collect::<Vec<_>>()
            );
            println!("  origins : {:?}", r.manifest.origins);
            println!("  差异提示: {} 条", r.warnings.len());
            for (i, w) in r.warnings.iter().enumerate() {
                println!("    {}. {w}", i + 1);
            }
            println!("\n--- 生成的 HEP 清单 ---");
            print!("{}", to_hep(&r.manifest));
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("✗ 归一失败: {e}");
            ExitCode::FAILURE
        }
    }
}

fn cmd_policy() {
    let p = Shell::new();
    let s = p.policy();
    println!("Hollo 默认沙盒策略");
    println!("  隔离级别        : {}", s.level.as_str());
    println!("  默认拒绝        : {}", s.default_deny);
    println!("  锁定模式禁 JIT  : {}", s.deny_jit_in_lockdown);
    println!("  出站数据重校验  : {}", s.revalidate_untrusted_output);
    println!("  当前锁定模式    : {}", s.lockdown);
    println!("  策略已被削弱    : {}", s.weakened());
}

fn cmd_targets() {
    println!("Hollo 内核支持的目标平台（内核零依赖，平台差异由插件承担）:");
    for t in Target::SUPPORTED {
        println!("  {:<8} → {}", t.as_str(), t.rust_target_triple());
    }
}

fn cmd_demo() {
    println!("Hollo {} — 只是个壳，其余皆插件\n", KERNEL_VERSION);

    let mut shell = Shell::new();
    let grant = Grant::new()
        .allow(Capability::EngineSwap)
        .allow(Capability::NetConnect)
        .allow(Capability::NetIntercept)
        .allow(Capability::UiChrome)
        .allow(Capability::UiContent)
        .allow(Capability::StorageRead);

    println!("[1] 安装插件（渲染引擎 / 界面 / 扩展运行时都是插件）");
    for (label, text) in [
        ("渲染引擎", EXAMPLE_ENGINE),
        ("外壳界面", EXAMPLE_UI),
        ("扩展运行时", EXAMPLE_EXT),
    ] {
        match shell.install(text, &grant) {
            Ok(r) => println!(
                "    ✓ {:<6} {}  能力={:?} 服务={:?}",
                label,
                r.plugin_id,
                r.capabilities
                    .iter()
                    .map(|c| c.as_str())
                    .collect::<Vec<_>>(),
                r.services
            ),
            Err(e) => println!("    ✗ {label}: {e}"),
        }
    }

    println!("\n[2] 内核当前提供的服务（全部来自插件，内核自身为 0）");
    println!("    {:?}", shell.services());
    println!("    shell.boot 订阅者: {:?}", shell.publish("shell.boot"));

    println!("\n[3] 引擎是插件替身，调用一次看看");
    let engine = StubEngine {
        label: "demo-engine".to_string(),
    };
    let frame = engine
        .render(&RenderRequest {
            url: "https://example.com/".to_string(),
            origin: "https://example.com".to_string(),
        })
        .expect("演示引擎应返回结果");
    println!("    {:?}", frame);

    println!("\n[4] 能力仲裁：默认拒绝");
    let ext = "org.hollo.ext.webextensions";
    println!(
        "    storage.read → {:?}",
        shell.request(ext, Capability::StorageRead, false)
    );
    println!(
        "    telemetry    → {:?}（未授予）",
        shell.request(ext, Capability::Telemetry, false)
    );
    println!(
        "    engine.swap  → {:?}（未授予给该插件）",
        shell.request(ext, Capability::EngineSwap, false)
    );

    println!("\n[5] 锁定模式：冻结可改变隔离边界的能力");
    let engine_id = "org.hollo.engine.blink";
    println!(
        "    加固前 engine.swap → {:?}",
        shell.request(engine_id, Capability::EngineSwap, false)
    );
    shell.harden();
    println!(
        "    加固后 engine.swap → {:?}",
        shell.request(engine_id, Capability::EngineSwap, false)
    );
    println!(
        "    加固后 net.connect → {:?}",
        shell.request(engine_id, Capability::NetConnect, false)
    );

    println!("\n[6] 三类扩展 manifest.json 归一为同一份 HEP 清单");
    for (flavor, text) in [
        ("Chrome", hollo::compat::CHROME_MV3),
        ("Edge", hollo::compat::EDGE_MV3),
        ("Firefox", hollo::compat::FIREFOX_MV3),
    ] {
        let r = from_json_manifest(text).expect("内置样例应可归一");
        println!(
            "    {:<8} → id={} 能力={} 差异提示={}",
            flavor,
            r.manifest.id,
            r.manifest.capabilities.len(),
            r.warnings.len()
        );
    }

    println!("\n[7] 卸载可逆：撤销后服务与订阅一起消失");
    let ids: Vec<u64> = shell.active().iter().map(|r| r.id).collect();
    for id in ids {
        shell.uninstall(id);
    }
    println!("    剩余插件: {}", shell.active().len());
    println!("    剩余服务: {:?}", shell.services());
    println!("    shell.boot 订阅者: {:?}", shell.publish("shell.boot"));

    println!("\n结论：内核没有渲染、没有网络、没有界面、没有遥测。它只是个壳。");
}

fn print_help() {
    println!("Hollo {KERNEL_VERSION} — 只是个壳，其余皆插件\n");
    println!("用法:");
    println!("  hollo verify <file.hep>        校验 HEP 清单");
    println!("  hollo compat <manifest.json>   把 Chrome/Edge/Firefox 扩展归一为 HEP");
    println!("  hollo demo                      跑一遍内核自检演示");
    println!("  hollo policy                    打印默认沙盒策略");
    println!("  hollo targets                   打印支持的平台与 target triple");
    println!("  hollo --version                 打印版本");
}
