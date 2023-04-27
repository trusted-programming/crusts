use rust_hero::{
    query::{Invocation, QueryFormat},
    safe::SafeLanguageModel,
};
use std::fs;
use std::io::{BufRead, BufReader, Write};

use crate::utils::run_check_json_output;

/// runs curs and removes all unsafe marked by curs that are considered safe
/// after that runs cargo check and while it finds an error it will add the unsafe back for the function and check again
pub fn run() {
    let predictions: Vec<Prediction> = unsafe_detection(".")
        .iter()
        .map(|s| Prediction::from_str(s))
        .collect();

    for prediction in predictions {
        prediction.remove_unsafe();
    }
    let errors = run_check_json_output()
        .as_array()
        .unwrap()
        .iter()
        .filter(|msg| msg["message"]["level"].as_str().unwrap() == "warning")
        .count();
    let mut errors = vec![];
    while errors.len() > 1 {}
    unimplemented!()
}

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
        let file = fs::File::open(&self.file_path).expect("Failed to open file");
        let reader = BufReader::new(file);
        let mut lines: Vec<String> = reader.lines().map(|l| l.unwrap()).collect();

        if self.line > 0 && self.line <= lines.len() {
            if lines[self.line][self.col..].contains("unsafe") {
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

fn add_unsafe_to_lines(file_path: &str, line_numbers: Vec<usize>) -> std::io::Result<()> {
    let file = fs::File::open(file_path)?;
    let reader = BufReader::new(file);

    let mut lines: Vec<String> = reader.lines().map(|l| l.unwrap()).collect();

    for line_number in line_numbers {
        if let Some(line) = lines.get_mut(line_number - 1) {
            *line = format!("unsafe {}", line);
        }
    }

    let mut file = fs::File::create(file_path)?;
    for line in lines {
        writeln!(file, "{}", line)?;
    }

    Ok(())
}
