use std::collections::HashMap;
use std::fs;
use std::process::{Command, exit};
use regex::Regex;
use colored::*;
use clap::{Parser, Subcommand};
use lazy_static::lazy_static;

#[derive(Parser)]
#[command(name = "Astra")]
#[command(author = "NEOAPPS")]
#[command(version = "1.1.0")]
#[command(about = "The Powerful Transpiled Programming Language", long_about = "The Powerful Transpiled Programming Language")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
	#[arg(value_name = "INPUT")]
    input: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
	/// Transpiles Astra Code into C++
    Transpile { input: String },
	/// Transpiles then Compiles Astra Code
    Compile { input: String },
}

struct AstraTranspiler {
    variables: HashMap<String, String>,
    functions: HashMap<String, FunctionInfo>,
    current_indent: usize,
    in_function: bool,
    in_raw_cpp: bool,
    raw_cpp_content: Vec<String>,
}

struct FunctionInfo {
    return_type: String,
    params: Vec<String>,
}

lazy_static! {
    static ref FN_REGEX: Regex = Regex::new(r"(pub |priv |prot |virt )?fn (\w+)\((.*?)\)(?: -> (\w+))?").unwrap();
    static ref RANGE_REGEX: Regex = Regex::new(r"for (\w+) in range\((\d+),\s*(\d+)\)").unwrap();
    static ref FOREACH_REGEX: Regex = Regex::new(r"for (\w+) in (\w+)").unwrap();
}

impl AstraTranspiler {
    fn new() -> Self {
        AstraTranspiler {
            variables: HashMap::new(),
            functions: HashMap::new(),
            current_indent: 0,
            in_function: false,
            in_raw_cpp: false,
            raw_cpp_content: Vec::new(),
        }
    }

fn transpile(&mut self, filename: &str) -> Option<String> {
    fs::read_to_string(filename).ok().map(|content| {
        let output_filename = filename.replace(".astra", ".cpp");
        let cpp_code = self.process_content(&content);

        if let Err(e) = fs::write(&output_filename, cpp_code) {
            println!("{} Error writing to file {}: {}", "✗".red(), output_filename.cyan(), e);
            exit(1);
        }

        println!("{} Transpiled {} to {}", "✓".green(), filename.cyan(), output_filename.cyan());
        output_filename
    })
}


