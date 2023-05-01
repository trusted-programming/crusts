use crate::utils::run_cargo_check_json_output;
use log::info;
use quote::quote;
use rust_hero::{
    query::{Invocation, QueryFormat},
    safe::SafeLanguageModel,
};
use std::fs;
use std::io::{BufRead, BufReader, Write};
use syn::{self, parse_file, File};

/// runs curs and removes all unsafe marked by curs that are considered safe
/// after that runs cargo check and while it finds an error it will add the unsafe back for the function and check again
pub fn run() {
    info!("Starting Auto Curs");

    let predictions: Vec<Prediction> = unsafe_detection(".")
        .iter()
        .map(|s| Prediction::from_str(s))
        .collect();

    for prediction in predictions {
        prediction.remove_unsafe();
    }

    for error in run_cargo_check_json_output()
        .as_array()
        .unwrap()
        .iter()
        .filter(|msg| msg["message"]["level"].as_str().unwrap() == "error")
    {
        // Get the function name and file name from the error message
        let function_name = error["message"]["spans"][0]["label"].as_str().unwrap();
        let file_name = error["message"]["spans"][0]["file_name"].as_str().unwrap();

        // Add the unsafe keyword to the function using the `add_unsafe_keyword` function from some_crate_name
        add_unsafe_keyword(file_name, function_name);
    }
}

// TODO: improve efficiency of this by doing all the function names at the same time
fn add_unsafe_keyword(file_name: &str, function_name: &str) {
    info!("Adding unsafe keyword for file_name:{file_name} function_name:{function_name}");

    // Read the contents of the file into a string
    let contents = fs::read_to_string(file_name).expect("failed to read file");

    // Parse the file into a syntax tree
    let syn_file: File = parse_file(&contents).expect("failed to parse file");

    // Find the function declaration and add the `unsafe` keyword
    let mut items = Vec::new();
    for item in &syn_file.items {
        if let syn::Item::Fn(ref function) = item {
            if function.sig.ident == function_name {
                let mut new_function: syn::ItemFn = function.clone();
                new_function.sig.unsafety = Some(syn::token::Unsafe::default());
                items.push(syn::Item::Fn(new_function));
            } else {
                items.push(item.clone());
            }
        } else {
            items.push(item.clone());
        }
    }

    // Generate the modified code and write it back to the file
    let new_contents = quote! {
        #(#items)*
    }
    .to_string();
    fs::write(file_name, new_contents).expect("failed to write to file");
}

// FIXME: need to check if true or false otherwise it's useless
struct Prediction {
    pub file_path: String,
    pub line: usize,
    pub col: usize,
}

impl Prediction {
    fn from_str(s: &str) -> Self {
        let mut split = s.split(',');
        let file_path = split.next().unwrap().to_string();
        let line = split.next().unwrap().parse::<usize>().unwrap() - 1;
        let col = split.next().unwrap().parse::<usize>().unwrap() - 1;

        Self {
            file_path,
            line,
            col,
        }
    }

    fn remove_unsafe(&self) {
        info!("removing unsafe according to curs prediction");
        let file = fs::File::open(&self.file_path).expect("Failed to open file");
        let reader = BufReader::new(file);
        let mut lines: Vec<String> = reader.lines().map(|l| l.unwrap()).collect();

        if self.line > 0
            && self.line <= lines.len()
            && lines[self.line][self.col..].contains("unsafe")
        {
            lines[self.line] = lines[self.line].replacen("unsafe", "", 1);
            let mut file = fs::File::create(&self.file_path).expect("Failed to create file");
            for line in lines {
                writeln!(file, "{}", line).expect("Failed to write to file");
            }
        }
    }
}

fn unsafe_detection(file_path: &str) -> Vec<String> {
    info!("running curs for unsafe detection");
    let args = vec!["rust_hero".to_string(), file_path.to_string()];
    let invocation = Invocation::from_args(args).unwrap();
    if let Invocation::DoQuery(query_opts) = invocation {
        let safe_model = SafeLanguageModel::new(query_opts).unwrap();
        if let QueryFormat::Classes = safe_model.get_opt().format {
            safe_model
                .predict()
                .expect("couldn't perform the prediction")
        } else {
            panic!("Unsupported {:?}", safe_model.get_opt().format);
        }
    } else {
        panic!("Unsupported invocation");
    }
}
