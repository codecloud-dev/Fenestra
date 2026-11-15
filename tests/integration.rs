//! 端到端验收：从三家扩展的 manifest.json 一路走到能力仲裁与可逆卸载。

use fenestra::{from_json_manifest, to_hep, Capability, ExtensionFlavor, Grant, Shell, Verdict};

const CHROME: &str = fenestra::compat::CHROME_MV3;
const EDGE: &str = fenestra::compat::EDGE_MV3;
const FIREFOX: &str = fenestra::compat::FIREFOX_MV3;

fn grant_for(report: &fenestra::CompatReport) -> Grant {
    let mut g = Grant::new();
    for cap in &report.manifest.capabilities {
        g = g.allow(*cap);
    }
    g
}

#[test]
fn three_extension_flavors_install_into_one_shell() {
    let mut shell = Shell::new();
    let mut flavors = Vec::new();
    for text in [CHROME, EDGE, FIREFOX] {
        let report = from_json_manifest(text).expect("样例清单应可归一");
        flavors.push(report.flavor);
        let hep = to_hep(&report.manifest);
        let grant = grant_for(&report);
        shell
            .install(&hep, &grant)
            .expect("归一后的 FEP 清单应可直接安装");
    }
    assert_eq!(
        flavors,
        vec![
            ExtensionFlavor::Chrome,
            ExtensionFlavor::Edge,
            ExtensionFlavor::Firefox,
        ]
    );
    assert_eq!(shell.active().len(), 3);
    assert_eq!(shell.publish("shell.boot").len(), 3);
    assert!(shell.services().contains(&"ext.load".to_string()));
}

#[test]
fn firefox_gecko_id_survives_roundtrip() {
    let report = from_json_manifest(FIREFOX).unwrap();
    let hep = to_hep(&report.manifest);
    let parsed = Shell::verify(&hep).unwrap();
    assert_eq!(parsed.id, "reader@example.org");
    assert_eq!(parsed.kind, fenestra::PluginKind::ExtensionRuntime);
}

#[test]
fn rights_are_least_privilege_even_after_compat() {
    let report = from_json_manifest(CHROME).unwrap();
    let mut shell = Shell::new();
    // 只授予一项：其余声明全部应被裁掉。
    let narrow = Grant::new().allow(Capability::StorageRead);
    let reg = shell.install(&to_hep(&report.manifest), &narrow).unwrap();
    assert_eq!(reg.capabilities, vec![Capability::StorageRead]);
    assert_eq!(
        shell.request(&reg.plugin_id, Capability::NetIntercept, false),
        Verdict::Deny
    );
    assert_eq!(
        shell.request(&reg.plugin_id, Capability::StorageRead, false),
        Verdict::Allow
    );
}

#[test]
fn uninstall_clears_every_trace() {
    let report = from_json_manifest(EDGE).unwrap();
    let mut shell = Shell::new();
    let reg = shell
        .install(&to_hep(&report.manifest), &grant_for(&report))
        .unwrap();
    assert_eq!(shell.publish("shell.boot").len(), 1);
    assert!(shell.uninstall(reg.id));
    assert!(shell.active().is_empty());
    assert!(shell.services().is_empty());
    assert!(shell.publish("shell.boot").is_empty());
    assert_eq!(
        shell.request(&reg.plugin_id, Capability::StorageRead, false),
        Verdict::Deny
    );
}

#[test]
fn lockdown_then_uninstall_is_still_consistent() {
    let report = from_json_manifest(FIREFOX).unwrap();
    let mut shell = Shell::new();
    let reg = shell
        .install(&to_hep(&report.manifest), &grant_for(&report))
        .unwrap();
    shell.harden();
    assert_eq!(
        shell.request(&reg.plugin_id, Capability::NetIntercept, false),
        Verdict::Deny
    );
    assert!(!shell.policy().weakened());
    assert_eq!(shell.uninstall_plugin(&reg.plugin_id), 1);
    assert!(shell.active().is_empty());
}

#[test]
fn compat_warns_about_real_cross_browser_gaps() {
    let chrome = from_json_manifest(CHROME).unwrap();
    let firefox = from_json_manifest(FIREFOX).unwrap();
    assert!(chrome
        .warnings
        .iter()
        .any(|w| w.contains("declarativeNetRequest")));
    assert!(firefox.warnings.iter().any(|w| w.contains("webRequest")));
    assert!(!firefox.warnings.iter().any(|w| w.contains("AMO 要求声明")));
}
