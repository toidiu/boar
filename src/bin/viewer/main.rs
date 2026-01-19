use leptos::prelude::*;

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
}

#[component]
fn App() -> impl IntoView {
    // Store loaded reports
    let (reports, set_reports) = signal(Vec::<boar::Report>::new());

    view! {
        <div class="min-h-screen bg-gray-100 p-8">
            <h1 class="text-3xl font-bold text-gray-800 mb-8">"Boar Report Viewer"</h1>
            <FilePicker set_reports=set_reports />
            <ReportList reports=reports />
        </div>
    }
}

#[component]
fn FilePicker(set_reports: WriteSignal<Vec<boar::Report>>) -> impl IntoView {
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
                let mut loaded_reports = Vec::new();

                for i in 0..files.length() {
                    if let Some(file) = files.get(i) {
                        // Only process report.json files
                        if file.name() == "report.json" {
                            if let Ok(text) = read_file_as_text(&file).await {
                                if let Ok(report) = serde_json::from_str::<boar::Report>(&text) {
                                    loaded_reports.push(report);
                                }
                            }
                        }
                    }
                }

                // Only update if we found at least one report
                if !loaded_reports.is_empty() {
                    set_reports.set(loaded_reports);
                }
            });
        }
    };

    view! {
        <div class="mb-8">
            <label class="block text-sm font-medium text-gray-700 mb-2">
                "Select report folder(s)"
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
fn ReportList(reports: ReadSignal<Vec<boar::Report>>) -> impl IntoView {
    view! {
        <div>
            {move || {
                let reports_vec = reports.read();
                if reports_vec.is_empty() {
                    view! {
                        <div class="text-gray-500 italic">
                            "No reports loaded. Select a report folder above."
                        </div>
                    }.into_any()
                } else {
                    let cards: Vec<_> = reports_vec
                        .iter()
                        .map(|report| {
                            let report = report.clone();
                            view! { <ReportCard report=report /> }
                        })
                        .collect();
                    view! {
                        <div class="grid gap-4">
                            {cards}
                        </div>
                    }.into_any()
                }
            }}
        </div>
    }
}

#[component]
fn ReportCard(report: boar::Report) -> impl IntoView {
    let plan = report.plan.clone();
    let stats = report.stat_report.clone();

    view! {
        <div class="bg-white rounded-lg shadow p-6">
            <div class="flex justify-between items-start mb-4">
                <h2 class="text-lg font-semibold text-gray-800">
                    {format!("Report: {}", plan.uuid)}
                </h2>
            </div>

            <div class="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm">
                <div>
                    <span class="text-gray-500">"Delay: "</span>
                    <span class="font-medium">{format!("{}ms", plan.network_setup.delay_ms)}</span>
                </div>
                <div>
                    <span class="text-gray-500">"Rate: "</span>
                    <span class="font-medium">{format!("{}mbit", plan.network_setup.rate_mbit)}</span>
                </div>
                <div>
                    <span class="text-gray-500">"Loss: "</span>
                    <span class="font-medium">{plan.network_setup.loss_model.clone()}</span>
                </div>
                <div>
                    <span class="text-gray-500">"CCA: "</span>
                    <span class="font-medium">{plan.endpoint_setup.server_cca.clone()}</span>
                </div>
            </div>

            <div class="mt-4 pt-4 border-t border-gray-200">
                <h3 class="text-sm font-medium text-gray-700 mb-2">"Statistics"</h3>
                <div class="grid gap-2">
                    {stats
                        .iter()
                        .map(|stat| {
                            let name = stat.aggregate.name.clone();
                            let median = stat.aggregate.median;
                            view! {
                                <div class="text-sm">
                                    <span class="text-gray-500">{name}": "</span>
                                    <span class="font-medium">
                                        {format!("median={:.3}", median)}
                                    </span>
                                </div>
                            }
                        })
                        .collect::<Vec<_>>()}
                </div>
            </div>
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
