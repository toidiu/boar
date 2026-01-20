use leptos::prelude::*;

use super::app::ALL_FIELDS;

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

    let close = move |_| {
        set_show_settings.set(false);
    };

    view! {
        <div class="relative">
            <button
                on:click=toggle
                class="p-2 bg-white text-gray-600 hover:bg-gray-50 rounded-lg border border-gray-300 shadow-sm"
                title="Settings"
            >
                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                          d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                          d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                </svg>
            </button>

            {move || {
                if show_settings.get() {
                    view! {
                        // Invisible overlay to catch clicks outside
                        <div
                            class="fixed inset-0 z-40"
                            on:click=close
                        ></div>
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
