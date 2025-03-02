use std::collections::HashMap;
use std::fs;
use std::process::{Command, exit};
use regex::Regex;
use colored::*;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "Astra")]
#[command(author = "NEOAPPS")]
#[command(version = "1.0")]
#[command(about = "The Powerful Transpiled Programming Language", long_about = None)]
struct Cli {
    #[arg(value_name = "INPUT")]
    input: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Transpile an Astra file to C++
    Transpile {
        /// Input .astra file
        #[arg(value_name = "INPUT")]
        input: String,
    },
    /// Transpile and compile an Astra file
    Compile {
        /// Input .astra file
        #[arg(value_name = "INPUT")]
        input: String,
    },
    /// Transpile, compile, and run an Astra file
    Run {
        /// Input .astra file
        #[arg(value_name = "INPUT")]
        input: String,
    },
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
        match fs::read_to_string(filename) {
            Ok(content) => {
                let output_filename = filename.replace(".astra", ".cpp");
                let cpp_code = self.process_content(&content);
                
                match fs::write(&output_filename, cpp_code) {
                    Ok(_) => {
                        println!("{} Transpiled {} to {}", "✓".green(), filename.cyan(), output_filename.cyan());
                        Some(output_filename)
                    },
                    Err(e) => {
                        println!("{} Error writing to file {}: {}", "✗".red(), output_filename.cyan(), e);
                        None
                    }
                }
            },
            Err(e) => {
                println!("{} Error reading file {}: {}", "✗".red(), filename.cyan(), e);
                None
            }
        }
    }

    fn process_content(&mut self, content: &str) -> String {
        let lines: Vec<&str> = content.split('\n').collect();
        let mut cpp_lines = vec![
		    "#include <iostream>".to_string(),
            "#define any auto".to_string(),
			"#define pub public".to_string(),
			"#define priv private".to_string(),
			"#define prot protected".to_string(),
			"#define println print".to_string(),
			"#define wfile ofstream".to_string(),
			"#define rfile ifstream".to_string(),
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
            "".to_string(),
            "using namespace std;".to_string(),
            "".to_string(),
            "void print(auto x) {cout << x << '\\n';}".to_string(),
            "int $(char* x) {return system(x);}".to_string(),
			"".to_string(),
        ];
        
        let mut i = 0;
        while i < lines.len() {
            let line = lines[i].trim();
            
            if line.is_empty() {
                cpp_lines.push("".to_string());
                i += 1;
                continue;
            }

            if line == "::++ {" {
                self.in_raw_cpp = true;
                i += 1;
                continue;
            } else if line == "}" && self.in_raw_cpp {
                self.in_raw_cpp = false;
                cpp_lines.extend(self.raw_cpp_content.clone());
                self.raw_cpp_content.clear();
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
                
            if line.starts_with("use ") {
                cpp_lines.push(format!("#include <{}>", &line[4..]));
                i += 1;
                continue;
            }
            
            if line.starts_with("def ") {
                cpp_lines.push(format!("#define {}", &line[4..]));
                i += 1;
                continue;
            }
                
            if line.starts_with("type ") {
                let mut x = "";
                if !line.ends_with(';') {
                    x = ";";
                }
                cpp_lines.push(line.replace("type", "enum") + x);
                i += 1;
                continue;
            }
          
            if line.starts_with("fn ") {
                cpp_lines.extend(self.process_function_definition(line));
			} else if line.starts_with("pub fn ") {
				cpp_lines.extend(self.process_public_function_definition(line));
			} else if line.starts_with("prot fn ") {
				cpp_lines.extend(self.process_protected_function_definition(line));
			} else if line.starts_with("priv fn ") {
				cpp_lines.extend(self.process_private_function_definition(line));
            } else if line.starts_with("if ") {
                cpp_lines.extend(self.process_if_statement(line));
            } else if line.starts_with("else ") {
                cpp_lines.extend(self.process_else_statement(line));
            } else if line.starts_with("elif ") {
                cpp_lines.extend(self.process_elif_statement(line));
            } else if line.starts_with("for ") {
                cpp_lines.extend(self.process_for_loop(line));
            } else if line.starts_with("while ") {
                cpp_lines.extend(self.process_while_loop(line));
            } else if line.starts_with("return ") {
                cpp_lines.extend(self.process_return_statement(line));
            } else if line == "}" {
                self.current_indent -= 1;
                cpp_lines.push("    ".repeat(self.current_indent) + "}");
            } else if line.contains('=') && !line.starts_with("    ") && !line.contains("==") {
                cpp_lines.extend(self.process_variable_declaration(line));
            } else if line.ends_with(';') {
                cpp_lines.push("    ".repeat(self.current_indent) + line);
            } else {
                let mut line_copy = line.to_string();
                if !line_copy.ends_with(';') && !line_copy.ends_with('{') && !line_copy.ends_with('}') {
                    line_copy += ";";
                }
                cpp_lines.push("    ".repeat(self.current_indent) + &line_copy);
            }
            
            i += 1;
        }
        
        cpp_lines.join("\n")
    }

    fn process_function_definition(&mut self, line: &str) -> Vec<String> {
        let function_pattern = Regex::new(r"fn (\w+)\((.*?)\)(?: -> (\w+))?").unwrap();
        
        if let Some(captures) = function_pattern.captures(line) {
            let name = captures.get(1).map_or("", |m| m.as_str());
            let params = captures.get(2).map_or("", |m| m.as_str());
            let return_type = captures.get(3).map_or("void", |m| m.as_str());

            let mut param_list = Vec::new();
            if !params.is_empty() {
                for param in params.split(',') {
                    let param = param.trim();
                    if param.contains(':') {
                        let parts: Vec<&str> = param.split(':').collect();
                        param_list.push(format!("{} {}", parts[1].trim(), parts[0].trim()));
                    } else {
                        param_list.push(format!("any {}", param));
                    }
                }
            }
            
            let function_signature = format!("{} {}({})", return_type, name, param_list.join(", "));
            self.functions.insert(name.to_string(), FunctionInfo {
                return_type: return_type.to_string(),
                params: param_list,
            });
            self.in_function = true;
            self.current_indent += 1;
            
            return vec![
                format!("{} {{", function_signature),
            ];
        }
        
        return vec![format!("// Error parsing function: {}", line)];
    }
	
    fn process_public_function_definition(&mut self, line: &str) -> Vec<String> {
        let function_pattern = Regex::new(r"pub fn (\w+)\((.*?)\)(?: -> (\w+))?").unwrap();
        
        if let Some(captures) = function_pattern.captures(line) {
            let name = captures.get(1).map_or("", |m| m.as_str());
            let params = captures.get(2).map_or("", |m| m.as_str());
            let return_type = captures.get(3).map_or("void", |m| m.as_str());

            let mut param_list = Vec::new();
            if !params.is_empty() {
                for param in params.split(',') {
                    let param = param.trim();
                    if param.contains(':') {
                        let parts: Vec<&str> = param.split(':').collect();
                        param_list.push(format!("{} {}", parts[1].trim(), parts[0].trim()));
                    } else {
                        param_list.push(format!("any {}", param));
                    }
                }
            }
            
            let function_signature = format!("{} {}({})", return_type, name, param_list.join(", "));
            self.functions.insert(name.to_string(), FunctionInfo {
                return_type: return_type.to_string(),
                params: param_list,
            });
            self.in_function = true;
            self.current_indent += 1;
            
            return vec![
                format!("pub:{} {{", function_signature),
            ];
        }
        
        return vec![format!("// Error parsing public function: {}", line)];
    }
	
    fn process_protected_function_definition(&mut self, line: &str) -> Vec<String> {
        let function_pattern = Regex::new(r"prot fn (\w+)\((.*?)\)(?: -> (\w+))?").unwrap();
        
        if let Some(captures) = function_pattern.captures(line) {
            let name = captures.get(1).map_or("", |m| m.as_str());
            let params = captures.get(2).map_or("", |m| m.as_str());
            let return_type = captures.get(3).map_or("void", |m| m.as_str());

            let mut param_list = Vec::new();
            if !params.is_empty() {
                for param in params.split(',') {
                    let param = param.trim();
                    if param.contains(':') {
                        let parts: Vec<&str> = param.split(':').collect();
                        param_list.push(format!("{} {}", parts[1].trim(), parts[0].trim()));
                    } else {
                        param_list.push(format!("any {}", param));
                    }
                }
            }
            
            let function_signature = format!("{} {}({})", return_type, name, param_list.join(", "));
            self.functions.insert(name.to_string(), FunctionInfo {
                return_type: return_type.to_string(),
                params: param_list,
            });
            self.in_function = true;
            self.current_indent += 1;
            
            return vec![
                format!("prot:{} {{", function_signature),
            ];
        }
        
        return vec![format!("// Error parsing public function: {}", line)];
    }
	fn process_private_function_definition(&mut self, line: &str) -> Vec<String> {
        let function_pattern = Regex::new(r"priv fn (\w+)\((.*?)\)(?: -> (\w+))?").unwrap();
        
        if let Some(captures) = function_pattern.captures(line) {
            let name = captures.get(1).map_or("", |m| m.as_str());
            let params = captures.get(2).map_or("", |m| m.as_str());
            let return_type = captures.get(3).map_or("void", |m| m.as_str());

            let mut param_list = Vec::new();
            if !params.is_empty() {
                for param in params.split(',') {
                    let param = param.trim();
                    if param.contains(':') {
                        let parts: Vec<&str> = param.split(':').collect();
                        param_list.push(format!("{} {}", parts[1].trim(), parts[0].trim()));
                    } else {
                        param_list.push(format!("any {}", param));
                    }
                }
            }
            
            let function_signature = format!("{} {}({})", return_type, name, param_list.join(", "));
            self.functions.insert(name.to_string(), FunctionInfo {
                return_type: return_type.to_string(),
                params: param_list,
            });
            self.in_function = true;
            self.current_indent += 1;
            
            return vec![
                format!("priv:{} {{", function_signature),
            ];
        }
        
        return vec![format!("// Error parsing private function: {}", line)];
    }

    fn process_variable_declaration(&mut self, line: &str) -> Vec<String> {
        if line.contains(':') && line.contains('=') {
            let parts: Vec<&str> = line.split('=').collect();
            let name_type = parts[0];
            let value = parts[1].trim();
            
            let name_type_parts: Vec<&str> = name_type.split(':').collect();
            let name = name_type_parts[0].trim();
            let var_type = name_type_parts[1].trim();
            
            let mut value_copy = value.to_string();
            if !value_copy.ends_with(';') {
                value_copy += ";";
            }
            
            self.variables.insert(name.to_string(), var_type.to_string());
            return vec![format!("{}{} {} = {}", "    ".repeat(self.current_indent), var_type, name, value_copy)];
        } else if line.contains('=') {
            let parts: Vec<&str> = line.split('=').collect();
            let name = parts[0].trim();
            let value = parts[1].trim();
            
            let mut value_copy = value.to_string();
            if !value_copy.ends_with(';') {
                value_copy += ";";
            }
            
            self.variables.insert(name.to_string(), "any".to_string());
            return vec![format!("{}any {} = {}", "    ".repeat(self.current_indent), name, value_copy)];
        }
        
        return vec![format!("// Error parsing variable declaration: {}", line)];
    }

    fn process_if_statement(&mut self, line: &str) -> Vec<String> {
        let condition = &line[3..].trim();
        self.current_indent += 1;
        return vec![format!("{}if {} {{", "    ".repeat(self.current_indent - 1), condition)];
    }

    fn process_else_statement(&mut self, _line: &str) -> Vec<String> {
        return vec![format!("{}else {{", "    ".repeat(self.current_indent - 1))];
    }

    fn process_elif_statement(&mut self, line: &str) -> Vec<String> {
        let condition = &line[5..].trim();
        return vec![format!("{}else if ({}) {{", "    ".repeat(self.current_indent - 1), condition)];
    }

    fn process_for_loop(&mut self, line: &str) -> Vec<String> {
        let range_pattern = Regex::new(r"for (\w+) in range\((\d+),\s*(\d+)\)").unwrap();
        
        if let Some(captures) = range_pattern.captures(line) {
            let var = captures.get(1).unwrap().as_str();
            let start = captures.get(2).unwrap().as_str();
            let end = captures.get(3).unwrap().as_str();
            
            self.current_indent += 1;
            return vec![format!("{}for (int {} = {}; {} < {}; {}++) {{", 
                "    ".repeat(self.current_indent - 1),
                var, start, var, end, var
            )];
        }

        let foreach_pattern = Regex::new(r"for (\w+) in (\w+)").unwrap();
        
        if let Some(captures) = foreach_pattern.captures(line) {
            let var = captures.get(1).unwrap().as_str();
            let container = captures.get(2).unwrap().as_str();
            
            self.current_indent += 1;
            return vec![format!("{}for (any& {} : {})", 
                "    ".repeat(self.current_indent - 1),
                var, container
            )];
        }
        
        return vec![format!("// Error parsing for loop: {}", line)];
    }

    fn process_while_loop(&mut self, line: &str) -> Vec<String> {
        let condition = &line[6..].trim();
        self.current_indent += 1;
        return vec![format!("{}while {}", "    ".repeat(self.current_indent - 1), condition)];
    }

    fn process_return_statement(&mut self, line: &str) -> Vec<String> {
        let mut value = line[7..].trim().to_string();
        if !value.ends_with(';') {
            value += ";";
        }
        return vec![format!("{}return {}", "    ".repeat(self.current_indent), value)];
    }
}

