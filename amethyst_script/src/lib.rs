#![feature(custom_attribute)]
#![allow(dead_code)]
pub mod builder;
pub mod schema;

use amethyst_script_derive::script;

#[script(accessible)]
pub struct TestDerive<T> {
    _scalar: i32,
    _generic: T,
    pub _public: i32,
}


#[script(accessible)]
pub struct TestReflection<T> {
    _scalar: i32,
    _generic: T,
    pub _public: i32,
}

#[script(accessible)]
impl<T> TestReflection<T> {
    fn function(&self, ) {

    }

    #[script(ignore)]
    pub fn not_function(&self, ) {

    }

    #[script(accessible)]
    pub fn pub_function_ret_ref(&self, ) -> &'static str {
        "asdf"
    }

    pub fn pub_function(&self, ) {

    }

    #[script(ignore)]
    fn not_pub_function(&self, ) {

    }
}


#[script(accessible)]
pub fn free_function(asdf: i32) {

}


#[script(acccesible)]
pub struct TestRepr {

}