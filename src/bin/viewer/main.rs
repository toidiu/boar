use humansize::{DECIMAL, format_size};
use leptos::prelude::*;
use std::collections::HashMap;

// All available aggregate stat fields
const ALL_FIELDS: &[&str] = &[
    "mean", "median", "std_dev", "p0", "p25", "p50", "p75", "p90", "p99", "p100", "trimean",
];

// Default visible fields
const DEFAULT_FIELDS: &[&str] = &["trimean", "p99"];

// localStorage key for column preferences
const STORAGE_KEY: &str = "boar_viewer_visible_fields";

/// A report with its associated CDF HTML content
#[derive(Clone)]
struct ReportWithCdf {
    report: boar::Report,
    /// Map from stat name to CDF HTML content
    cdf_html: HashMap<String, String>,
}

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
}

/// Load visible fields from localStorage, or return default
fn load_visible_fields() -> Vec<String> {
    let window = web_sys::window().expect("no window");
    let storage = window.local_storage().ok().flatten();

    if let Some(storage) = storage {
        if let Ok(Some(json)) = storage.get_item(STORAGE_KEY) {
            if let Ok(fields) = serde_json::from_str::<Vec<String>>(&json) {
                // Validate fields are still valid (filter out any that no longer exist)
                let valid: Vec<String> = fields
                    .into_iter()
                    .filter(|f| ALL_FIELDS.contains(&f.as_str()))
                    .collect();
                if !valid.is_empty() {
                    return valid;
                }
            }
        }
    }

    // Return defaults if nothing saved or invalid
    DEFAULT_FIELDS.iter().map(|s| s.to_string()).collect()
}

/// Save visible fields to localStorage
fn save_visible_fields(fields: &[String]) {
    let window = web_sys::window().expect("no window");
    let storage = window.local_storage().ok().flatten();

    if let Some(storage) = storage {
        if let Ok(json) = serde_json::to_string(fields) {
            let _ = storage.set_item(STORAGE_KEY, &json);
        }
    }
}

#[component]
fn App() -> impl IntoView {
    // Store loaded reports with CDF content
    let (reports, set_reports) = signal(Vec::<ReportWithCdf>::new());

    // Visible fields for stats columns - load from localStorage
    let (visible_fields, set_visible_fields) = signal(load_visible_fields());

    // Persist visible_fields to localStorage whenever it changes
    Effect::new(move |_| {
        let fields = visible_fields.get();
        save_visible_fields(&fields);
    });

    // Settings modal visibility
    let (show_settings, set_show_settings) = signal(false);

    // Drag state for row reordering
    let (dragging_index, set_dragging_index) = signal(Option::<usize>::None);
    let (drop_target_index, set_drop_target_index) = signal(Option::<usize>::None);

    // Expanded CDF state: (report_uuid, stat_name)
    let (expanded_cdf, set_expanded_cdf) = signal(Option::<(uuid::Uuid, String)>::None);

    // Global Escape key handler for closing modals/dropdowns
    Effect::new(move |_| {
        use wasm_bindgen::{JsCast, prelude::*};

        let window = web_sys::window().expect("no window");
        let closure =
            Closure::<dyn Fn(web_sys::KeyboardEvent)>::new(move |ev: web_sys::KeyboardEvent| {
                if ev.key() == "Escape" {
                    // Close CDF modal first (higher priority), then settings dropdown
                    if expanded_cdf.get().is_some() {
                        set_expanded_cdf.set(None);
                    } else if show_settings.get() {
                        set_show_settings.set(false);
                    }
                }
            });

        let _ =
            window.add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref());
        closure.forget(); // Leak the closure to keep it alive
    });

    let has_reports = move || !reports.read().is_empty();

    let on_clear_all = move |_| {
        set_reports.set(Vec::new());
    };

    view! {
        <div class="min-h-screen bg-gray-100 p-8">
            <h1 class="text-3xl font-bold text-gray-800 mb-8">"Boar Report Viewer"</h1>
            <div class="mb-8 flex items-center gap-4">
                <FilePicker set_reports=set_reports />
                <button
                    on:click=on_clear_all
                    disabled=move || !has_reports()
                    class=move || {
                        if has_reports() {
                            "px-4 py-2 bg-red-500 text-white rounded hover:bg-red-600"
                        } else {
                            "px-4 py-2 bg-gray-300 text-gray-500 rounded cursor-not-allowed"
                        }
                    }
                >
                    "Clear All"
                </button>
                <SettingsDropdown
                    visible_fields=visible_fields
                    set_visible_fields=set_visible_fields
                    show_settings=show_settings
                    set_show_settings=set_show_settings
                />
            </div>
            <ReportTable
                reports=reports
                set_reports=set_reports
                visible_fields=visible_fields
                dragging_index=dragging_index
                set_dragging_index=set_dragging_index
                drop_target_index=drop_target_index
                set_drop_target_index=set_drop_target_index
                _expanded_cdf=expanded_cdf
                set_expanded_cdf=set_expanded_cdf
            />

            // CDF Modal (expanded view)
            {move || {
                if let Some((report_uuid, stat_name)) = expanded_cdf.get() {
                    // Find the CDF HTML content
                    let cdf_html = reports.read()
                        .iter()
                        .find(|r| r.report.plan.uuid == report_uuid)
                        .and_then(|r| r.cdf_html.get(&stat_name).cloned());

                    if let Some(html_content) = cdf_html {
                        view! {
                            <CdfModal
                                stat_name=stat_name
                                html_content=html_content
                                set_expanded_cdf=set_expanded_cdf
                            />
                        }.into_any()
                    } else {
                        view! { <div></div> }.into_any()
                    }
                } else {
                    view! { <div></div> }.into_any()
                }
            }}
        </div>
    }
}

