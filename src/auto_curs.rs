use rust_hero::{
    query::{Invocation, QueryFormat},
    safe::SafeLanguageModel,
};

/// runs curs and removes all unsafe marked by curs that are considered safe
/// after that runs cargo check and while it finds an error it will add the unsafe back for the function and check again
pub fn run() {
    let to_remove_vec: Vec<Prediction> = unsafe_detection(".");
}

fn unsafe_detection(file_path: &str) -> Vec<String> {
    let args = vec!["rust_hero".to_string(), file_path.to_string()];
    let invocation = Invocation::from_args(args).unwrap();
    if let Invocation::DoQuery(query_opts) = invocation {
        let safe_model = SafeLanguageModel::new(query_opts).unwrap();
        if let QueryFormat::Classes = safe_model.get_opt().format {
            let output = safe_model
                .predict()
                .expect("couldn't perform the prediction");
            output
        } else {
            panic!("Unsupported {:?}", safe_model.get_opt().format);
        }
    } else {
        panic!("Unsupported invocation");
    }
}

struct Prediction {
    pub file: String,
    pub line: u32,
    pub col: u32,
    pub text: String,
    pub class: String,
    pub probability: f32,
}
