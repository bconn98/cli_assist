
use std::{
    env,
    fs::{File, remove_dir_all, create_dir_all},
    io::{Write, BufRead},
    path::Path,
    process::Command,
};
use glob::glob;
use clap::Parser;
use regex::Regex;

#[derive(Parser, Debug)]
pub struct CmakeVars {
    /// Destroy the CMake Build Directory
    #[clap(short, long, action)]
    destroy: bool,

    /// Configure the CMake Project
    #[clap(long, action)]
    configure: bool,

    /// Execute the CMake Build process
    #[clap(short, long, action)]
    build: bool,

    /// Execute CTest on the CMake Project
    #[clap(long, action)]
    test: bool,

    /// Execute Code Coverage on the CMake Project
    #[clap(long, action)]
    coverage: bool,

    /// Install the Project via CMake
    #[clap(short, long, action)]
    install: bool,

    /// Build a specfic CMake target
    #[clap(short, long)]
    target: Option<String>,

    /// Execute Clang-Tidy on the CMake Project
    #[clap(long, action)]
    tidy: bool,

    /// Configure CMake for the Release Configuration
    #[clap(long, action)]
    release: bool,
}

pub fn process(cmds: CmakeVars) {
    let mut status = true;
    let build_path = env::var("BUILD_DIR")
        .expect("BUILD_DIR environment variable not set");

    if cmds.destroy && Path::new(&build_path).exists() {
        status = destroy_cmake(&build_path)
    }

    let (cmake_target, target) = match cmds.target {
        Some(ref cmake_target) => (cmake_target.clone(), true),
        None => (String::default(), false),
    };

    // Must check in order of impact. I.e. If coverage is enabled, it needs to
    // enable tests.
    let release = cmds.release;
    let install = cmds.install;
    let coverage = cmds.coverage;
    let tidy = cmds.tidy;
    let test = cmds.test || coverage;
    let build = cmds.build || test || install || tidy;
    let configure = cmds.configure || build || release || target || tidy;

    if target && cmake_target == "clean" && Path::new(&build_path).exists() {
        // If this doesn't run as a true clean, it will just run a configure,
        // esentially acting as a call to configure a fresh BUILD_DIR.
        status = clean_cmake(&build_path);
    }

    if status && configure {
        status = configure_cmake(cmds, release, &build_path)
    }

    if status && build {
        status = build_cmake(&build_path)
    }

    if status && target && cmake_target != "clean" {
        status = target_cmake(&cmake_target, &build_path)
    }

    if status && test {
        status = test_cmake(&build_path)
    }

    if status && coverage {
        status = coverage_cmake(&build_path)
    }

    if status && tidy {
        status = clang_tidy(&build_path)
    }

    if status && install {
        status = install_cmake(&build_path)
    }

    println!("CMake finished with: {status}");
}

fn destroy_cmake(artifacts: &String) -> bool {
    match remove_dir_all(artifacts) {
        Ok(_) => true,
        _ => false,
    }
}

fn configure_cmake(cmds: CmakeVars, release: bool, artifacts: &String) -> bool {
    let mut cmd = Command::new("cmake");

    let build_cfg = match release {
        true => "-DCMAKE_BUILD_TYPE=Release",
        false => "-DCMAKE_BUILD_TYPE=Debug",
    };

    cmd.arg("-S").arg(".").arg("-B").arg(artifacts).arg("-G").arg("Ninja").arg(build_cfg);

    if cmds.coverage {
        cmd.arg("-Dtest=ON").arg("-DENABLE_COVERAGE=ON");
    } else if cmds.test {
        cmd.arg("-Dtest=ON");
    }

    cmd.status().expect("failed to execute process").success()
}

fn target_cmake(target: &str, artifacts: &String) -> bool {
    Command::new("cmake")
        .arg("--build")
        .arg(artifacts)
        .arg("--parallel")
        .arg("--target")
        .arg(target)
        .status()
        .expect("failed to execute process")
        .success()
}

fn build_cmake(artifacts: &String) -> bool {
    target_cmake("all", artifacts)
}

fn coverage_cmake(artifacts: &String) -> bool {
    target_cmake("ExperimentalCoverage", artifacts)
}

