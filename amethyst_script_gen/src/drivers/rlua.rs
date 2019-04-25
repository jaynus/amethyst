//! LUA driver for scripting

// We need to take a given reflection schema, and generate the appropriate abstract call map for it
// This will probably happen at compile time right?

use quote::quote;

pub fn gen_rust_rlua_bindings(buffer: &[u8] ) {

    // Parse the flatbuffers entry

    let var = quote! {
        println!("LOL");
    };

}