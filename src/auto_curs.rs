use crate::utils::{is_file_with_ext, run_cargo_check_json_output};
use cargo_metadata::diagnostic::DiagnosticLevel;
use jwalk::WalkDir;
use log::info;
use rust_hero::{
    query::{Invocation, QueryFormat},
    safe::SafeLanguageModel,
};
use std::fs;
use std::{
    fs::canonicalize,
    io::{BufRead, BufReader, Write},
};

// FIXME: ignore build.rs files

/// runs curs and removes all unsafe marked by curs that are considered safe
/// after that runs cargo check and while it finds an error it will add the unsafe back for the function and check again
pub fn run() {
    info!("Starting Auto Curs");
    WalkDir::new(".")
        .sort(true)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| is_file_with_ext(&e.path(), "rs"))
        .for_each(|e: jwalk::DirEntry<((), ())>| {
            let path = e.path();
            let file = path.to_string_lossy().to_string();

            let predictions = unsafe_detection(&file);
            let mut removed_predictions = Vec::new();

            for prediction in predictions {
                if prediction.prediction && !prediction.actual {
                    let removed = prediction.remove_unsafe();
                    if let Some(r) = removed {
                        removed_predictions.push(r);
                    }
                }
            }

            if removed_predictions.len() > 0 {
                for compiler_message in run_cargo_check_json_output()
                    .iter()
                    .filter(|m| m.message.level == DiagnosticLevel::Error)
                {
                    for diagnostic_span in &compiler_message.message.spans {
                        let mut file_name = (&diagnostic_span).file_name.to_string();

                        info!("file_name: {}", file_name);
                        if file_name.starts_with("/rustc") {
                            let expansion =
                                diagnostic_span.to_owned().expansion.expect("expected some");

                            file_name = expansion.span.file_name.to_string();
                        }
                        let canonical_path = canonicalize(&file_name).unwrap();
                        let file_path_canonical = canonicalize(&file).unwrap();
                        if canonical_path.to_str().unwrap() == file_path_canonical.to_str().unwrap()
                        {
                            let mut diff = (usize::MAX, None);

                            for removed_prediction in &removed_predictions {
                                if removed_prediction.0 > diagnostic_span.line_start {
                                    continue;
                                }
                                let new_diff = diagnostic_span.line_start - removed_prediction.0;
                                if new_diff < diff.0 {
                                    diff = (new_diff, Some(removed_prediction));
                                }
                            }

                            add_unsafe_keyword(
                                canonical_path.to_str().unwrap(),
                                diff.1.unwrap().1.to_string(),
                                diff.1.unwrap().0,
                            );
                        }
                    }
                }
            }
        });
}

// FIXME: improve efficiency of this by doing all the function names at the same time
fn add_unsafe_keyword(file_path: &str, line: String, line_number: usize) {
    info!("Adding unsafe keyword for file_path:{file_path} line:{line} line_number:{line_number}");

    let file = fs::File::open(file_path).unwrap();
    let reader = BufReader::new(file);

    let mut lines: Vec<String> = reader.lines().map(|l| l.unwrap()).collect();
    if lines[line_number].contains("unsafe") {
        return;
    }
    lines[line_number] = line;

    let output = lines.join("\n");
    let mut file = fs::File::create(file_path).unwrap();
    file.write_all(output.as_bytes()).unwrap();
}

#[derive(Debug, Clone)]
struct Prediction {
    pub file_path: String,
    pub line: usize,
    pub col: usize,
    pub prediction: bool,
    pub actual: bool,
}

impl Prediction {
    fn from_str(s: &str) -> Self {
        let mut split = s.split(',');
        let file_path = split.next().unwrap().to_string();
        let line = split.next().unwrap().parse::<usize>().unwrap() - 1;
        let col = split.next().unwrap().parse::<usize>().unwrap() - 1;
        let _end_line = split.next().unwrap();
        let _end_col = split.next().unwrap();
        let prediction = !split.next().unwrap().contains("Unsafe");
        let actual = split.next().unwrap().parse::<bool>().unwrap();

        Self {
            file_path,
            line,
            col,
            prediction,
            actual,
        }
    }

    fn remove_unsafe(&self) -> Option<(usize, String)> {
        let file = fs::File::open(&self.file_path).expect("Failed to open file");
        let reader = BufReader::new(file);
        let mut lines: Vec<String> = reader.lines().map(|l| l.unwrap()).collect();
        let to_be_removed = lines[self.line][self.col..].to_owned();
        if lines[self.line][self.col..].contains("unsafe") {
            lines[self.line] = lines[self.line].replacen("unsafe", "", 1);
            let mut file = fs::File::create(&self.file_path).expect("Failed to create file");
            for line in lines {
                writeln!(file, "{}", line).expect("Failed to write to file");
            }
            info!(
                "removed unsafe from file: {}, line {}, col {}",
                self.file_path, self.line, self.col
            );
            return Some((self.line, to_be_removed));
        }
        None
    }
}

fn unsafe_detection(file_path: &str) -> Vec<Prediction> {
    info!("running curs for unsafe detection file path: {file_path}");
    let args = vec!["rust_hero".to_string(), file_path.to_string()];
    let invocation = Invocation::from_args(args).unwrap();
    if let Invocation::DoQuery(query_opts) = invocation {
        let safe_model = SafeLanguageModel::new(query_opts).unwrap();
        if let QueryFormat::Classes = safe_model.get_opt().format {
            safe_model
                .predict()
                .expect("couldn't perform the prediction")
                .iter()
                .map(|s| Prediction::from_str(s))
                .collect()
        } else {
            panic!("Unsupported {:?}", safe_model.get_opt().format);
        }
    } else {
        panic!("Unsupported invocation");
    }
}
