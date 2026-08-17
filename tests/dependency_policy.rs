const MANIFEST: &str = include_str!("../Cargo.toml");

#[test]
fn every_direct_registry_dependency_has_an_exact_version() {
    let mut in_dependency_section = false;
    let mut observed = Vec::new();

    for (index, raw_line) in MANIFEST.lines().enumerate() {
        let line = raw_line.trim();
        if line.starts_with('[') {
            in_dependency_section = is_dependency_section(line);
            continue;
        }
        if !in_dependency_section
            || raw_line.chars().next().is_some_and(char::is_whitespace)
            || line.is_empty()
            || line.starts_with('#')
            || line.starts_with(']')
        {
            continue;
        }

        let (name, requirement) = line
            .split_once('=')
            .expect("a dependency declaration must contain an equals sign");
        let name = name.trim();
        let requirement = requirement.trim();
        let exact = requirement.starts_with(r#""="#)
            || (requirement.starts_with('{') && requirement.contains(r#"version = "="#));

        assert!(
            exact,
            "direct dependency {} on Cargo.toml line {} must use an exact =x.y.z requirement",
            name,
            index + 1
        );
        assert!(
            !requirement.contains("git =") && !requirement.contains("registry ="),
            "direct dependency {} on Cargo.toml line {} must use the reviewed crates.io source",
            name,
            index + 1
        );
        observed.push(name);
    }

    assert_eq!(
        observed,
        [
            "base64",
            "clap",
            "jsonschema",
            "libc",
            "process-wrap",
            "reqwest",
            "rustls",
            "serde",
            "serde_json",
            "tokio",
            "quick-xml",
            "rcgen",
            "tempfile",
        ],
        "the reviewed direct dependency inventory changed; update the dated PROJECT.md review"
    );
}

fn is_dependency_section(header: &str) -> bool {
    matches!(
        header,
        "[dependencies]" | "[dev-dependencies]" | "[build-dependencies]"
    ) || header.ends_with(".dependencies]")
        || header.ends_with(".dev-dependencies]")
        || header.ends_with(".build-dependencies]")
}