fn get_compiler_command() -> Vec<&'static str> {
    if cfg!(windows) {
        vec!["g++", "-std=c++20"]
    } else if cfg!(target_os = "macos") {
        vec!["clang++", "-std=c++20"]
    } else if cfg!(target_os = "linux") {
        vec!["g++", "-std=c++20"]
    } else {
        vec!["g++", "-std=c++20"]
    }
}

fn compile_cpp(cpp_filename: &str) -> Option<String> {
    let mut out_filename = cpp_filename.replace(".cpp", "");
    if cfg!(windows) {
        out_filename += ".exe";
    }
    
    let compiler_cmd = get_compiler_command();
    
    println!("{} Compiling with {}...", "⟳".yellow(), compiler_cmd.join(" ").cyan());
    
    let output = Command::new(compiler_cmd[0])
        .args(&[compiler_cmd[1], cpp_filename, "-o", &out_filename])
        .output();
    
    match output {
        Ok(output) => {
            if output.status.success() {
                println!("{} Successfully compiled {}", "✓".green(), cpp_filename.cyan());
                Some(out_filename)
            } else {
                println!("{} Compilation error:", "✗".red());
                let error = String::from_utf8_lossy(&output.stderr);
                for line in error.lines() {
                    if !line.trim().is_empty() {
                        println!("  {} {}", "→".red(), line);
                    }
                }
                None
            }
        },
        Err(e) => {
            println!("{} Error: Compiler not found. Please install g++ or clang++: {}", "✗".red(), e);
            None
        }
    }
}

