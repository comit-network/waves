use std::process::Command;

fn main() {
    let output = Command::new("docker")
        .arg("build")
        .arg("--tag")
        .arg("coblox/elementsd:0.18.1.9")
        .arg(".")
        .output();

    let build_result = match output {
        Ok(output) => {
            let stdout = String::from_utf8(output.stdout)
                .unwrap_or_else(|_| String::from("Could not decode stdout"));
            let stderr = String::from_utf8(output.stderr)
                .unwrap_or_else(|_| String::from("Could not decode stderr"));
            format!("Stdout: {} \n Stderr: {}", stdout, stderr)
        }
        Err(error) => format!("Docker container not built: {}", error),
    };

    println!("Docker container build result: {}", build_result);
    println!("cargo:rerun-if-changed=Dockerfile");
}
