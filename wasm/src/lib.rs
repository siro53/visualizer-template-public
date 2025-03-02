use wasm_bindgen::prelude::*;
mod util;

#[wasm_bindgen]
pub fn gen(seed: i32, problemId: &str) -> String {
    util::gen(seed as u64, problemId).to_string()
}

#[wasm_bindgen(getter_with_clone)]
pub struct Ret {
    pub score: i64,
    pub err: String,
    pub svg: String,
}

#[wasm_bindgen]
pub fn vis(_input: String, _output: String, turn: usize) -> Ret {
    let input = util::parse_input(&_input);
    let output = match util::parse_output(&input, &_output) {
        Ok(o) => o,
        Err(e) => {
            return Ret {
                score: 0,
                err: e,
                svg: "".to_owned(),
            };
        }
    };
    let vr = util::vis(&input, &output, turn);
    Ret {
        score: vr.score,
        err: vr.err,
        svg: vr.svg,
    }
}

#[wasm_bindgen]
pub fn get_max_turn(_input: String, _output: String) -> usize {
    let input = util::parse_input(&_input);
    let output = match util::parse_output(&input, &_output) {
        Ok(o) => o,
        Err(_) => return 0,
    };
    util::get_max_turn(&input, &output)
}
