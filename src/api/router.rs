use crate::server::{Response, json_error, json_ok};

use super::{android, evidence, ram, system, wireguard};

pub fn route_api(method: &str, path: &str, body: &[u8]) -> Response {
    match (method, path) {
        ("GET", "/api/health") => json_ok(serde_json::json!({
            "ok": true,
            "version": env!("CARGO_PKG_VERSION"),
        })),
        ("GET", "/api/settings-default") => {
            match serde_json::to_value(crate::settings::AppSettings::default()) {
                Ok(value) => json_ok(value),
                Err(err) => json_error(500, err.to_string()),
            }
        }
        ("GET", "/api/disk-list") => system::disk_list_endpoint(),
        ("GET", "/api/android-adb-status") => {
            match serde_json::to_value(crate::android::adb_status()) {
                Ok(value) => json_ok(value),
                Err(err) => json_error(500, err.to_string()),
            }
        }
        ("GET", "/api/android-devices") => match crate::android::list_devices() {
            Ok(devices) => json_ok(serde_json::json!({ "devices": devices })),
            Err(err) => json_error(500, crate::android::explain_android_error(err)),
        },
        ("POST", "/api/android-device-profile") => android::android_device_profile_endpoint(body),
        ("POST", "/api/android-profile-acquisition") => {
            android::android_profile_acquisition_endpoint(body)
        }
        ("POST", "/api/android-logical-image") => android::android_logical_image_endpoint(body),
        ("POST", "/api/android-filesystem-image") => {
            android::android_filesystem_image_endpoint(body)
        }
        ("POST", "/api/android-ram-image") => android::android_ram_image_endpoint(body),
        ("GET", "/api/ram-status") => json_ok(serde_json::json!({
            "avml": crate::ram::avml_status(None),
            "winpmem": crate::ram::winpmem_status(None),
        })),
        ("POST", "/api/avml-install") => ram::avml_install_endpoint(),
        ("POST", "/api/winpmem-install") => ram::winpmem_install_endpoint(),
        ("POST", "/api/acquisition-control") => ram::acquisition_control_endpoint(body),
        ("POST", "/api/acquisition-status") => ram::acquisition_status_endpoint(body),
        ("POST", "/api/connect") => system::connect_endpoint(body),
        ("POST", "/api/hash") => system::hash_endpoint(body),
        ("POST", "/api/local-image") => system::local_image_endpoint(body),
        ("POST", "/api/local-ram") => ram::local_ram_endpoint(body),
        ("POST", "/api/remote-disks") => system::remote_disks_endpoint(body),
        ("POST", "/api/remote-image") => system::remote_image_endpoint(body),
        ("POST", "/api/remote-ram") => ram::remote_ram_endpoint(body),
        ("POST", "/api/remote-tool-check") => system::remote_tool_check_endpoint(body),
        ("POST", "/api/evidence-create") => evidence::evidence_create_endpoint(body),
        ("POST", "/api/evidence-add-note") => evidence::evidence_add_note_endpoint(body),
        ("POST", "/api/evidence-list-files") => evidence::evidence_list_files_endpoint(body),
        ("GET", "/api/evidence-cases") => evidence::evidence_cases_endpoint(),
        ("GET", "/api/evidence-summary") => evidence::evidence_summary_endpoint(),
        ("POST", "/api/report-create") => evidence::report_create_endpoint(body),
        ("POST", "/api/image-mount-readonly") => system::image_mount_readonly_endpoint(body),
        ("POST", "/api/image-unmount") => system::image_unmount_endpoint(),
        ("POST", "/api/image-analyze") => system::image_analyze_endpoint(body),
        ("POST", "/api/image-browse") => system::image_browse_endpoint(body),
        ("POST", "/api/image-read-file") => system::image_read_file_endpoint(body),
        ("POST", "/api/ram-analyze-strings") => ram::ram_analyze_strings_endpoint(body),
        ("POST", "/api/ram-carve-files") => ram::ram_carve_files_endpoint(body),
        ("POST", "/api/ram-list-processes") => ram::ram_list_processes_endpoint(body),
        ("POST", "/api/ram-process-details") => ram::ram_process_details_endpoint(body),
        ("POST", "/api/ram-process-search") => ram::ram_process_search_endpoint(body),
        ("POST", "/api/ram-read-carved") => ram::ram_read_carved_endpoint(body),
        ("POST", "/api/wireguard-config") => wireguard::wireguard_config_endpoint(body),
        ("POST", "/api/wireguard-start") => wireguard::wireguard_start_endpoint(body),
        ("POST", "/api/wireguard-stop") => wireguard::wireguard_stop_endpoint(),
        ("GET", "/api/wireguard-status") => wireguard::wireguard_status_endpoint(),
        ("GET", "/api/update-check") => system::update_check_endpoint(),
        ("POST", "/api/update-download") => system::update_download_endpoint(body),
        ("POST", "/api/update-install") => system::update_install_endpoint(body),
        ("POST", "/api/open-url") => system::open_url_endpoint(body),
        ("POST", "/api/pick-file") => system::pick_path_endpoint(false),
        ("POST", "/api/pick-folder") => system::pick_path_endpoint(true),
        _ => json_error(404, "api endpoint not found"),
    }
}
