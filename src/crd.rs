use std::fs;
use std::path::PathBuf;
use kube::{CustomResourceExt, Resource};
use serde_yaml;
use anyhow::{Context, Result};

mod protogen;
mod ingress;

fn main() -> Result<()> {
    println!("Generating crd for {}", ingress::XIngress::kind(&())); // impl kube::Resource
    let crd_file = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("config/crd/xingress.yaml");
    fs::create_dir_all(crd_file.parent().unwrap())?;
    fs::write(crd_file, serde_yaml::to_string(&ingress::XIngress::crd()).unwrap()).context("Failed to write file")
}
