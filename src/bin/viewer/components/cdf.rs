use leptos::prelude::*;

#[component]
pub fn CdfPreview(
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
                    class="w-40 h-28 cursor-pointer hover:ring-2 hover:ring-blue-400 rounded overflow-hidden bg-gray-50"
                    title="Click to expand CDF"
                    on:click=on_click
                >
                    <iframe
                        srcdoc=html
                        class="w-full h-full pointer-events-none border-0"
                        style="transform: scale(0.25); transform-origin: top left; width: 400%; height: 400%;"
                        sandbox="allow-scripts"
                    />
                </div>
            }
            .into_any()
        }
        None => {
            view! {
                <div class="w-40 h-28 flex items-center justify-center text-gray-300 text-xs bg-gray-50 rounded">
                    "No CDF"
                </div>
            }
            .into_any()
        }
    }
}

#[component]
pub fn CdfModal(
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
    let display_name = utils::snake_to_title(&stat_name);

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

mod utils {
    /// Convert snake_case to Title Case (e.g., "download_duration" -> "Download Duration")
    pub fn snake_to_title(s: &str) -> String {
        s.replace('_', " ")
            .split_whitespace()
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    Some(first) => first.to_uppercase().chain(chars).collect::<String>(),
                    None => String::new(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}