    fn process_content(&mut self, content: &str) -> String {
        let mut cpp_lines = vec![
            "#include <iostream>".to_string(),
            "#define any auto".to_string(),
            "#define lib namespace".to_string(),
            "#define tn typename".to_string(),
            "#define pub public".to_string(),
            "#define priv private".to_string(),
            "#define prot protected".to_string(),
            "#define virt virtual".to_string(),
            "#define println print".to_string(),
            "#define blueprint(...) template <__VA_ARGS__>".to_string(),
            "#define null nullptr".to_string(),
            "#define mut const".to_string(),
            "#define match(val) switch(val)".to_string(),
            "#define case(val) case val:".to_string(),
            "#include <fstream>".to_string(),
            "#include <string>".to_string(),
            "#include <functional>".to_string(),
            "#include <memory>".to_string(),
            "#include <cmath>".to_string(),
            "#include <ctime>".to_string(),
            "#include <stdexcept>".to_string(),
            "#include <cstdio>".to_string(),
            "#include <filesystem>".to_string(),
            "#include <vector>".to_string(),
            "using namespace std;".to_string(),
            "void print(auto x) {cout << x << '\\n';}".to_string(),
            "void printn(auto x) {cout << x;}".to_string(),
            "int $(char* x) {return system(x);}".to_string(),
            "lib Astra {".to_string(),
            "class fs { pub: static void write(const std::string& p, const std::string& c) { std::ofstream(p) << c; } static std::string read(const std::string& p) { std::ifstream f(p); return {std::istreambuf_iterator<char>(f), {}}; } static void remove(const std::string& p) { std::filesystem::remove(p); } static std::vector<std::string> list(const std::string& d) { std::vector<std::string> fs; for (auto& e : std::filesystem::directory_iterator(d)) {fs.push_back(e.path().string());} return fs; } };".to_string(),
            "blueprint(tn T)".to_string(),
            "class DynamicArray {".to_string(),
            "pub:".to_string(),
            "T* arr;".to_string(),
            "size_t capacity, size;".to_string(),
            "void resize() { capacity *= 2; T* newArr = new T[capacity]; for (size_t i = 0; i < size; ++i) newArr[i] = arr[i]; delete[] arr; arr = newArr; }".to_string(),
            "DynamicArray(size_t initial_capacity = 2) : capacity(initial_capacity), size(0) { arr = new T[capacity]; }".to_string(),
            "~DynamicArray() { delete[] arr; }".to_string(),
            "void add(T value) { if (size == capacity) resize(); arr[size++] = value; }".to_string(),
            "T get(size_t index) const { if (index >= size) throw std::out_of_range(\"Index out of range\"); return arr[index]; }".to_string(),
            "size_t getSize() const { return size; }".to_string(),
            "void removeAt(size_t index) { if (index >= size) throw std::out_of_range(\"Index out of range\"); for (size_t i = index; i < size - 1; ++i) arr[i] = arr[i + 1]; --size; }".to_string(),
            "void print() const { for (size_t i = 0; i < size; ++i) std::cout << arr[i] << ' '; std::cout << std::endl; }".to_string(),
            "};".to_string(),
            "}".to_string(),
        ];

        let mut i = 0;
        let lines: Vec<&str> = content.split('\n').collect();
        while i < lines.len() {
            let line = lines[i].trim();
            
            if line.is_empty() {
                cpp_lines.push("".to_string());
                i += 1;
                continue;
            }

            if line == "::++ {" || line.starts_with("::++ {") {
                self.in_raw_cpp = true;
                i += 1;
                continue;
            } else if line == "}" && self.in_raw_cpp {
                self.in_raw_cpp = false;
                cpp_lines.extend(self.raw_cpp_content.drain(..));
                i += 1;
                continue;
            } else if self.in_raw_cpp {
                self.raw_cpp_content.push(line.to_string());
                i += 1;
                continue;
            }
            
            if line.starts_with("//") || line.starts_with("##") {
                i += 1;
                continue;
            }
            
            match line.split_once(' ') {
                Some(("use", rest)) => cpp_lines.push(format!("#include <{}>", rest.trim())),
                Some(("def", rest)) => cpp_lines.push(format!("#define {}", rest)),
				Some(("ifdef", rest)) => cpp_lines.push(format!("#ifdef {}", rest)),
				Some(("ifndef", rest)) => cpp_lines.push(format!("#ifndef {}", rest)),
				Some(("endif", rest)) => cpp_lines.push(format!("#endif {}", rest)),
				Some(("type", rest)) => cpp_lines.push(rest.replace("type ", "enum ") + if rest.ends_with(';') || rest.ends_with('{') { "" } else { ";" }),
                _ => {
                    if line.starts_with("fn") || line.starts_with("pub fn") || line.starts_with("priv fn") || line.starts_with("prot fn") || line.starts_with("virt fn") || line.starts_with("stat fn") {
                        if let Some(captures) = FN_REGEX.captures(line) {
                            let modifier = captures.get(1).map_or("", |m| m.as_str().trim());
                            let name = captures.get(2).unwrap().as_str();
                            let params = captures.get(3).map_or("", |m| m.as_str());
                            let return_type = captures.get(4).map_or("void", |m| m.as_str());

                            let param_list: Vec<String> = params.split(',')
                                .filter_map(|p| {
                                    let p = p.trim();
                                    if p.is_empty() { None } else {
                                        p.split_once(':').map(|(n, t)| format!("{} {}", t.trim(), n.trim()))
                                            .or_else(|| Some(format!("any {}", p)))
                                    }
                                })
                                .collect();

                            let access = match modifier {
                                "pub" => "pub:",
                                "priv" => "priv:",
                                "prot" => "prot:",
                                "virt" => "virtual",
								"stat" => "static",
                                _ => "",
                            };
							
                            let sig = format!("{}{} {}({})", 
                                if modifier == "virt" { "virtual " } else if modifier == "stat" { "static " } else { "" },
                                return_type, name, param_list.join(", "));

                            cpp_lines.push(if !access.is_empty() && modifier != "virt" {
                                format!("{} {}", access, sig)
                            } else {
                                sig
                            } + " {");
                            
                        }
                    }
                    else if line.starts_with("if ") {
                        cpp_lines.push(format!("if {}", &line[3..].trim()));
                    }
                    else if line.starts_with("} else if ") {
						cpp_lines.push(format!("}} else {}", &line[7..].trim()));
                    }
					else if line.starts_with("} else") {
						cpp_lines.push(format!("}} {}else {{", ""));
                    }
                    else if line.starts_with("for ") {
                        if let Some(caps) = RANGE_REGEX.captures(line) {
                            let (var, start, end) = (
                                caps.get(1).unwrap().as_str(),
                                caps.get(2).unwrap().as_str(),
                                caps.get(3).unwrap().as_str()
                            );
                            cpp_lines.push(format!("{}for (int {} = {}; {} < {}; {}++) {{", 
                                "".repeat(self.current_indent),
                                var, start, var, end, var
                            ));
                        } else if let Some(caps) = FOREACH_REGEX.captures(line) {
                            let (var, container) = (
                                caps.get(1).unwrap().as_str(),
                                caps.get(2).unwrap().as_str()
                            );
                            cpp_lines.push(format!("{}for (any&& {} : {}) {{", 
                                "".repeat(self.current_indent),
                                var, container
                            ));
                        }
                    }
                    else if line.starts_with("while ") {
                        cpp_lines.push(format!("while {}", &line[6..].trim()));
                    }
                    else if line.starts_with("return ") {
                        let ret = &line[7..].trim();
                        cpp_lines.push(format!("return {};",if ret.ends_with(';') { &ret[..ret.len()-1] } else { ret }));
                    }
                    else if line.contains('=') && !line.contains("-=") && !line.contains("+=") && !line.contains("*=") && !line.contains("/=") {
                        if let Some((left, right)) = line.split_once('=') {
                            let (name, var_type) = left.split_once(':').map(|(n, t)| (n.trim(), t.trim()))
                                .unwrap_or((left.trim(), "any"));
                            let value = right.trim().trim_end_matches(';');
                            cpp_lines.push(format!("{} {} = {};", 
                                var_type, name, value));
                        }
                    }
                    else {
                        cpp_lines.push(format!("{}{}", 
                            line,
                            if line.ends_with('{') || line.ends_with(',') || line.ends_with(':') || line.ends_with(';') { "" } else { ";" }));
                    }
                }
            }
            i += 1;
        }
        cpp_lines.join("\n")
    }
}

