//! Component Model exports — WASM target only.
wit_bindgen::generate!({ world: "kami-tool", path: "../../wit" });
struct P;
impl exports::kami::tool::tool::Guest for P {
    fn run(i: String) -> Result<String, String> { crate::handle(&i) }
    fn describe() -> String { crate::__kami_describe() }
}
export!(P);
