mod generator;
fn main() {
    if std::env::var("GENERATE_PARSER").map_or(false, |env| env == "1") {
        generator::generate();
    }
}