fn run_executable(executable: &str) -> i32 {
    println!("{} Running {}...", "⟳".yellow(), executable.cyan());
    
    match Command::new(executable).status() {
        Ok(status) => {
            println!("{} Program exited with code {}", "✓".green(), status.code().unwrap_or(0));
            status.code().unwrap_or(0)
        },
        Err(e) => {
            println!("{} Failed to execute program: {}", "✗".red(), e);
            1
        }
    }
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
        Some(Commands::Transpile { input }) => input,
        Some(Commands::Compile { input }) => input,
        Some(Commands::Run { input }) => input,
        None => match &cli.input {
            Some(input) => input,
            None => {
                println!("{} Usage: astra [COMMAND] filename.astra", "⚠".yellow());
                println!("Commands:");
                println!("  transpile  - Transpile Astra Code");
                println!("  compile    - Transpile Astra Code and compile");
                println!("  run        - Transpile, compile, and execute the program");
                exit(1);
            }
        }
    };

    if !filename.ends_with(".astra") {
        println!("{} Error: File must have {} extension", "✗".red(), ".astra".cyan());
        exit(1);
    }
    
    println!("{} Processing {}", "ℹ".blue(), filename.cyan());
    
    let mut transpiler = AstraTranspiler::new();
    let cpp_file = transpiler.transpile(filename);
    
    match &cli.command {
        Some(Commands::Transpile { .. }) => {
            if cpp_file.is_none() {
                println!("{} Failed to transpile {}", "✗".red(), filename.cyan());
                exit(1);
            }
        },
        Some(Commands::Compile { .. }) | Some(Commands::Run { .. }) => {
            if let Some(cpp_filename) = cpp_file {
                let executable = compile_cpp(&cpp_filename);
                
                if let Some(exec_name) = executable {
                    println!("{} Ready to run: {}", "✓".green(), exec_name.cyan());
                    
                    if let Some(Commands::Run { .. }) = &cli.command {
                        let exit_code = run_executable(&exec_name);
                        exit(exit_code);
                    }
                } else {
                    println!("{} Failed to compile {}", "✗".red(), cpp_filename.cyan());
                    exit(1);
                }
            } else {
                println!("{} Failed to transpile {}", "✗".red(), filename.cyan());
                exit(1);
            }
        },
        None => {
            if cpp_file.is_none() {
                println!("{} Failed to transpile {}", "✗".red(), filename.cyan());
                exit(1);
            }
        },
    }
}
