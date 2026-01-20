use leptos::prelude::*;

use super::app::{ALL_FIELDS, DEFAULT_FIELDS, STORAGE_KEY};

#[component]
pub fn SettingsDropdown(
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
