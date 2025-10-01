fn main() {
    build_interoper();
}

fn build_interoper() {
    println!("cargo::rerun-if-changed=Interoper.toml");
    println!("cargo::rerun-if-changed=./styles/src");
    let project = match interoper::build() {
        Ok(project) => project,
        Err(e) => {
            println!("cargo::warning=failed to build interoper: {e:?}");
            return;
        }
    };

    project.build_templates("./styles/src", "./styles/generated");
}
