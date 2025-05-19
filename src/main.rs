use cfg_if::cfg_if;
use rostig::run;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;


#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub fn main() 
{
    cfg_if!
    {
        if #[cfg(target_arch = "wasm32")]
        {
            wasm_bindgen_futures::spawn_local(run());
        }
        else
        {
            pollster::block_on(run());
        }
    }
}