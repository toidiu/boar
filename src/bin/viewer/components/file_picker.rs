use leptos::prelude::*;
use std::collections::HashMap;

use super::app::ReportWithCdf;

#[component]
pub fn FilePicker(
    set_reports: WriteSignal<Vec<ReportWithCdf>>,
    reports: ReadSignal<Vec<ReportWithCdf>>,
    clear_trigger: ReadSignal<u32>,
    input_reset: ReadSignal<u32>,
) -> impl IntoView {
    use leptos::html::Input;

    let input_ref: NodeRef<Input> = NodeRef::new();

    // Track the number of reports found (None = never loaded, Some(n) = loaded n reports)
    let (report_count, set_report_count) = signal(Option::<usize>::None);

    // Set webkitdirectory attribute after mount
    Effect::new(move |_| {
        if let Some(input) = input_ref.get() {
            let el: &web_sys::Element = input.as_ref();
            let _ = el.set_attribute("webkitdirectory", "true");
        }
    });

    // Update report count when reports change (e.g., when × is clicked)
    Effect::new(move |prev_count: Option<usize>| {
        let count = reports.read().len();
        // Only update if we've loaded reports at least once (prev_count is Some)
        // and the count has actually changed
        if prev_count.is_some() && prev_count != Some(count) {
            set_report_count.set(Some(count));
        }
        count
    });

    // Reset file input and report count when clear_trigger changes (Clear All)
    Effect::new(move |prev: Option<u32>| {
        let current = clear_trigger.get();
        if prev.is_some() && prev != Some(current) {
            // Clear was triggered - reset the file input and count
            if let Some(input) = input_ref.get() {
                input.set_value("");
            }
            set_report_count.set(None);
        }
        current
    });

    // Reset file input only when input_reset changes (× button clicked)
    Effect::new(move |prev: Option<u32>| {
        let current = input_reset.get();
        if prev.is_some() && prev != Some(current) {
            // Reset file input only (keep count as-is, it's updated by the reports Effect)
            if let Some(input) = input_ref.get() {
                input.set_value("");
            }
        }
        current
    });

    let on_change = move |ev: web_sys::Event| {
        let input: web_sys::HtmlInputElement = event_target(&ev);
        if let Some(files) = input.files() {
            wasm_bindgen_futures::spawn_local(async move {
                // Group files by directory (report folder)
                let mut folder_files: HashMap<String, Vec<web_sys::File>> = HashMap::new();

                for i in 0..files.length() {
                    if let Some(file) = files.get(i) {
                        // Get the relative path which includes folder structure
                        let path = utils::get_file_relative_path(&file);
                        // Extract folder name (parent directory of the file)
                        if let Some(folder) = path.rsplit_once('/').map(|(dir, _)| dir.to_string())
                        {
                            folder_files.entry(folder).or_default().push(file);
                        }
                    }
                }

                let mut loaded_reports = Vec::new();

                // Process each folder
                for (_folder, files) in folder_files {
                    let mut report: Option<boar::Report> = None;
                    let mut cdf_html: HashMap<String, String> = HashMap::new();

                    for file in files {
                        let name = file.name();
                        if name == "report.json" {
                            if let Ok(text) = utils::read_file_as_text(&file).await
                                && let Ok(r) = serde_json::from_str::<boar::Report>(&text)
                            {
                                report = Some(r);
                            }
                        } else if name.starts_with("cdf_") && name.ends_with(".html") {
                            // Extract stat name from filename: cdf_download_duration.html -> DownloadDuration
                            if let Ok(html) = utils::read_file_as_text(&file).await {
                                // Get stat name from file, e.g., "cdf_download_duration.html"
                                let stat_name = name
                                    .trim_start_matches("cdf_")
                                    .trim_end_matches(".html")
                                    .to_string();
                                cdf_html.insert(stat_name, html);
                            }
                        }
                    }

                    if let Some(r) = report {
                        loaded_reports.push(ReportWithCdf {
                            report: r,
                            cdf_html,
                        });
                    }
                }

                // Update the report count
                let count = loaded_reports.len();
                set_report_count.set(Some(count));

                // Accumulate reports, deduplicating by UUID, then sort by delay ascending
                if !loaded_reports.is_empty() {
                    set_reports.update(|existing| {
                        for report_with_cdf in loaded_reports {
                            // Only add if UUID not already present
                            if !existing
                                .iter()
                                .any(|r| r.report.plan.uuid == report_with_cdf.report.plan.uuid)
                            {
                                existing.push(report_with_cdf);
                            }
                        }
                        // Sort by delay_ms in ascending order
                        existing.sort_by_key(|r| r.report.plan.network_setup.delay_ms);
                    });
                }
            });
        }
    };

    let count_text = move || match report_count.get() {
        Some(0) => "No reports found".to_string(),
        Some(1) => "1 report found".to_string(),
        Some(n) => format!("{} reports found", n),
        None => "No reports selected".to_string(),
    };

    view! {
        <div class="flex items-center gap-3">
            <label class="py-2 px-4 rounded text-sm font-semibold bg-blue-50 text-blue-700 hover:bg-blue-100 cursor-pointer">
                "Choose Reports"
                <input
                    type="file"
                    multiple=true
                    on:change=on_change
                    class="hidden"
                    node_ref=input_ref
                />
            </label>
            <span class="text-sm text-gray-600">{count_text}</span>
        </div>
    }
}

mod utils {
    /// Read a File as text using the File API
    pub async fn read_file_as_text(file: &web_sys::File) -> Result<String, wasm_bindgen::JsValue> {
        use js_sys::Uint8Array;
        use wasm_bindgen_futures::JsFuture;

        let array_buffer = JsFuture::from(file.array_buffer()).await?;
        let uint8_array = Uint8Array::new(&array_buffer);
        let vec = uint8_array.to_vec();
        String::from_utf8(vec).map_err(|e| wasm_bindgen::JsValue::from_str(&e.to_string()))
    }

    /// Get the relative path from a File using webkitRelativePath property
    pub fn get_file_relative_path(file: &web_sys::File) -> String {
        let file_ref: &js_sys::Object = file.as_ref();
        js_sys::Reflect::get(
            file_ref,
            &wasm_bindgen::JsValue::from_str("webkitRelativePath"),
        )
        .ok()
        .and_then(|v| v.as_string())
        .unwrap_or_else(|| file.name())
    }
}
