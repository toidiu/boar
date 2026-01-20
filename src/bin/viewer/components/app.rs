use leptos::prelude::*;
use std::collections::HashMap;

use super::{
    cdf::CdfModal, file_picker::FilePicker, report_table::ReportTable, settings::SettingsDropdown,
};

// All available aggregate stat fields
pub const ALL_FIELDS: &[&str] = &[
    "mean", "median", "std_dev", "p0", "p25", "p50", "p75", "p90", "p99", "p100", "trimean",
];

// Default visible fields
pub const DEFAULT_FIELDS: &[&str] = &["trimean", "p99"];

// localStorage key for column preferences
pub const STORAGE_KEY: &str = "boar_viewer_visible_fields";

/// A report with its associated CDF HTML content
#[derive(Clone)]
pub struct ReportWithCdf {
    pub report: boar::Report,
    /// Map from stat name to CDF HTML content
    pub cdf_html: HashMap<String, String>,
}

#[component]
pub fn App() -> impl IntoView {
    // Store loaded reports with CDF content
    let (reports, set_reports) = signal(Vec::<ReportWithCdf>::new());

    // Trigger to reset file input (incremented on clear all)
    let (clear_trigger, set_clear_trigger) = signal(0u32);

    // Trigger to reset file input only (incremented when × is clicked, keeps count)
    let (input_reset, set_input_reset) = signal(0u32);

    // Visible fields for stats columns - load from localStorage
    let (visible_fields, set_visible_fields) = signal(utils::load_visible_fields());

    // Persist visible_fields to localStorage whenever it changes
    Effect::new(move |_| {
        let fields = visible_fields.get();
        utils::save_visible_fields(&fields);
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
        set_clear_trigger.update(|n| *n += 1); // Trigger file input reset
    };

    view! {
        <div class="min-h-screen bg-gray-100 p-8">
            <h1 class="text-3xl font-bold text-gray-800 mb-8">"Boar Report Viewer"</h1>
            <div class="mb-8 flex items-center gap-4">
                <FilePicker set_reports=set_reports reports=reports clear_trigger=clear_trigger input_reset=input_reset />
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
                set_input_reset=set_input_reset
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

mod utils {
    use super::{ALL_FIELDS, DEFAULT_FIELDS, STORAGE_KEY};

    /// Load visible fields from localStorage, or return default
    pub fn load_visible_fields() -> Vec<String> {
        let window = web_sys::window().expect("no window");
        let storage = window.local_storage().ok().flatten();

        if let Some(storage) = storage
            && let Ok(Some(json)) = storage.get_item(STORAGE_KEY)
            && let Ok(fields) = serde_json::from_str::<Vec<String>>(&json)
        {
            // Validate fields are still valid (filter out any that no longer exist)
            let valid: Vec<String> = fields
                .into_iter()
                .filter(|f| ALL_FIELDS.contains(&f.as_str()))
                .collect();
            if !valid.is_empty() {
                return valid;
            }
        }

        // Return defaults if nothing saved or invalid
        DEFAULT_FIELDS.iter().map(|s| s.to_string()).collect()
    }

    /// Save visible fields to localStorage
    pub fn save_visible_fields(fields: &[String]) {
        let window = web_sys::window().expect("no window");
        let storage = window.local_storage().ok().flatten();

        if let Some(storage) = storage
            && let Ok(json) = serde_json::to_string(fields)
        {
            let _ = storage.set_item(STORAGE_KEY, &json);
        }
    }
}
