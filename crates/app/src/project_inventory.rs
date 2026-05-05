use std::{fs, path::Path};

#[derive(Debug)]
pub(crate) struct ProjectInventory {
    pub(crate) sol_files: Vec<String>,
    pub(crate) has_foundry_toml: bool,
    pub(crate) has_remappings: bool,
}

pub(crate) fn scan_project_inventory(root: &Path) -> ProjectInventory {
    let mut sol_files = Vec::new();

    let source_dirs = ["src", "script", "test", "contracts"];
    for dir in source_dirs {
        let path = root.join(dir);
        if path.is_dir() {
            collect_sol_files(&path, root, &mut sol_files, 0);
        }
    }

    if sol_files.is_empty() {
        collect_sol_files(root, root, &mut sol_files, 0);
    }

    sol_files.sort();
    sol_files.dedup();
    if sol_files.len() > 300 {
        sol_files.truncate(300);
    }

    ProjectInventory {
        sol_files,
        has_foundry_toml: root.join("foundry.toml").exists(),
        has_remappings: root.join("remappings.txt").exists(),
    }
}

fn collect_sol_files(dir: &Path, root: &Path, output: &mut Vec<String>, depth: usize) {
    if depth > 6 {
        return;
    }

    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let file_name = entry.file_name();
        let file_name = file_name.to_string_lossy();

        if path.is_dir() {
            if should_skip_dir(&file_name) {
                continue;
            }
            collect_sol_files(&path, root, output, depth + 1);
            continue;
        }

        if path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.eq_ignore_ascii_case("sol"))
            .unwrap_or(false)
        {
            let relative = path
                .strip_prefix(root)
                .ok()
                .map(|relative| relative.to_string_lossy().to_string())
                .unwrap_or_else(|| path.to_string_lossy().to_string());
            output.push(relative);
        }
    }
}

fn should_skip_dir(name: &str) -> bool {
    matches!(
        name,
        ".git"
            | ".github"
            | ".idea"
            | ".vscode"
            | "target"
            | "cache"
            | "out"
            | "node_modules"
            | "broadcast"
            | "tmp"
    )
}
