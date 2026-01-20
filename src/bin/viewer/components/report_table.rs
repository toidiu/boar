use humansize::{DECIMAL, format_size};
use leptos::prelude::*;
use std::collections::HashMap;

use super::{app::ReportWithCdf, cdf::CdfPreview};

#[component]
pub fn ReportTable(
    reports: ReadSignal<Vec<ReportWithCdf>>,
    set_reports: WriteSignal<Vec<ReportWithCdf>>,
    visible_fields: ReadSignal<Vec<String>>,
    dragging_index: ReadSignal<Option<usize>>,
    set_dragging_index: WriteSignal<Option<usize>>,
    drop_target_index: ReadSignal<Option<usize>>,
    set_drop_target_index: WriteSignal<Option<usize>>,
    _expanded_cdf: ReadSignal<Option<(uuid::Uuid, String)>>,
    set_expanded_cdf: WriteSignal<Option<(uuid::Uuid, String)>>,
    set_input_reset: WriteSignal<u32>,
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
                    let baseline_stats: HashMap<(String, String), f64> = {
                        let mut map = HashMap::new();
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
                                    set_input_reset=set_input_reset
                                />
                            }
                        })
                        .collect();

                    // First header row: config columns + stat names with colspan (+1 for CDF)
                    let stat_group_headers: Vec<_> = stat_names_clone
                        .iter()
                        .map(|name| {
                            let colspan = fields_clone.len() + 1; // +1 for CDF column
                            view! {
                                <th
                                    colspan=colspan
                                    class="px-4 py-3 text-center text-xs font-semibold text-gray-700 uppercase tracking-wider border-l-2 border-gray-300 bg-gray-100"
                                >
                                    {utils::pascal_to_display(name)}
                                </th>
                            }
                        })
                        .collect();

                    // Second header row: CDF column first, then field names for each stat
                    let field_headers: Vec<_> = stat_names_clone
                        .iter()
                        .flat_map(|_| {
                            // CDF header first (with left border for stat group)
                            let mut headers = vec![view! {
                                <th class="px-3 py-2 text-center text-xs font-medium text-gray-600 border-l-2 border-r border-gray-300 border-b border-gray-200 bg-gray-50">
                                    {"CDF".to_string()}
                                </th>
                            }];
                            // Then stat field headers
                            let field_headers: Vec<_> = fields_clone.iter().map(move |field| {
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
                                            <th rowspan=2 class="relative px-4 py-3 text-center text-xs font-semibold text-gray-600 uppercase tracking-wider bg-gray-100 min-w-56 sticky left-0 z-20 after:absolute after:top-0 after:right-0 after:bottom-0 after:w-4 after:-mr-4 after:pointer-events-none after:bg-gradient-to-r after:from-black/[0.08] after:to-transparent">
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
    baseline_stats: HashMap<(String, String), f64>,
    set_reports: WriteSignal<Vec<ReportWithCdf>>,
    dragging_index: ReadSignal<Option<usize>>,
    set_dragging_index: WriteSignal<Option<usize>>,
    drop_target_index: ReadSignal<Option<usize>>,
    set_drop_target_index: WriteSignal<Option<usize>>,
    set_expanded_cdf: WriteSignal<Option<(uuid::Uuid, String)>>,
    set_input_reset: WriteSignal<u32>,
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
        // Reset file input so the removed report can be reloaded
        set_input_reset.update(|n| *n += 1);
    };

    // Drag event handlers
    let on_drag_start = move |ev: web_sys::DragEvent| {
        set_dragging_index.set(Some(index));
        set_drop_target_index.set(None);
        // Set drag data (required for Firefox)
        if let Some(dt) = ev.data_transfer() {
            let _ = dt.set_data("text/plain", &index.to_string());
            dt.set_effect_allowed("move");

            // Use the entire row as the drag image, aligned to the drag icon
            if let Some(target) = ev.target() {
                use wasm_bindgen::JsCast;
                if let Some(drag_el) = target.dyn_ref::<web_sys::HtmlElement>() {
                    // Traverse up to find the <tr> element
                    let mut current: web_sys::Element = drag_el.clone().into();
                    while let Some(parent) = current.parent_element() {
                        if parent.tag_name().to_lowercase() == "tr" {
                            // Calculate offset: position of drag icon relative to row
                            let row_rect = parent.get_bounding_client_rect();
                            let icon_rect = drag_el.get_bounding_client_rect();
                            let offset_x = (icon_rect.left() - row_rect.left()
                                + icon_rect.width() / 2.0)
                                as i32;
                            let offset_y = (icon_rect.top() - row_rect.top()
                                + icon_rect.height() / 2.0)
                                as i32;

                            // Set the row as the drag image with proper offset
                            if let Some(html_el) = parent.dyn_ref::<web_sys::HtmlElement>() {
                                dt.set_drag_image(html_el, offset_x, offset_y);
                            }
                            break;
                        }
                        current = parent;
                    }
                }
            }
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
        if let Some(from_idx) = dragging_index.get()
            && from_idx != index
        {
            set_drop_target_index.set(Some(index));
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
        if let Some(from_index) = dragging_index.get()
            && from_index != index
        {
            set_reports.update(|reports| {
                if from_index < reports.len() && index < reports.len() {
                    let item = reports.remove(from_index);
                    reports.insert(index, item);
                }
            });
        }
        set_dragging_index.set(None);
        set_drop_target_index.set(None);
    };

    // Create stat cells for each stat name × visible field combination
    let report_uuid = report.plan.uuid;
    let stat_cells: Vec<_> =
        stat_names
            .iter()
            .flat_map(|name| {
                let stat = stats.iter().find(|s| &s.aggregate.name == name);
                let name_clone = name.clone();
                let baseline_stats_clone = baseline_stats.clone();
                let cdf_html_map_clone = cdf_html_map.clone();
                let set_expanded_cdf_clone = set_expanded_cdf;

                // Convert PascalCase stat name to snake_case for CDF lookup
                let cdf_key = utils::pascal_to_snake(name);
                let cdf_content = cdf_html_map_clone.get(&cdf_key).cloned();

                // CDF preview cell first (with left border for stat group)
                let cdf_bg = if is_baseline { "bg-blue-50" } else { "bg-white" };
                let cdf_cell = view! {
                    <td class={format!("px-2 py-1 whitespace-nowrap text-center border-l-2 border-r border-gray-300 {}", cdf_bg)}>
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
                let field_cells: Vec<_> = visible_fields.iter().map(move |field| {
                let value = stat
                    .map(|s| utils::get_stat_field(s, field))
                    .unwrap_or_else(|| "-".to_string());

                // Get current raw value for comparison
                let current_value = stat.and_then(|s| utils::get_stat_raw_value(s, field));

                // Get baseline value and compute comparison color
                let comparison_style = if is_baseline {
                    None // First row is the baseline, no comparison
                } else {
                    let baseline_key = (name_clone.clone(), field.clone());
                    baseline_stats_clone.get(&baseline_key).and_then(|&baseline| {
                        current_value.and_then(|current| {
                            utils::get_comparison_color(current, baseline, &name_clone)
                        })
                    })
                };

                let style = comparison_style
                    .map(|c| format!("background-color: {};", c))
                    .unwrap_or_default();

                let field_bg = if is_baseline { "bg-blue-50" } else { "bg-white" };
                view! {
                    <td class={format!("px-3 py-3 whitespace-nowrap text-sm text-gray-800 text-right font-mono border-r border-gray-200 {}", field_bg)} style=style>
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
        } else if is_baseline {
            // Baseline row - solid blue tint
            "border-b border-gray-200 bg-blue-50 transition-all duration-150"
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
            <td class={format!("relative px-3 py-2 text-sm text-gray-800 {} min-w-56 sticky left-0 z-10 after:absolute after:top-0 after:right-0 after:bottom-0 after:w-4 after:-mr-4 after:pointer-events-none after:bg-gradient-to-r after:from-black/[0.08] after:to-transparent", if is_baseline { "bg-blue-50" } else { "bg-white" })}>
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
                        <div><span class="text-gray-500">"Delay: "</span><span class="font-medium">{format!("{}ms", plan.network_setup.delay_ms)}</span></div>
                        <div><span class="text-gray-500">"Rate: "</span><span class="font-medium">{format!("{}mbit", plan.network_setup.rate_mbit)}</span></div>
                        <div><span class="text-gray-500">"Loss: "</span><span class="font-medium">{plan.network_setup.loss_model.clone()}</span></div>
                        <div class="mt-1 px-2 py-1.5 bg-gray-100 rounded border border-gray-200">
                            <div class="flex flex-col gap-0.5 text-xs">
                                <div><span class="text-gray-500">"Size: "</span><span class="font-medium">{format_size(plan.download_bytes.as_u64(), DECIMAL)}</span></div>
                                <div class="whitespace-nowrap"><span class="text-gray-500">"CCA: "</span><span class="font-medium">{plan.endpoint_setup.server_cca.clone()}</span></div>
                            </div>
                        </div>
                    </div>
                </div>
            </td>
            {stat_cells}
        </tr>
    }
}

#[component]
fn UuidCell(uuid: uuid::Uuid) -> impl IntoView {
    let full_uuid = uuid.to_string();
    let short_uuid = format!("{}...", &full_uuid[..5]);
    let full_uuid_for_click = full_uuid.clone();
    let full_uuid_for_title = full_uuid.clone();
    let bg_color = utils::uuid_to_color(uuid);

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

mod utils {
    /// Format a number with commas as thousand separators (max 3 decimal places)
    pub fn format_with_commas(n: f64) -> String {
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

    /// Convert PascalCase to display string with spaces (e.g., "DownloadDuration" -> "Download Duration")
    pub fn pascal_to_display(s: &str) -> String {
        let mut result = String::new();
        for (i, c) in s.chars().enumerate() {
            if c.is_uppercase() && i > 0 {
                result.push(' ');
            }
            result.push(c);
        }
        result
    }

    /// Generate a pastel background color from a UUID
    pub fn uuid_to_color(uuid: uuid::Uuid) -> String {
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
    pub fn get_comparison_color(current: f64, baseline: f64, stat_name: &str) -> Option<String> {
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

    /// Get the value of a specific field from AggregateStats
    pub fn get_stat_field(stat: &boar::StatsReport, field: &str) -> String {
        match field {
            "mean" => stat
                .aggregate
                .mean
                .map(format_with_commas)
                .unwrap_or_else(|| "-".to_string()),
            "median" => format_with_commas(stat.aggregate.median),
            "std_dev" => stat
                .aggregate
                .std_dev
                .map(format_with_commas)
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
    pub fn get_stat_raw_value(stat: &boar::StatsReport, field: &str) -> Option<f64> {
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

    /// Convert PascalCase to snake_case (e.g., "DownloadDuration" -> "download_duration")
    pub fn pascal_to_snake(s: &str) -> String {
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
}