fn install_cmake(artifacts: &String) -> bool {
    target_cmake("install", artifacts)
}

fn clean_cmake(artifacts: &String) -> bool {
    target_cmake("clean", artifacts)
}

fn find_cpp_files(exlude_dirs: String, repo_root: String) -> Vec<String> {
    // Search for .clang-tidy file
    let mut glob_path = repo_root.to_owned();
    glob_path.push_str("/**/*.cpp");

    let all_cpp_glob = glob(glob_path.as_str()).expect("Failed to read glob pattern");
    let mut all_cpp_files = Vec::<String>::new();
    let exlude_dirs = exlude_dirs.split(" ");
    for file in all_cpp_glob {
        let file = file.expect("Invalid file found in cpp glob").into_os_string().into_string().expect("Pathbuf into String");
       
        let mut regex_match = false;
        for dir in exlude_dirs.clone() {
            let regex = Regex::new(dir).unwrap();
            if regex.is_match(file.as_str()) {
                regex_match = true;
            }
        }

        if regex_match {
            all_cpp_files.push(file.clone());
        }
    }

    all_cpp_files
}

fn clang_tidy(artifacts: &String) -> bool {
    let repo_root = env::var("REPO_ROOT").expect("REPO_ROOT not set.");

    // Search for .clang-tidy file
    let cfg_loc = combine_artifact_path(&repo_root, "/**/.clang-tidy");
    let cfg_loc = glob(cfg_loc.as_str())
        .expect("Failed to find clang-tidy config")
        .into_iter()
        .next()
        .unwrap() // Unwrap option
        .unwrap() // Unwrap result
        .into_os_string()
        .into_string()
        .unwrap();
    
    // Scan for files that aren't excluded
    let tidy_exclude_dirs = match env::var("TIDY_EXCLUDE") {
        Ok(val) => val,
        Err(_) => String::default(),
    };
    
    let cpp_files = find_cpp_files(tidy_exclude_dirs, repo_root);

    let mut cfg_file = "--config-file=".to_string();
    cfg_file.push_str(cfg_loc.as_str());

    let mut fixes_file = combine_artifact_path(artifacts, "/ClangTidy");
    create_dir_all(&fixes_file).unwrap();
    fixes_file.push_str("/clang-tidy-fixes.yaml");

    let mut fix_file = "--export-fixes=".to_string();
    fix_file.push_str(fixes_file.as_str());

    // Call tool
    let output = Command::new("clang-tidy")
        .arg("-p")
        .arg(artifacts)
        .arg(cfg_file)
        .arg("--format-style=file")
        .arg(fix_file)
        .args(cpp_files)
        .output()
        .expect("failed to execute process");
    
    // Capture output to clang-tidy.log
    let out_file = combine_artifact_path(artifacts, "/ClangTidy/clang-tidy.log");
    make_and_write_file(out_file, &output.stdout);

    // Search output for error: or warning:
    let search_output = search_tidy(&output.stdout);

    // Write reduced to clang-tidy-err.log
    let mut out_file = artifacts.clone();
    out_file.push_str("/ClangTidy/clang-tidy-err.log");
    make_and_write_file(out_file, search_output.as_bytes());

    output.status.success()
}

fn combine_artifact_path(artifacts: &String, text: &str) -> String {
    let mut out_file = artifacts.clone();
    out_file.push_str(text);

    out_file
}

fn search_tidy(text: &[u8]) -> String {
    let mut search_output = String::default();
    let regex = Regex::new(r"error: |warning: ").unwrap();
    for line in text.lines() {
        let line = line.unwrap();
        if regex.is_match(line.as_str()) {
            search_output.push_str(line.as_str());
            search_output.push_str("\n");
        }
    }

    search_output
}

fn make_and_write_file(path: String, text: &[u8]) {
    let mut file = File::create(path.as_str()).unwrap_or_else(|_| panic!("Failed to create {}", path));
    file.write_all(text).unwrap();
}

fn test_cmake(artifacts: &String) -> bool {
    Command::new("ctest")
        .arg("--test-dir")
        .arg(artifacts)
        .arg("--output-junit")
        .arg("report.xml")
        .arg("--output-on-failure")
        .status()
        .expect("failed to execute process")
        .success()
}