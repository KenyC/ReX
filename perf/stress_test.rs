/// This executable parses a very large number of formulas 
/// Its performance is monitored with `perf` to find out the main
/// cost centers of the `parser` module
///
/// Takes path to a .json file representing a list of strings (as below) 
/// and parse each of them ; prints number of successes
///
/// ```json
/// [
///   "",
///   "X(3823)",
///   "\\texttt{TERM}_T",
///   "\\mathop{\\mathrm{Pic}}(\\mathcal{O}_{\\Delta_0 d'^2})",
///   "m_\\alpha n_\\alpha = 3",
///   "\\sqrt{s^g/N} \\sqrt{s^m/N} \\ll N^{-1/2} \\Leftrightarrow s^g s^m \\ll N",
///   ...
///  ]
///  ```
///
///
/// Run with:
/// 
/// ```bash
/// CARGO_PROFILE_RELEASE_DEBUG=true cargo flamegraph --root --example stress-test -- PATH_TO_FILE
/// ```

use rex::parser::parse;


fn main() {
	let mut n_successes = 0;
	let mut n_compiles  = 0;

	let fomulas_file_path = std::env::args().nth(1).expect("Usage: stess-test PATH");
	eprintln!("Formulas from: {}", fomulas_file_path);

	let formulas_file = std::fs::File::open(&fomulas_file_path).unwrap();
	let formulas_buffer = std::io::BufReader::new(formulas_file);
	let formulas : Vec<String> = serde_json::from_reader(formulas_buffer).unwrap();

	for formula in formulas.iter() {
		if let Ok(_) = parse(formula) {
			n_successes += 1;
		}
		n_compiles += 1;
	}
	eprintln!("{} / {}", n_successes, n_compiles);
}