#[component]
fn SettingsDropdown(
    visible_fields: ReadSignal<Vec<String>>,
    set_visible_fields: WriteSignal<Vec<String>>,
    show_settings: ReadSignal<bool>,
    set_show_settings: WriteSignal<bool>,
) -> impl IntoView {
    let toggle = move |_| {
        set_show_settings.update(|v| *v = !*v);
    };

    let select_all = move |_| {
        set_visible_fields.set(ALL_FIELDS.iter().map(|s| s.to_string()).collect());
    };

    let deselect_all = move |_| {
        set_visible_fields.set(Vec::new());
    };

    view! {
        <div class="relative">
            <button
                on:click=toggle
                class="px-4 py-2 bg-gray-500 text-white rounded hover:bg-gray-600 flex items-center gap-2"
            >
                "Settings"
                <span class="text-xs">{move || if show_settings.get() { "▲" } else { "▼" }}</span>
            </button>

            {move || {
                if show_settings.get() {
                    view! {
                        <div class="absolute top-full right-0 mt-1 bg-white rounded-lg shadow-lg border border-gray-200 py-2 w-48 z-50">
                            <div class="px-3 py-1 text-xs font-semibold text-gray-500 uppercase tracking-wider border-b border-gray-100 mb-1">
                                "Visible Columns"
                            </div>
                            <div class="flex gap-2 px-3 py-1.5 border-b border-gray-100 mb-1">
                                <button
                                    on:click=select_all
                                    class="text-xs text-blue-600 hover:text-blue-800 hover:underline"
                                >
                                    "All"
                                </button>
                                <span class="text-gray-300">"|"</span>
                                <button
                                    on:click=deselect_all
                                    class="text-xs text-blue-600 hover:text-blue-800 hover:underline"
                                >
                                    "None"
                                </button>
                            </div>
                            {ALL_FIELDS
                                .iter()
                                .map(|&field| {
                                    let field_str = field.to_string();
                                    let field_for_check = field_str.clone();
                                    let field_for_change = field_str.clone();

                                    let is_checked = move || {
                                        visible_fields.read().contains(&field_for_check)
                                    };

                                    let on_change = move |_| {
                                        let field = field_for_change.clone();
                                        set_visible_fields.update(|fields| {
                                            if fields.contains(&field) {
                                                fields.retain(|f| f != &field);
                                            } else {
                                                // Add in the order defined in ALL_FIELDS
                                                let mut new_fields = Vec::new();
                                                for &f in ALL_FIELDS {
                                                    let f_str = f.to_string();
                                                    if fields.contains(&f_str) || f_str == field {
                                                        new_fields.push(f_str);
                                                    }
                                                }
                                                *fields = new_fields;
                                            }
                                        });
                                    };

                                    view! {
                                        <label class="flex items-center gap-2 cursor-pointer hover:bg-gray-100 px-3 py-1.5">
                                            <input
                                                type="checkbox"
                                                checked=is_checked
                                                on:change=on_change
                                                class="w-4 h-4 text-blue-600 rounded border-gray-300 focus:ring-blue-500"
                                            />
                                            <span class="text-sm text-gray-700">{field_str}</span>
                                        </label>
                                    }
                                })
                                .collect::<Vec<_>>()}
                        </div>
                    }.into_any()
                } else {
                    view! { <div></div> }.into_any()
                }
            }}
        </div>
    }
}