fn get_compiler_command() -> Vec<&'static str> {
    if cfg!(windows) { vec!["g++", "-std=c++20"] }
    else { vec!["clang++", "-std=c++20"] }
}

fn compile_cpp(cpp_filename: &str) -> Option<String> {
    let out_filename = format!("{}{}", cpp_filename.replace(".cpp", ""), if cfg!(windows) { ".exe" } else { "" });
    let compiler = get_compiler_command();

    println!("{} Compiling with {}...", "⟳".yellow(), compiler.join(" ").cyan());
    
    Command::new(compiler[0])
        .args(&[compiler[1], cpp_filename, "-o", &out_filename])
        .output()
        .map(|output| {
            if output.status.success() {
                println!("{} Successfully compiled to {}", "✓".green(), &out_filename.cyan());
                Some(out_filename)
            } else {
                println!("{} Compilation error:\n{}", "✗".red(), String::from_utf8_lossy(&output.stderr));
                None
            }
        })
        .unwrap_or_else(|e| {
            println!("{} Compiler error: {}", "✗".red(), e);
            None
        })
}

fn display_banner() {
    println!("{}", "────────────────────────────────────────────".magenta());
    println!("{} {} {}", "│".magenta(), " █████  ███████ ████████ ██████   █████ ".cyan(), "│".magenta());
    println!("{} {} {}", "│".magenta(), "██   ██ ██         ██    ██   ██ ██   ██".cyan(), "│".magenta());
    println!("{} {} {}", "│".magenta(), "███████ ███████    ██    ██████  ███████".cyan(), "│".magenta());
    println!("{} {} {}", "│".magenta(), "██   ██      ██    ██    ██   ██ ██   ██".cyan(), "│".magenta());
    println!("{} {} {}", "│".magenta(), "██   ██ ███████    ██    ██   ██ ██   ██".cyan(), "│".magenta());
    println!("{}", "────────────────────────────────────────────".magenta());
}

fn main() {
    display_banner();
    let cli = Cli::parse();

    let filename = match &cli.command {
        Some(Commands::Transpile { input }) | Some(Commands::Compile { input }) => input,
        None => cli.input.as_ref().unwrap_or_else(|| {
            println!("{} Usage: astra [COMMAND] filename.astra", "⚠".yellow());
            exit(1);
        })
    };

    if !filename.ends_with(".astra") {
        println!("{} Invalid file extension: {}", "✗".red(), filename.cyan());
        exit(1);
    }

    let mut transpiler = AstraTranspiler::new();
    let cpp_file = transpiler.transpile(filename);

    match &cli.command {
        Some(Commands::Compile { .. }) => {
            if let Some(cpp) = cpp_file {
                if let Some(exe) = compile_cpp(&cpp) {
					exit(0)
                } else {
                    exit(1);
                }
            }
        }
        _ => {}
    }
}