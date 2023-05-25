// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{sync::Mutex, convert::Infallible, io::Write};

use llm::{ModelParameters, Model, LoadError, KnownModel, InferenceSession, InferenceRequest, InferenceResponse, InferenceFeedback};
use tauri::Window;

struct State {
    //path: std::path::PathBuf,
    model: Mutex<Option<Box<dyn Model>>>,
    session: Mutex<Option<InferenceSession>>,
}

#[derive(Clone, serde::Serialize)]
struct Payload(Option<String>);

#[tauri::command]
fn get_file_list() -> Vec<String> {
    let path = std::env::current_dir().unwrap();
    let paths = std::fs::read_dir(&path).unwrap();

    let mut ret = paths.filter_map(|entry| {
        entry.ok().and_then(|e|
        e.path().file_name()
        .and_then(|n| n.to_str())
        .and_then(|n| {
            let metadata = std::fs::metadata(path.join(n)).unwrap();
            if metadata.is_file() {
                Some(String::from(n))
            }else {
                None
            }
        })
    )
    }).collect::<Vec<String>>();
    ret.sort();
    ret
}

#[tauri::command]
fn choose_model(state: tauri::State<State>, path: &std::path::Path, modeltype: &str) {
    let params = ModelParameters::default();

    println!("get path: {path:?}");

    fn load_model<M: KnownModel + 'static>(path: &std::path::Path, params: ModelParameters) -> Result<Box<dyn Model>, LoadError> {
        let model = llm::load::<M>(path, params, None, |_| {println!("loading")})
            .map(Box::new);
        Ok(model?)
    }

    let model = match modeltype {
        "llama" => load_model::<llm::models::Llama>(path, params),
        "bloom" => load_model::<llm::models::Bloom>(path, params),
        "gpt2"  => load_model::<llm::models::Gpt2>(path, params),
        "gptj"  => load_model::<llm::models::GptJ>(path, params),
        _       => load_model::<llm::models::NeoX>(path, params),
        //"gptneox" => handle_args::<llm::models::GptNeoX>(args, None),
        //_ => handle_args::<llm::models::Mpt>(args, None),
    };

    println!("load finish");

    //if let Ok(model) = model {
    //    let session = model.start_session(Default::default());
    //    *state.model.lock().unwrap() = Some(model);
    //    *state.session.lock().unwrap() = Some(session);
    //}
    match model {
        Ok(model) => {
            let session = model.start_session(Default::default());
            *state.model.lock().unwrap() = Some(model);
            *state.session.lock().unwrap() = Some(session);
        }
        Err(e) => println!("{e}")
    }
}

#[tauri::command]
async fn speak(window: Window, state: tauri::State<'_, State>, s: String) -> Result<(), ()> {
    let mut session_guard = state.session.lock().unwrap();
    let session = session_guard.as_mut().unwrap();
    let model_guard = state.model.lock().unwrap();
    let model = model_guard.as_ref().unwrap();
    session.infer::<Infallible>(
            model.as_ref(),
            &mut rand::thread_rng(),
            &InferenceRequest {
                prompt: &s,
                ..Default::default()
            },
            &mut Default::default(),
            |s| match s {
                InferenceResponse::PromptToken(s) | InferenceResponse::InferredToken(s) => {
                    window.emit("answer", Payload(Some(s.clone()))).unwrap();
                    print!("{s}");
                    std::io::stdout().flush().unwrap();
                    Ok(InferenceFeedback::Continue)
                }
                _ => {Ok(InferenceFeedback::Continue)}
            }).unwrap();

    window.emit("answer", Payload(None)).unwrap();
    println!();
    Ok(())
}

fn main() {
    tauri::Builder::default()
        .manage(State{
            model: None.into(),
            session: None.into(),
        })
        .invoke_handler(tauri::generate_handler![
            get_file_list,
            choose_model,
            speak])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
