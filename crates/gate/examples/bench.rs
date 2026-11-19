//! 闸门真实微基准（零第三方依赖，std::time::Instant）。
//!
//!   cargo run --release -p fenestra-gate --example bench -- 200000
//!
//! 打印的是本机实测 ns/次，随机器负载波动；不写进文档时脱离具体机器与迭代数。
//! 不引入 criterion 等第三方依赖，保持闸门「零依赖」边界。

use fenestra_gate::{Capability, Gate, AUDIT_CAPACITY};
use std::time::Instant;

fn main() {
    let iters: usize = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(100_000);
    assert!(iters > 0, "迭代数必须为正");

    let mut gate = Gate::new();
    gate.set_policy(
        [
            Capability::UiNewTab,
            Capability::DnsResolve,
            Capability::NetOutbound,
        ]
        .into_iter()
        .collect(),
    );
    gate.declare("newtab", [Capability::UiNewTab].into_iter().collect());
    gate.declare(
        "accelerate",
        [Capability::DnsResolve, Capability::NetOutbound]
            .into_iter()
            .collect(),
    );

    // 预热：让分支预测与代码缓存进入稳态。
    for _ in 0..iters.min(10_000) {
        let _ = gate.check("accelerate", Capability::DnsResolve);
    }

    // 1) 纯检查（&self，不记录审计）。
    let mut allowed = 0usize;
    let t0 = Instant::now();
    for _ in 0..iters {
        if gate.check("accelerate", Capability::DnsResolve).is_allowed() {
            allowed += 1;
        }
    }
    let check_ns = t0.elapsed().as_nanos() as f64 / iters as f64;

    // 2) 权限字符串入口（&mut self，含解析 + 有界审计写入）。
    let mut strings_allowed = 0usize;
    let t1 = Instant::now();
    for _ in 0..iters {
        if gate
            .check_permission("accelerate", "net.dns")
            .is_allowed()
        {
            strings_allowed += 1;
        }
    }
    let permission_ns = t1.elapsed().as_nanos() as f64 / iters as f64;

    // 3) 最小权限交集（每次分配一个 BTreeSet）。
    let mut eff_total = 0usize;
    let t2 = Instant::now();
    for _ in 0..iters {
        eff_total += gate.effective("accelerate").len();
    }
    let effective_ns = t2.elapsed().as_nanos() as f64 / iters as f64;

    println!("Fenestra gate benchmark · iters={iters} · AUDIT_CAPACITY={AUDIT_CAPACITY}");
    println!("check(纯检查)          {check_ns:>9.1} ns/op  (allowed={allowed})");
    println!("check_permission(审计) {permission_ns:>9.1} ns/op  (allowed={strings_allowed})");
    println!("effective(含分配)      {effective_ns:>9.1} ns/op  ( caps={eff_total})");
}