#[component]
fn FilePicker(set_reports: WriteSignal<Vec<ReportWithCdf>>) -> impl IntoView {
    use leptos::html::Input;

    let input_ref: NodeRef<Input> = NodeRef::new();

    // Set webkitdirectory attribute after mount
    Effect::new(move |_| {
        if let Some(input) = input_ref.get() {
            let el: &web_sys::Element = input.as_ref();
            let _ = el.set_attribute("webkitdirectory", "true");
        }
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
                        let path = get_file_relative_path(&file);
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
                            if let Ok(text) = read_file_as_text(&file).await {
                                if let Ok(r) = serde_json::from_str::<boar::Report>(&text) {
                                    report = Some(r);
                                }
                            }
                        } else if name.starts_with("cdf_") && name.ends_with(".html") {
                            // Extract stat name from filename: cdf_download_duration.html -> DownloadDuration
                            if let Ok(html) = read_file_as_text(&file).await {
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

    view! {
        <div>
            <label class="block text-sm font-medium text-gray-700 mb-2">
                "Select a folder containing report(s)"
            </label>
            <input
                type="file"
                multiple=true
                on:change=on_change
                class="block w-full text-sm text-gray-500
                       file:mr-4 file:py-2 file:px-4
                       file:rounded file:border-0
                       file:text-sm file:font-semibold
                       file:bg-blue-50 file:text-blue-700
                       hover:file:bg-blue-100
                       cursor-pointer"
                node_ref=input_ref
            />
        </div>
    }
}

#[component]
fn ReportTable(
    reports: ReadSignal<Vec<ReportWithCdf>>,
    set_reports: WriteSignal<Vec<ReportWithCdf>>,
    visible_fields: ReadSignal<Vec<String>>,
    dragging_index: ReadSignal<Option<usize>>,
    set_dragging_index: WriteSignal<Option<usize>>,
    drop_target_index: ReadSignal<Option<usize>>,
    set_drop_target_index: WriteSignal<Option<usize>>,
    _expanded_cdf: ReadSignal<Option<(uuid::Uuid, String)>>,
    set_expanded_cdf: WriteSignal<Option<(uuid::Uuid, String)>>,
) -> impl IntoView {
    view! {
        <div>
            {move || {
                let reports_vec = reports.read();
                let fields = visible_fields.get();

                if reports_vec.is_empty() {
                    view! {
                        <div class="text-gray-500 italic">
                            "No reports loaded. Select a report folder above."
                        </div>
                    }.into_any()
                } else {
                    // Collect all unique stat names across all reports
                    let stat_names: Vec<String> = {
                        let mut names = Vec::new();
                        for rwc in reports_vec.iter() {
                            for stat in &rwc.report.stat_report {
                                if !names.contains(&stat.aggregate.name) {
                                    names.push(stat.aggregate.name.clone());
                                }
                            }
                        }
                        names
                    };

                    // Extract baseline stats from first report (for comparison coloring)
                    let baseline_stats: std::collections::HashMap<(String, String), f64> = {
                        let mut map = std::collections::HashMap::new();
                        if let Some(first_rwc) = reports_vec.first() {
                            for stat in &first_rwc.report.stat_report {
                                let name = &stat.aggregate.name;
                                // Store all field values for this stat
                                if let Some(mean) = stat.aggregate.mean {
                                    map.insert((name.clone(), "mean".to_string()), mean);
                                }
                                map.insert((name.clone(), "median".to_string()), stat.aggregate.median);
                                if let Some(std_dev) = stat.aggregate.std_dev {
                                    map.insert((name.clone(), "std_dev".to_string()), std_dev);
                                }
                                map.insert((name.clone(), "p0".to_string()), stat.aggregate.p0);
                                map.insert((name.clone(), "p25".to_string()), stat.aggregate.p25);
                                map.insert((name.clone(), "p50".to_string()), stat.aggregate.p50);
                                map.insert((name.clone(), "p75".to_string()), stat.aggregate.p75);
                                map.insert((name.clone(), "p90".to_string()), stat.aggregate.p90);
                                map.insert((name.clone(), "p99".to_string()), stat.aggregate.p99);
                                map.insert((name.clone(), "p100".to_string()), stat.aggregate.p100);
                                map.insert((name.clone(), "trimean".to_string()), stat.aggregate.trimean);
                            }
                        }
                        map
                    };

                    let fields_clone = fields.clone();
                    let stat_names_clone = stat_names.clone();
                    let num_reports = reports_vec.len();

                    let rows: Vec<_> = reports_vec
                        .iter()
                        .enumerate()
                        .map(|(index, rwc)| {
                            let rwc = rwc.clone();
                            let stat_names = stat_names.clone();
                            let fields = fields.clone();
                            let baseline = baseline_stats.clone();
                            view! {
                                <ReportRow
                                    report_with_cdf=rwc
                                    index=index
                                    total_count=num_reports
                                    stat_names=stat_names
                                    visible_fields=fields
                                    baseline_stats=baseline
                                    set_reports=set_reports
                                    dragging_index=dragging_index
                                    set_dragging_index=set_dragging_index
                                    drop_target_index=drop_target_index
                                    set_drop_target_index=set_drop_target_index
                                    set_expanded_cdf=set_expanded_cdf
                                />
                            }
                        })
                        .collect();

                    // First header row: config columns + stat names with colspan (+1 for CDF)
                    let stat_group_headers: Vec<_> = stat_names_clone
                        .iter()
                        .enumerate()
                        .map(|(_, name)| {
                            let colspan = fields_clone.len() + 1; // +1 for CDF column
                            view! {
                                <th
                                    colspan=colspan
                                    class="px-4 py-3 text-center text-xs font-semibold text-gray-700 uppercase tracking-wider border-l-2 border-gray-300 bg-gray-100"
                                >
                                    {name.clone()}
                                </th>
                            }
                        })
                        .collect();

                    // Second header row: CDF column first, then field names for each stat
                    let field_headers: Vec<_> = stat_names_clone
                        .iter()
                        .enumerate()
                        .flat_map(|(_, _)| {
                            // CDF header first (with left border for stat group)
                            let mut headers = vec![view! {
                                <th class="px-3 py-2 text-center text-xs font-medium text-gray-600 border-l-2 border-gray-300 border-b border-gray-200 bg-gray-50">
                                    {"CDF".to_string()}
                                </th>
                            }];
                            // Then stat field headers
                            let field_headers: Vec<_> = fields_clone.iter().enumerate().map(move |(_, field)| {
                                view! {
                                    <th class="px-3 py-2 text-center text-xs font-medium text-gray-600 border-r border-gray-200 border-b border-gray-200 bg-gray-50">
                                        {field.clone()}
                                    </th>
                                }
                            }).collect();
                            headers.extend(field_headers);
                            headers
                        })
                        .collect();

                    view! {
                        <div class="relative">
                            <div class="overflow-x-auto bg-white rounded-lg shadow-lg border border-gray-300">
                                <table class="min-w-full border-collapse">
                                    <thead>
                                        <tr class="bg-gray-100 border-b-2 border-gray-300">
                                            <th rowspan=2 class="px-4 py-3 text-center text-xs font-semibold text-gray-600 uppercase tracking-wider bg-gray-100 min-w-56 sticky left-0 z-20">
                                                "Setup"
                                            </th>
                                            {stat_group_headers}
                                        </tr>
                                        <tr class="border-b border-gray-300">
                                            {field_headers}
                                        </tr>
                                    </thead>
                                    <tbody>
                                        {rows}
                                    </tbody>
                                </table>
                            </div>
                            // Gradient shadow overlay on the right edge of frozen column
                            <div
                                class="absolute top-0 bottom-0 w-4 pointer-events-none z-30"
                                style="left: 14rem; background: linear-gradient(to right, rgba(0,0,0,0.08), transparent);"
                            ></div>
                        </div>
                    }.into_any()
                }
            }}
        </div>
    }
}

#[component]
fn ReportRow(
    report_with_cdf: ReportWithCdf,
    index: usize,
    #[allow(unused)] total_count: usize,
    stat_names: Vec<String>,
    visible_fields: Vec<String>,
    baseline_stats: std::collections::HashMap<(String, String), f64>,
    set_reports: WriteSignal<Vec<ReportWithCdf>>,
    dragging_index: ReadSignal<Option<usize>>,
    set_dragging_index: WriteSignal<Option<usize>>,
    drop_target_index: ReadSignal<Option<usize>>,
    set_drop_target_index: WriteSignal<Option<usize>>,
    set_expanded_cdf: WriteSignal<Option<(uuid::Uuid, String)>>,
) -> impl IntoView {
    let is_baseline = index == 0;
    let report = report_with_cdf.report.clone();
    let cdf_html_map = report_with_cdf.cdf_html.clone();
    let uuid = report.plan.uuid;
    let plan = report.plan.clone();
    let stats = report.stat_report.clone();

    let on_remove = move |_| {
        set_reports.update(|reports| {
            reports.retain(|r| r.report.plan.uuid != uuid);
        });
    };

    // Drag event handlers
    let on_drag_start = move |ev: web_sys::DragEvent| {
        set_dragging_index.set(Some(index));
        set_drop_target_index.set(None);
        // Set drag data (required for Firefox)
        if let Some(dt) = ev.data_transfer() {
            let _ = dt.set_data("text/plain", &index.to_string());
            dt.set_effect_allowed("move");
        }
    };

    let on_drag_end = move |_| {
        set_dragging_index.set(None);
        set_drop_target_index.set(None);
    };

    let on_drag_over = move |ev: web_sys::DragEvent| {
        ev.prevent_default();
        if let Some(dt) = ev.data_transfer() {
            dt.set_drop_effect("move");
        }
        // Only set drop target if we're dragging a different row
        if let Some(from_idx) = dragging_index.get() {
            if from_idx != index {
                set_drop_target_index.set(Some(index));
            }
        }
    };

    let on_drag_leave = move |_| {
        // Only clear if this row was the target
        if drop_target_index.get() == Some(index) {
            set_drop_target_index.set(None);
        }
    };

    let on_drop = move |ev: web_sys::DragEvent| {
        ev.prevent_default();
        if let Some(from_index) = dragging_index.get() {
            if from_index != index {
                set_reports.update(|reports| {
                    if from_index < reports.len() && index < reports.len() {
                        let item = reports.remove(from_index);
                        reports.insert(index, item);
                    }
                });
            }
        }
        set_dragging_index.set(None);
        set_drop_target_index.set(None);
    };

    // Create stat cells for each stat name × visible field combination
    let report_uuid = report.plan.uuid;
    let stat_cells: Vec<_> =
        stat_names
            .iter()
            .enumerate()
            .flat_map(|(_, name)| {
                let stat = stats.iter().find(|s| &s.aggregate.name == name);
                let name_clone = name.clone();
                let baseline_stats_clone = baseline_stats.clone();
                let cdf_html_map_clone = cdf_html_map.clone();
                let set_expanded_cdf_clone = set_expanded_cdf;

                // Convert PascalCase stat name to snake_case for CDF lookup
                let cdf_key = pascal_to_snake(name);
                let cdf_content = cdf_html_map_clone.get(&cdf_key).cloned();

                // CDF preview cell first (with left border for stat group)
                let cdf_cell = view! {
                    <td class="px-2 py-1 whitespace-nowrap text-center border-l-2 border-gray-300 bg-white">
                        <CdfPreview
                            cdf_content=cdf_content.clone()
                            stat_name=cdf_key.clone()
                            report_uuid=report_uuid
                            set_expanded_cdf=set_expanded_cdf_clone
                        />
                    </td>
                }.into_any();

                let mut cells = vec![cdf_cell];

                // Then stat field cells
                let field_cells: Vec<_> = visible_fields.iter().enumerate().map(move |(_, field)| {
                let value = stat
                    .map(|s| get_stat_field(s, field))
                    .unwrap_or_else(|| "-".to_string());

                // Get current raw value for comparison
                let current_value = stat.and_then(|s| get_stat_raw_value(s, field));

                // Get baseline value and compute comparison color
                let comparison_style = if is_baseline {
                    None // First row is the baseline, no comparison
                } else {
                    let baseline_key = (name_clone.clone(), field.clone());
                    baseline_stats_clone.get(&baseline_key).and_then(|&baseline| {
                        current_value.and_then(|current| {
                            get_comparison_color(current, baseline, &name_clone)
                        })
                    })
                };

                let style = comparison_style
                    .map(|c| format!("background-color: {};", c))
                    .unwrap_or_default();

                view! {
                    <td class="px-3 py-3 whitespace-nowrap text-sm text-gray-800 text-right font-mono border-r border-gray-200 bg-white" style=style>
                        {value}
                    </td>
                }.into_any()
            }).collect();

                cells.extend(field_cells);
                cells
            })
            .collect();

    // Determine row styling based on drag state
    let row_class = move || {
        let is_dragging = dragging_index.get() == Some(index);
        let is_drop_target = drop_target_index.get() == Some(index);
        let dragging_from = dragging_index.get();

        if is_dragging {
            // The row being dragged
            "border-b border-gray-200 opacity-40 bg-gray-300 scale-[0.98] transition-all duration-150"
        } else if is_drop_target {
            // Show where the row will be inserted
            if let Some(from_idx) = dragging_from {
                if from_idx > index {
                    // Dragging from below - show indicator at top
                    "border-t-4 border-t-blue-500 border-b border-gray-200 bg-blue-50 transition-all duration-150"
                } else {
                    // Dragging from above - show indicator at bottom
                    "border-b-4 border-b-blue-500 bg-blue-50 transition-all duration-150"
                }
            } else {
                "border-b border-gray-200 hover:bg-gray-50"
            }
        } else {
            "border-b border-gray-200 hover:bg-gray-50 transition-all duration-150"
        }
    };

    view! {
        <tr
            class=row_class
            on:dragover=on_drag_over
            on:dragleave=on_drag_leave
            on:drop=on_drop
        >
            <td class="px-3 py-2 text-sm text-gray-800 bg-white min-w-56 sticky left-0 z-10">
                <div class="flex items-stretch gap-2">
                    <div class="flex flex-col items-center">
                        <button
                            on:click=on_remove
                            class="w-5 h-5 flex items-center justify-center text-gray-400 hover:text-red-500 hover:bg-red-50 rounded text-sm"
                            title="Remove report"
                        >
                            "×"
                        </button>
                        <div class="flex-1 flex items-center">
                            <span
                                class="text-gray-400 hover:text-gray-600 cursor-grab select-none"
                                style="font-size: 24px; line-height: 1;"
                                title="Drag to reorder"
                                draggable="true"
                                on:dragstart=on_drag_start
                                on:dragend=on_drag_end
                            >"⠿"</span>
                        </div>
                    </div>
                    <div class="flex flex-col gap-1">
                        <UuidCell uuid=uuid />
                        <div><span class="text-gray-500">"Size: "</span><span class="font-medium">{format_size(plan.download_bytes.as_u64(), DECIMAL)}</span></div>
                        <div class="whitespace-nowrap"><span class="text-gray-500">"CCA: "</span><span class="font-medium">{plan.endpoint_setup.server_cca.clone()}</span></div>
                        <div class="mt-1 px-2 py-1.5 bg-gray-100 rounded border border-gray-200">
                            <div class="flex flex-col gap-0.5 text-xs">
                                <div><span class="text-gray-500">"Delay: "</span><span class="font-medium">{format!("{}ms", plan.network_setup.delay_ms)}</span></div>
                                <div><span class="text-gray-500">"Rate: "</span><span class="font-medium">{format!("{}mbit", plan.network_setup.rate_mbit)}</span></div>
                                <div><span class="text-gray-500">"Loss: "</span><span>{plan.network_setup.loss_model.clone()}</span></div>
                            </div>
                        </div>
                    </div>
                </div>
            </td>
            {stat_cells}
        </tr>
    }
}

/// Generate a pastel background color from a UUID
fn uuid_to_color(uuid: uuid::Uuid) -> String {
    let bytes = uuid.as_bytes();
    // Use first 3 bytes for RGB, but make them pastel by mixing with white
    // Pastel = high lightness, moderate saturation
    let r = 180 + (bytes[0] % 60); // 180-239
    let g = 180 + (bytes[1] % 60); // 180-239
    let b = 180 + (bytes[2] % 60); // 180-239
    format!("rgb({}, {}, {})", r, g, b)
}

/// Returns true if lower values are better for this stat
fn is_lower_better(stat_name: &str) -> bool {
    matches!(stat_name, "DownloadDuration")
}

/// Calculate comparison color based on current vs baseline value
/// Returns CSS background color string with appropriate tint and intensity
fn get_comparison_color(current: f64, baseline: f64, stat_name: &str) -> Option<String> {
    if baseline == 0.0 || current == baseline {
        return None; // No comparison possible or same value
    }

    let lower_is_better = is_lower_better(stat_name);
    let pct_diff = ((current - baseline) / baseline.abs()) * 100.0;

    // Determine if this is better or worse
    let is_better = if lower_is_better {
        current < baseline
    } else {
        current > baseline
    };

    // Calculate intensity based on percentage difference (cap at 50% for full intensity)
    let intensity = (pct_diff.abs() / 50.0).min(1.0) * 0.4; // Max opacity 0.4

    if is_better {
        // Green tint for better values
        Some(format!("rgba(34, 197, 94, {:.2})", intensity))
    } else {
        // Red tint for worse values
        Some(format!("rgba(239, 68, 68, {:.2})", intensity))
    }
}

/// Format a number with commas as thousand separators (max 3 decimal places)
fn format_with_commas(n: f64) -> String {
    // Round to max 3 decimal places, then remove trailing zeros
    let rounded = (n * 1000.0).round() / 1000.0;
    let s = format!("{}", rounded);

    if let Some(dot_pos) = s.find('.') {
        let int_part = &s[..dot_pos];
        let dec_part = &s[dot_pos..]; // includes the dot
        // Trim trailing zeros from decimal part, but keep at least one digit after dot if there's a decimal
        let trimmed_dec: String = dec_part.trim_end_matches('0').to_string();
        let final_dec = if trimmed_dec == "." { "" } else { &trimmed_dec };
        format!("{}{}", add_commas_to_int(int_part), final_dec)
    } else {
        add_commas_to_int(&s)
    }
}

/// Add commas to an integer string
fn add_commas_to_int(s: &str) -> String {
    let negative = s.starts_with('-');
    let digits: String = if negative {
        s[1..].to_string()
    } else {
        s.to_string()
    };

    let mut result = String::new();
    for (i, c) in digits.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }

    let formatted: String = result.chars().rev().collect();
    if negative {
        format!("-{}", formatted)
    } else {
        formatted
    }
}

/// Get the value of a specific field from AggregateStats
fn get_stat_field(stat: &boar::StatsReport, field: &str) -> String {
    match field {
        "mean" => stat
            .aggregate
            .mean
            .map(|v| format_with_commas(v))
            .unwrap_or_else(|| "-".to_string()),
        "median" => format_with_commas(stat.aggregate.median),
        "std_dev" => stat
            .aggregate
            .std_dev
            .map(|v| format_with_commas(v))
            .unwrap_or_else(|| "-".to_string()),
        "p0" => format_with_commas(stat.aggregate.p0),
        "p25" => format_with_commas(stat.aggregate.p25),
        "p50" => format_with_commas(stat.aggregate.p50),
        "p75" => format_with_commas(stat.aggregate.p75),
        "p90" => format_with_commas(stat.aggregate.p90),
        "p99" => format_with_commas(stat.aggregate.p99),
        "p100" => format_with_commas(stat.aggregate.p100),
        "trimean" => format_with_commas(stat.aggregate.trimean),
        _ => "-".to_string(),
    }
}

/// Get the raw numeric value of a specific field from AggregateStats (for comparison)
fn get_stat_raw_value(stat: &boar::StatsReport, field: &str) -> Option<f64> {
    match field {
        "mean" => stat.aggregate.mean,
        "median" => Some(stat.aggregate.median),
        "std_dev" => stat.aggregate.std_dev,
        "p0" => Some(stat.aggregate.p0),
        "p25" => Some(stat.aggregate.p25),
        "p50" => Some(stat.aggregate.p50),
        "p75" => Some(stat.aggregate.p75),
        "p90" => Some(stat.aggregate.p90),
        "p99" => Some(stat.aggregate.p99),
        "p100" => Some(stat.aggregate.p100),
        "trimean" => Some(stat.aggregate.trimean),
        _ => None,
    }
}

#[component]
fn UuidCell(uuid: uuid::Uuid) -> impl IntoView {
    let full_uuid = uuid.to_string();
    let short_uuid = format!("{}...", &full_uuid[..5]);
    let full_uuid_for_click = full_uuid.clone();
    let full_uuid_for_title = full_uuid.clone();
    let bg_color = uuid_to_color(uuid);

    let (copied, set_copied) = signal(false);

    let on_click = move |_| {
        let uuid_str = full_uuid_for_click.clone();
        wasm_bindgen_futures::spawn_local(async move {
            if let Some(window) = web_sys::window() {
                let clipboard = window.navigator().clipboard();
                let _ = wasm_bindgen_futures::JsFuture::from(clipboard.write_text(&uuid_str)).await;
                set_copied.set(true);
                // Reset after 1.5 seconds
                let _ = wasm_bindgen_futures::JsFuture::from(js_sys::Promise::new(
                    &mut |resolve, _| {
                        let _ = window
                            .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, 1500);
                    },
                ))
                .await;
                set_copied.set(false);
            }
        });
    };

    let style = format!("background-color: {};", bg_color);

    view! {
        <div
            class="flex items-center gap-1 cursor-pointer"
            title=full_uuid_for_title
            on:click=on_click
        >
            <span
                class="px-1.5 py-0.5 rounded font-mono text-xs text-gray-700 hover:brightness-95 transition-all"
                style=style
            >
                {move || if copied.get() { "Copied!".to_string() } else { short_uuid.clone() }}
            </span>
            {move || if !copied.get() {
                view! {
                    <svg class="w-3 h-3 text-gray-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                              d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
                    </svg>
                }.into_any()
            } else {
                view! { <span></span> }.into_any()
            }}
        </div>
    }
}

async fn read_file_as_text(file: &web_sys::File) -> Result<String, wasm_bindgen::JsValue> {
    use js_sys::Uint8Array;
    use wasm_bindgen_futures::JsFuture;

    let array_buffer = JsFuture::from(file.array_buffer()).await?;
    let uint8_array = Uint8Array::new(&array_buffer);
    let vec = uint8_array.to_vec();
    String::from_utf8(vec).map_err(|e| wasm_bindgen::JsValue::from_str(&e.to_string()))
}

/// Get the relative path from a File using webkitRelativePath property
fn get_file_relative_path(file: &web_sys::File) -> String {
    let file_ref: &js_sys::Object = file.as_ref();
    js_sys::Reflect::get(
        file_ref,
        &wasm_bindgen::JsValue::from_str("webkitRelativePath"),
    )
    .ok()
    .and_then(|v| v.as_string())
    .unwrap_or_else(|| file.name())
}

/// Convert PascalCase to snake_case (e.g., "DownloadDuration" -> "download_duration")
fn pascal_to_snake(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(c.to_ascii_lowercase());
        } else {
            result.push(c);
        }
    }
    result
}

