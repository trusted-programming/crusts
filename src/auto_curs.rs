use crate::utils::{process_files_with_ext, run_cargo_check_json_output};
use cargo_metadata::diagnostic::DiagnosticLevel;
use log::info;
use rust_hero::{
    query::{Invocation, QueryFormat},
    safe::SafeLanguageModel,
};
use std::collections::HashSet;
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
    process_files_with_ext("rs", |file| {
        let predictions = unsafe_detection(&file);
        let removed_predictions = remove_unsafe_predictions(predictions);

        if !removed_predictions.is_empty() {
            readd_required_unsafe_keywords(&file, &removed_predictions);
        }
    });
}

fn remove_unsafe_predictions(predictions: Vec<Prediction>) -> Vec<(usize, String)> {
    predictions
        .into_iter()
        .filter(|p| p.prediction && !p.actual)
        .filter_map(|p| p.remove_unsafe())
        .collect()
}

// FIXME: improve efficiency of this by doing all the function names at the same time
fn add_unsafe_keyword(file_path: &str, line: String, line_number: usize) {
    info!("Adding unsafe keyword for at line number:{line_number}");

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
        let parts: Vec<&str> = s.split(',').collect();
        let file_path = parts[0].to_string();
        let line = parts[1].parse::<usize>().unwrap() - 1;
        let col = parts[2].parse::<usize>().unwrap() - 1;
        let prediction = !parts[5].contains("Unsafe");
        let actual = parts[6].parse::<bool>().unwrap();

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
            for line in &lines {
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

fn readd_required_unsafe_keywords(file: &str, removed_predictions: &[(usize, String)]) {
    let mut readded_lines = HashSet::new();

    let error_spans = run_cargo_check_json_output()
        .into_iter()
        .filter(|m| m.message.level == DiagnosticLevel::Error)
        .flat_map(|compiler_message| compiler_message.message.spans);

    for diagnostic_span in error_spans {
        let file_name = if diagnostic_span.file_name.starts_with("/rustc") {
            diagnostic_span
                .expansion
                .as_ref()
                .expect("expected some")
                .span
                .file_name
                .to_string()
        } else {
            diagnostic_span.file_name.to_string()
        };

        let canonical_path = canonicalize(&file_name).unwrap();
        let file_path_canonical = canonicalize(&file).unwrap();

        if canonical_path != file_path_canonical {
            continue;
        }

        let closest_removed_prediction = removed_predictions
            .iter()
            .filter(|(line, _)| *line <= diagnostic_span.line_start)
            .min_by_key(|(line, _)| diagnostic_span.line_start - line);

        if let Some((line, removed_text)) = closest_removed_prediction {
            if readded_lines.insert(*line) {
                add_unsafe_keyword(
                    canonical_path.to_str().unwrap(),
                    removed_text.to_string(),
                    *line,
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Seek, Write};
    use tempfile::NamedTempFile;

    #[test]
    fn test_add_unsafe_keyword() {
        // Create a temporary file with some content
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "fn some_function() {{}}").unwrap();

        // Run add_unsafe_keyword on the temporary file
        let file_path = temp_file.path().to_str().unwrap();
        add_unsafe_keyword(file_path, "unsafe fn some_function() {{}}".to_string(), 0);

        // Read the modified content of the temporary file
        let mut modified_content = String::new();
        temp_file.seek(std::io::SeekFrom::Start(0)).unwrap();
        temp_file.read_to_string(&mut modified_content).unwrap();

        // Verify if the unsafe keyword was added correctly
        assert_eq!(modified_content, "unsafe fn some_function() {{}}");
    }

    // Test case 1: Add unsafe keyword to a function that doesn't have it
    #[test]
    fn test_add_unsafe_keyword_no_unsafe() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "fn some_function() {{}}").unwrap();

        let file_path = temp_file.path().to_str().unwrap();
        add_unsafe_keyword(file_path, "unsafe fn some_function() {{}}".to_string(), 0);

        let mut modified_content = String::new();
        temp_file.seek(std::io::SeekFrom::Start(0)).unwrap();
        temp_file.read_to_string(&mut modified_content).unwrap();

        assert_eq!(modified_content, "unsafe fn some_function() {{}}");
    }

    // Test case 2: Don't modify a function that already has the unsafe keyword
    #[test]
    fn test_add_unsafe_keyword_already_unsafe() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "unsafe fn another_function() {{}}").unwrap();

        let file_path = temp_file.path().to_str().unwrap();
        add_unsafe_keyword(
            file_path,
            "unsafe fn another_function() {{}}".to_string(),
            0,
        );

        let mut modified_content = String::new();
        temp_file.seek(std::io::SeekFrom::Start(0)).unwrap();
        temp_file.read_to_string(&mut modified_content).unwrap();

        assert_eq!(modified_content, "unsafe fn another_function() {}\n");
    }
}
