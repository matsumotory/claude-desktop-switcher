// Detection and ejection of a leftover CSW installer disk image.
// Spec: docs/SPECIFICATION.md §5.A「インストール用ディスクイメージの取り出し案内」.
// A mounted image qualifies when its image file name or a mount-point name
// starts with "Claude-Desktop-Switcher" (the productName; the DMG file and its
// volume are both derived from it). The volume the running app itself lives on
// is excluded: ejecting it would kill the app, and the right guidance there is
// to install first, not to eject. eject() only accepts a mount point that the
// detection currently reports, so arbitrary paths can never be detached.

use std::path::Path;
use std::process::Command;

const DMG_NAME_PREFIX: &str = "Claude-Desktop-Switcher";

fn name_matches(path: &str) -> bool {
    Path::new(path)
        .file_name()
        .map(|n| n.to_string_lossy().starts_with(DMG_NAME_PREFIX))
        .unwrap_or(false)
}

/// Mount points of mounted CSW installer images, from `hdiutil info -plist`
/// output. An image whose volume the running app lives on is skipped entirely.
pub fn csw_dmg_mount_points(hdiutil_plist: &[u8], current_exe: &Path) -> Vec<String> {
    let Ok(root) = plist::Value::from_reader(std::io::Cursor::new(hdiutil_plist)) else {
        return Vec::new();
    };
    let Some(images) = root
        .as_dictionary()
        .and_then(|d| d.get("images"))
        .and_then(|v| v.as_array())
    else {
        return Vec::new();
    };

    let mut result = Vec::new();
    for image in images {
        let Some(image) = image.as_dictionary() else {
            continue;
        };
        let image_path = image
            .get("image-path")
            .and_then(|v| v.as_string())
            .unwrap_or("");
        let mounts: Vec<String> = image
            .get("system-entities")
            .and_then(|v| v.as_array())
            .map(|entities| {
                entities
                    .iter()
                    .filter_map(|e| e.as_dictionary())
                    .filter_map(|e| e.get("mount-point"))
                    .filter_map(|v| v.as_string())
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default();

        let qualifies = name_matches(image_path) || mounts.iter().any(|m| name_matches(m));
        if !qualifies {
            continue;
        }
        // Never offer to eject the volume the app itself is running from.
        if mounts.iter().any(|m| current_exe.starts_with(m)) {
            continue;
        }
        result.extend(mounts);
    }
    result
}

/// Run hdiutil and report the currently mounted CSW installer volumes.
pub fn current_mount_status() -> Vec<String> {
    let Ok(out) = Command::new("hdiutil").args(["info", "-plist"]).output() else {
        return Vec::new();
    };
    if !out.status.success() {
        return Vec::new();
    }
    let exe = std::env::current_exe().unwrap_or_default();
    csw_dmg_mount_points(&out.stdout, &exe)
}

/// Detach a mount point, but only one the detection currently reports; any
/// other path is refused so the WebView can never eject an arbitrary volume.
pub fn eject(mount_point: &str) -> Result<(), String> {
    if !current_mount_status().iter().any(|m| m == mount_point) {
        return Err(format!("not a CSW disk image mount: {mount_point}"));
    }
    let out = Command::new("hdiutil")
        .args(["detach", mount_point])
        .output()
        .map_err(|e| e.to_string())?;
    if out.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    // Minimal but structurally faithful reproduction of `hdiutil info -plist`
    // output as observed on macOS 15 (images[] -> image-path + system-entities[]
    // where only mounted filesystem entities carry mount-point).
    fn fixture(image_path: &str, mount_point: &str) -> String {
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>images</key>
  <array>
    <dict>
      <key>image-path</key><string>{image_path}</string>
      <key>system-entities</key>
      <array>
        <dict><key>content-hint</key><string>GUID_partition_scheme</string></dict>
        <dict>
          <key>content-hint</key><string>48465300-0000-11AA-AA11-00306543ECAC</string>
          <key>mount-point</key><string>{mount_point}</string>
        </dict>
      </array>
    </dict>
  </array>
</dict>
</plist>"#
        )
    }

    const APP_IN_APPLICATIONS: &str =
        "/Applications/Claude-Desktop-Switcher.app/Contents/MacOS/csw-desktop";

    #[test]
    fn detects_mounted_csw_image_by_volume_name() {
        let xml = fixture(
            "/Users/someone/Downloads/Claude-Desktop-Switcher_0.15.1_universal.dmg",
            "/Volumes/Claude-Desktop-Switcher",
        );
        assert_eq!(
            csw_dmg_mount_points(xml.as_bytes(), Path::new(APP_IN_APPLICATIONS)),
            vec!["/Volumes/Claude-Desktop-Switcher".to_string()]
        );
    }

    #[test]
    fn detects_by_image_file_name_when_volume_was_renamed() {
        let xml = fixture(
            "/Users/someone/Downloads/Claude-Desktop-Switcher_0.15.1_universal.dmg",
            "/Volumes/Install",
        );
        assert_eq!(
            csw_dmg_mount_points(xml.as_bytes(), Path::new(APP_IN_APPLICATIONS)),
            vec!["/Volumes/Install".to_string()]
        );
    }

    #[test]
    fn detects_by_volume_name_when_the_file_was_renamed() {
        // macOS suffixes duplicate volume names with " 1", which must still match.
        let xml = fixture(
            "/Users/someone/Downloads/installer.dmg",
            "/Volumes/Claude-Desktop-Switcher 1",
        );
        assert_eq!(
            csw_dmg_mount_points(xml.as_bytes(), Path::new(APP_IN_APPLICATIONS)),
            vec!["/Volumes/Claude-Desktop-Switcher 1".to_string()]
        );
    }

    #[test]
    fn ignores_unrelated_disk_images() {
        let xml = fixture(
            "/Users/someone/Downloads/SomeOtherApp.dmg",
            "/Volumes/SomeOtherApp",
        );
        assert!(csw_dmg_mount_points(xml.as_bytes(), Path::new(APP_IN_APPLICATIONS)).is_empty());
    }

    #[test]
    fn excludes_the_volume_the_app_itself_runs_from() {
        let xml = fixture(
            "/Users/someone/Downloads/Claude-Desktop-Switcher_0.15.1_universal.dmg",
            "/Volumes/Claude-Desktop-Switcher",
        );
        let exe = "/Volumes/Claude-Desktop-Switcher/Claude-Desktop-Switcher.app/Contents/MacOS/csw-desktop";
        assert!(csw_dmg_mount_points(xml.as_bytes(), Path::new(exe)).is_empty());
    }

    #[test]
    fn malformed_plist_yields_empty() {
        assert!(csw_dmg_mount_points(b"not a plist", Path::new(APP_IN_APPLICATIONS)).is_empty());
        assert!(csw_dmg_mount_points(b"", Path::new(APP_IN_APPLICATIONS)).is_empty());
    }

    // Live round trip on the real hdiutil: create a tiny CSW-named image, attach
    // it, confirm detection sees exactly that mount, eject through eject(), and
    // confirm it disappears. Tolerates other CSW volumes being mounted on the
    // machine (asserts only about the test volume). CI never runs this: test.yml
    // excludes csw-desktop, and this file is macOS-only anyway.
    #[test]
    fn live_create_attach_detect_and_eject() {
        use std::process::Command;
        let dir = std::env::temp_dir().join(format!("csw-dmg-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let dmg = dir.join("Claude-Desktop-Switcher-CSWTest.dmg");
        let status = Command::new("hdiutil")
            .args([
                "create",
                "-size",
                "1m",
                "-fs",
                "HFS+",
                "-volname",
                "Claude-Desktop-Switcher-CSWTest",
            ])
            .arg(&dmg)
            .status()
            .unwrap();
        assert!(status.success(), "hdiutil create failed");
        let out = Command::new("hdiutil")
            .args(["attach", "-nobrowse"])
            .arg(&dmg)
            .output()
            .unwrap();
        assert!(out.status.success(), "hdiutil attach failed");

        let mounts = current_mount_status();
        let test_mount = mounts
            .iter()
            .find(|m| m.contains("Claude-Desktop-Switcher-CSWTest"))
            .expect("test volume not detected")
            .clone();

        eject(&test_mount).expect("eject failed");
        assert!(
            !current_mount_status().iter().any(|m| m == &test_mount),
            "test volume still mounted after eject"
        );

        // A path that detection does not report must be refused.
        assert!(eject("/Volumes/NotDetected").is_err());

        std::fs::remove_dir_all(&dir).ok();
    }
}
