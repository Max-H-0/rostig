use rostig::run;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;


fn main()
{
    pollster::block_on(run());
}

#[cfg(target_arch="wasm32")]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn start() 
{ 
    run().await; 
}