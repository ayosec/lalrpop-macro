#![crate_type = "proc-macro"]
#![crate_name = "lalrpop_macro"]

// The `quote!` macro requires deep recursion.
#![recursion_limit = "256"]

#[macro_use]
extern crate quote;

//#[macro_use]
extern crate syn;

extern crate lalrpop;
extern crate proc_macro;
extern crate tempdir;

use lalrpop::Configuration;
use proc_macro::TokenStream;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use syn::{ DeriveInput, Ident, Meta, Lit, MetaNameValue };
use tempdir::TempDir;

/*
#[macro_export]
macro_rules! lalrpop {
    ($modname: expr, $source: expr) => {
        #[derive(LarlpopGenerator)]
        #[lalrpop(src = $modname, modname = $modname)]
        struct Parser;
    }
}
*/

#[proc_macro_derive(LarlpopGenerator, attributes(source))]
pub fn derive_parser_generator(input: TokenStream) -> TokenStream {
    let input: DeriveInput = syn::parse(input).unwrap();

    // Extract path of the .larlpop file from the #[source] attribute

    let source = input.attrs
        .iter()
        .filter_map(|a| a.interpret_meta())
        .find(|im| im.name() == Ident::from("source"));

    let source = match source {
        Some(Meta::NameValue(MetaNameValue { lit: Lit::Str(s), .. })) => s.value(),
        _ => panic!(r#"Missing #[source = "..."] attribute"#),
    };

    // The module will be the name of the struct with the Parser suffix

    let modname = Ident::from(format!("{}Parser", input.ident));

    // Generate parser from larlpop file

    let generated_code = match process_file(&source) {
        Ok(c) => c,
        Err(e) => panic!("Failed to process {}: {:?}", source, e),
    };

    let expanded = quote! {
        #[allow(non_snake_case)]
        mod #modname {
            #generated_code
        }
    };

    expanded.into()
}

fn process_file<P: AsRef<Path>>(path: P) -> Result<syn::File, Box<Error>> {
    let output_dir = TempDir::new("lalrpop")?;
    let mut config = Configuration::new();
    config.set_in_dir(path.as_ref().parent().unwrap());
    config.set_out_dir(output_dir.path());
    config.process_file(path)?;

    let generated_file = match output_dir.path().read_dir()?.next() {
        Some(Ok(f)) => f.path(),
        _ => panic!("no file in output directory"),
    };

    let mut body = String::new();
    let mut file = File::open(generated_file)?;
    file.read_to_string(&mut body)?;
    Ok(syn::parse_str::<syn::File>(&body)?)
}
