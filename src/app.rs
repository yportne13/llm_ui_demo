use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::{to_value, from_value};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::{EventTarget, HtmlSelectElement};
use yew::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "tauri"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "tauri"], js_name = invoke)]
    async fn invoke_0(cmd: &str) -> JsValue;
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "event"])]
    pub async fn listen(event: &str, handler: &Closure<dyn FnMut(JsValue)>);
}

#[derive(Serialize, Deserialize)]
struct GreetArgs<'a> {
    s: &'a str,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
struct ChooseArgs<'a> {
    path: &'a str,
    modeltype: &'a str,
}

#[function_component(App)]
pub fn app() -> Html {
    let model_type = use_state(|| "llama".to_owned());
    let model_list = vec!["llama", "bloom", "gpt2", "gptj", "neox"];

    let models = use_state(Vec::<String>::new);
    {
        let models = models.clone();
        spawn_local(async move {
            let new_models = from_value(invoke_0("get_file_list").await).unwrap();
            models.set(new_models);
        })
    }


    let greet_msg = use_state(String::new);
    let msg_answer = use_state(|| Some(String::new()));
    let msg_clone = msg_answer.clone();
    spawn_local(async move {
        #[derive(Debug, Serialize, Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct TauriEvent {
            event: String,
            window_label: String,
            payload: Option<String>,
            id: f64,
        }
        let closure = Closure::<dyn FnMut(JsValue)>::new(move |x: JsValue| {
            let tauri_event: TauriEvent = serde_wasm_bindgen::from_value(x).unwrap();//TODO:do not unwrap
            //console::log_1(&format!("callback: {:?}", tauri_event).into());
            msg_clone.set(tauri_event.payload);
            //match tauri_event.payload {
            //    Some(p) => msg_clone.set(format!("{}{}", *msg_clone, p)),
            //    None => msg_clone.set(format!("{}\n", *msg_clone)),
            //}
        });
        listen("answer", &closure).await;
        closure.forget();
    });
    let greet_msg_clone = greet_msg.clone();
    use_effect_with_deps(move |x| {
        match &**x {
            Some(s) => greet_msg_clone.set(format!("{}{}", *greet_msg_clone, s)),
            None => greet_msg_clone.set(format!("{}\n", *greet_msg_clone)),
        }
    }, msg_answer);


    let greet_input_ref = use_node_ref();

    let name = use_state(String::new);
    {
        let name = name.clone();
        let name2 = name.clone();
        use_effect_with_deps(
            move |_| {
                spawn_local(async move {
                    if name.is_empty() {
                        return;
                    }

                    let args = to_value(&GreetArgs { s: &name }).unwrap();
                    // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
                    let _ = invoke("speak", args).await;
                    //greet_msg.set(new_msg);
                });

                || {}
            },
            name2,
        );
    }

    let greet = {
        let name = name;
        let greet_input_ref = greet_input_ref.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            name.set(
                greet_input_ref
                    .cast::<web_sys::HtmlInputElement>()
                    .unwrap()
                    .value(),
            );
        })
    };

    html! {
        <main>// class="container"
            <div style="width:20%;float:left;height:100%;overflow-y:auto">
                <select class="select" onchange={
                    let t = model_type.clone();
                    let list = model_list.clone();
                    Callback::from(move |event: Event| {
                        let target: EventTarget = event
                            .target()
                            .expect("I'm sure this event has a target!");

                        // maybe the target is a select element?
                        if let Some(select_element) = target.dyn_ref::<HtmlSelectElement>() {
                            let idx = select_element.selected_index() as usize;
                            t.set(list[idx].to_string());
                        }
                    })}>
                    {
                        for model_list.iter()
                            .map(|s| html! {
                                <option value={*s} selected={&*model_type==s}>{s}</option>
                            })
                    }
                </select>
                {
                    for (*models).iter()
                        .map(|s| html!(<button class="model" onclick={
                            let m = s.clone();
                            let t = model_type.clone();
                            Callback::from(move |_| {
                                let args = to_value(&ChooseArgs{
                                        path: &m,
                                        modeltype: &t,
                                    }).unwrap();
                                spawn_local(async {
                                    let _ = invoke("choose_model",
                                        args
                                    ).await;
                                })
                            })
                        }>{s}</button>))
                }
            </div>

            <div style="width:80%;float:left;display:block;height:100%;overflow-y:auto">
                <div style="display:block;height:80%;overflow-y:auto">
                    <em>{ &*greet_msg }</em>
                </div>
                <div style="display:block;height:20%;overflow-y:auto">
                    <form class="row" onsubmit={greet}>
                        <input id="greet-input" ref={greet_input_ref} placeholder="Enter a question..." />
                        <button type="submit">{"Greet"}</button>
                    </form>
                </div>
            </div>
        </main>
    }
}
