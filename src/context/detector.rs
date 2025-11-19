use crate::Result;
use std::path::Path;
use std::fs;
use std::io::{self, Read};

/// Represents the detected project context
#[derive(Debug, Clone, Default)]
pub struct ProjectContext {
    pub tags: Vec<String>,
}

impl ProjectContext {
    /// Check if context has a specific tag
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.iter().any(|t| t == tag)
    }

    /// Get relevance boost multiplier for a command/context pair
    /// e.g., npm commands get 2.0x boost in nodejs projects
    pub fn get_boost_multiplier(&self, command: &str) -> f32 {
        let cmd_lower = command.to_lowercase();
        
        // Node.js project boosts
        if self.has_tag("nodejs") {
            if cmd_lower.contains("npm") || cmd_lower.contains("yarn") || cmd_lower.contains("node") {
                return 2.5;
            }
        }
        
        // Python project boosts
        if self.has_tag("python") {
            if cmd_lower.contains("pip") || cmd_lower.contains("python") || cmd_lower.contains("venv") {
                return 2.5;
            }
        }
        
        // Docker boosts
        if self.has_tag("docker") || self.has_tag("docker-compose") {
            if cmd_lower.contains("docker") {
                return 2.5;
            }
        }
        
        // Kubernetes boosts
        if self.has_tag("kubernetes") {
            if cmd_lower.contains("kubectl") || cmd_lower.contains("helm") || cmd_lower.contains("k8s") {
                return 2.5;
            }
        }
        
        // Git boosts
        if self.has_tag("git") {
            if cmd_lower.contains("git") {
                return 2.0;
            }
        }
        
        // Rust boosts
        if self.has_tag("rust") {
            if cmd_lower.contains("cargo") || cmd_lower.contains("rustc") {
                return 2.5;
            }
        }
        
        // Go boosts
        if self.has_tag("golang") {
            if cmd_lower.contains("go ") || cmd_lower.contains("go\t") {
                return 2.5;
            }
        }
        
        // Java boosts
        if self.has_tag("java") {
            if cmd_lower.contains("mvn") || cmd_lower.contains("gradle") || cmd_lower.contains("javac") {
                return 2.5;
            }
        }
        
        1.0 // Default: no boost
    }
}

/// Detect common project context tags by inspecting files in `path`.
/// Returns a ProjectContext with tags (e.g., ["nodejs","docker","k8s","python"]).
pub fn detect_project_context(path: &Path) -> Result<ProjectContext> {
    let mut tags: Vec<String> = Vec::new();

    if !path.exists() {
        return Ok(ProjectContext { tags });
    }

    // Helper: check for the existence of file names
    let has = |name: &str| -> bool { path.join(name).exists() };

    if has("Dockerfile") {
        tags.push("docker".to_string());
    }
    if has("docker-compose.yml") || has("docker-compose.yaml") {
        tags.push("docker-compose".to_string());
    }
    if has("package.json") {
        tags.push("nodejs".to_string());
    }
    if has("pyproject.toml") || has("requirements.txt") {
        tags.push("python".to_string());
    }
    if has("go.mod") {
        tags.push("golang".to_string());
    }
    if has("Cargo.toml") {
        tags.push("rust".to_string());
    }
    if has("pom.xml") {
        tags.push("java".to_string());
    }

    // Kubernetes manifests: look for .yaml/.yml files that contain "kind:" or "apiVersion:"
    let k8s_found = find_in_files_with_extensions(path, &["yaml", "yml"], |content| {
        content.contains("kind:") || content.contains("apiVersion:")
    })?;
    if k8s_found {
        tags.push("kubernetes".to_string());
    }

    // Check for .git directory to identify repo
    if path.join(".git").exists() {
        tags.push("git".to_string());
    }

    // Detect Docker engine running (best-effort): check for /var/run/docker.sock presence in container host
    if Path::new("/var/run/docker.sock").exists() {
        tags.push("docker-host".to_string());
    }

    // Deduplicate
    tags.sort();
    tags.dedup();

    Ok(ProjectContext { tags })
}

/// Search files with given extensions and apply predicate to file content.
/// Returns true if any file matches predicate.
fn find_in_files_with_extensions<F>(dir: &Path, exts: &[&str], mut predicate: F) -> io::Result<bool>
where
    F: FnMut(&str) -> bool,
{
    if !dir.is_dir() {
        return Ok(false);
    }
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let p = entry.path();
        if p.is_file() {
            if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
                if exts.contains(&ext) {
                    if let Ok(mut f) = fs::File::open(&p) {
                        let mut s = String::new();
                        if f.read_to_string(&mut s).is_ok() {
                            if predicate(&s) {
                                return Ok(true);
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn detects_node_and_docker_and_k8s() {
        let dir = tempdir().unwrap();
        let base = dir.path();
        // create package.json
        File::create(base.join("package.json")).unwrap();
        // create Dockerfile
        File::create(base.join("Dockerfile")).unwrap();
        // create a k8s yaml
        let mut f = File::create(base.join("deployment.yaml")).unwrap();
        writeln!(f, "apiVersion: apps/v1\nkind: Deployment").unwrap();

        let ctx = detect_project_context(base).unwrap();
        assert!(ctx.has_tag("docker"));
        assert!(ctx.has_tag("nodejs"));
        assert!(ctx.has_tag("kubernetes"));
    }

    #[test]
    fn boost_multiplier_works() {
        let ctx = ProjectContext {
            tags: vec!["nodejs".to_string()],
        };
        assert_eq!(ctx.get_boost_multiplier("npm install express"), 2.5);
        assert_eq!(ctx.get_boost_multiplier("ls -la"), 1.0);
    }
}
