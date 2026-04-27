mod generator;
fn main() {
    println!("cargo:rerun-if-changed=sparql.ungram");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=generator");
    println!("cargo:rerun-if-env-changed=GENERATE_PARSER");
    println!("cargo:rerun-if-env-changed=GENERATE_TYPES");
    println!("cargo:rerun-if-env-changed=GENERATE_RULES");
    generator::generate();
}
