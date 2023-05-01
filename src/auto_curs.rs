use crate::utils::run_cargo_check_json_output;
use log::info;
use rust_hero::{
    query::{Invocation, QueryFormat},
    safe::SafeLanguageModel,
};
use std::fs;
use std::io::{BufRead, BufReader, Write};

/// runs curs and removes all unsafe marked by curs that are considered safe
/// after that runs cargo check and while it finds an error it will add the unsafe back for the function and check again
pub fn run() {
    info!("Starting Auto Curs");

    let predictions: Vec<Prediction> = unsafe_detection("src/main.rs")
        .iter()
        .map(|s| Prediction::from_str(s))
        .collect();

    for prediction in predictions {
        prediction.remove_unsafe();
    }

    for error in run_cargo_check_json_output().iter() {
        // Get the function name and file name from the error message
        info!("{}",error["message"]);
        let file_path = error["message"]["src_path"].as_str().unwrap();
        let line_start = error["message"]["spans"][0]["line_start"].as_u64().unwrap() as usize;
        let line_end = error["message"]["spans"][0]["line_start"].as_u64().unwrap() as usize;

        // Add the unsafe keyword to the function using the `add_unsafe_keyword` function from some_crate_name
         add_unsafe_keyword(file_path, line_start, line_end);
    }
}

// TODO: improve efficiency of this by doing all the function names at the same time
fn add_unsafe_keyword(file_path: &str, line_start: usize, line_end: usize) {
    info!("Adding unsafe keyword for file_path:{file_path} line_start:{line_start} line_end:{line_end}");

    let file = fs::File::open(file_path).unwrap();
    let reader = BufReader::new(file);

    let mut lines: Vec<String> = reader.lines().map(|l| l.unwrap()).collect();
    let code_to_wrap = lines[line_start - 1].clone();

    lines[line_start - 1] = format!("unsafe {{\n    {}", code_to_wrap);
    lines.insert(line_end, String::from("}"));

    let output = lines.join("\n");
    let mut file = fs::File::create(file_path).unwrap();
    file.write_all(output.as_bytes()).unwrap();
}

// FIXME: need to check if true or false otherwise it's useless
struct Prediction {
    pub file_path: String,
    pub line: usize,
    pub col: usize,
    pub safe: bool,
}

impl Prediction {
    fn from_str(s: &str) -> Self {
        let mut split = s.split(',');
        let file_path = split.next().unwrap().to_string();
        let line = split.next().unwrap().parse::<usize>().unwrap() - 1;
        let col = split.next().unwrap().parse::<usize>().unwrap() - 1;
        let safe = split.last().unwrap().parse::<bool>().unwrap();

        Self {
            file_path,
            line,
            col,
            safe
        }
    }

    fn remove_unsafe(&self) {
        if !self.safe {
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
