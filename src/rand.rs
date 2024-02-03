use wasm_bindgen::prelude::*;

pub fn usize(min:usize, max:usize) -> usize {
  // let min = iter.next().unwrap() as f64;
  // let max = iter.last().unwrap() as f64;
  (random()*(max as f64 - min as f64) + min as f64).floor() as usize
}

#[wasm_bindgen]
extern "C" {
  #[wasm_bindgen(js_namespace = Math, js_name = random)]
  fn random() -> f64;

}