#[component]
fn CdfPreview(
    cdf_content: Option<String>,
    stat_name: String,
    report_uuid: uuid::Uuid,
    set_expanded_cdf: WriteSignal<Option<(uuid::Uuid, String)>>,
) -> impl IntoView {
    match cdf_content {
        Some(html) => {
            let stat_name_for_click = stat_name.clone();
            let on_click = move |_| {
                set_expanded_cdf.set(Some((report_uuid, stat_name_for_click.clone())));
            };

            view! {
                <div
                    class="w-20 h-14 cursor-pointer hover:ring-2 hover:ring-blue-400 rounded overflow-hidden bg-gray-50"
                    title="Click to expand CDF"
                    on:click=on_click
                >
                    <iframe
                        srcdoc=html
                        class="w-full h-full pointer-events-none border-0"
                        style="transform: scale(0.15); transform-origin: top left; width: 666%; height: 666%;"
                        sandbox="allow-scripts"
                    />
                </div>
            }.into_any()
        }
        None => {
            view! {
                <div class="w-20 h-14 flex items-center justify-center text-gray-300 text-xs bg-gray-50 rounded">
                    "No CDF"
                </div>
            }.into_any()
        }
    }
}

#[component]
fn CdfModal(
    stat_name: String,
    html_content: String,
    set_expanded_cdf: WriteSignal<Option<(uuid::Uuid, String)>>,
) -> impl IntoView {
    let on_close = move |_| {
        set_expanded_cdf.set(None);
    };

    // Close on escape key
    let on_keydown = move |ev: web_sys::KeyboardEvent| {
        if ev.key() == "Escape" {
            set_expanded_cdf.set(None);
        }
    };

    // Convert snake_case back to readable name for display
    let display_name = stat_name.replace('_', " ");
    let display_name = display_name
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().chain(chars).collect::<String>(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ");

    view! {
        <div
            class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
            on:click=on_close
            on:keydown=on_keydown
            tabindex="-1"
        >
            <div
                class="bg-white rounded-lg shadow-2xl w-[90vw] h-[85vh] flex flex-col"
                on:click=move |ev| ev.stop_propagation()
            >
                <div class="flex items-center justify-between px-6 py-4 border-b border-gray-200">
                    <h2 class="text-xl font-semibold text-gray-800">
                        {format!("CDF: {}", display_name)}
                    </h2>
                    <button
                        on:click=on_close
                        class="w-8 h-8 flex items-center justify-center text-gray-400 hover:text-gray-600 hover:bg-gray-100 rounded-full text-xl"
                        title="Close (Esc)"
                    >
                        "×"
                    </button>
                </div>
                <div class="flex-1 p-4 overflow-hidden">
                    <iframe
                        srcdoc=html_content
                        class="w-full h-full border-0 rounded"
                        sandbox="allow-scripts"
                    />
                </div>
            </div>
        </div>
    }
}
