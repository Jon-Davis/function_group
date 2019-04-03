
extern crate proc_macro;
use self::proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream, Result};
use syn::{parenthesized, braced, parse_macro_input, Block, Ident, Token, token::Paren, Type, Visibility};

struct Function {
    argument_idents : Vec<Ident>,
    argument_types: Vec<Type>,
    arugment_mutability : Vec<bool>,
    body : Block,
}

struct FunctionGroup {
    visibility : Visibility,    // The visibility of the function
    name: Ident,                // The name of the function and trait to be defined
    output: Option<Type>,       // the output the function will produce
    self_input : Option<Type>,  // the self type that will be passed in if method i.e. &self, self, &mut self
    self_type : Option<Type>,   // the type self will refer to, for example some user type Foo
    self_mut : bool,
    functions : Vec<Function>,  // the functions that will be implemented  
}

impl Parse for Function {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut argument_idents = vec!();
        let mut argument_types = vec!();
        let mut arugment_mutability = vec!();
        let content;
        parenthesized!(content in input);
        loop {
            let lookahead = content.lookahead1();
            if lookahead.peek(Token![mut]){
                content.parse::<Token![mut]>()?;
                arugment_mutability.push(true)
            } else {
                arugment_mutability.push(false)
            }
            let lookahead = content.lookahead1();
            if !lookahead.peek(Ident) {
                break;
            }
            argument_idents.push(content.parse()?);
            content.parse::<Token![:]>()?;
            argument_types.push(content.parse()?);
            let lookahead = content.lookahead1();
            if lookahead.peek(Token![,]) {
                content.parse::<Token![,]>()?;
            }
        }

        let body : Block = input.parse()?;
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![;]) {
            input.parse::<Token![;]>()?;
        }
        Ok(Function{argument_idents, argument_types, arugment_mutability, body})
    }
}

impl Parse for FunctionGroup {
    fn parse(input: ParseStream) -> Result<Self> {
        // parse the visibilty of the functions
        let visibility: Visibility = input.parse()?;
        input.parse::<Token![fn]>()?;

        // parse the name of the functions
        let name: Ident = input.parse()?;

        // optionally parse the self type and input of the functions
        let lookahead = input.lookahead1();
        let mut self_input : Option<Type> = None;
        let mut self_type : Option<Type> = None;
        let mut self_mut : bool = false;
        if lookahead.peek(Paren) {
            let content;
            parenthesized!(content in input);
            let lookahead = content.lookahead1();
            if lookahead.peek(Token![mut]){
                content.parse::<Token![mut]>()?;
                self_mut = true;
            } else {
                self_mut = false;
            }
            self_input = Some(content.parse()?);
            content.parse::<Token![:]>()?;
            self_type = Some(content.parse()?);
        }

        // optionally parse the output
        let lookahead = input.lookahead1();
        let mut output : Option<Type> = None;
        if lookahead.peek(Token![-]) {
            input.parse::<Token![-]>()?;
            input.parse::<Token![>]>()?;
            output = Some(input.parse()?);
        }

        // parse all the internal functions
        let mut functions : Vec<Function> = vec!();
        let content;
        braced!(content in input);
        loop {
            let lookahead = content.lookahead1();
            if !lookahead.peek(Paren) {
                break;
            }
            functions.push(content.parse()?);
        }

        Ok(FunctionGroup {visibility,name,output,self_input,self_type,self_mut,functions})
    }
}

/// The function_group macro is used to create a single function that accepts mutltiple
/// types of arguments. 
/// Function groups can take multiple types of arguments and even be recursive, the general form
/// of the function_group macro is displayed below.
/// ```
/// function_group! {
///     fn name_of_function -> return_type_of_function {
///         (argument_name_of_function : arugment_type_of_function...) {
///             // body of function
///         }
///         (argument_name_of_function : arugment_type_of_function...) {
///             // body of function
///         }
///         ...
///     }
/// }
/// ```
/// 
/// Function groups can also be declared on types to do this there is an additonal parameter that comes after
/// the function name and follows the form of ((&self | self | &mut self) : TypeTheFunctionWillBeImplementedOn).
/// for example:
/// ```
/// function_group! {
///     fn name_of_function(self : AwsomeStruct) {
///         (argument_name_of_function : arugment_type_of_function...) {
///             // body of function
///         }
///         (argument_name_of_function : arugment_type_of_function...) {
///             // body of function
///         }
///         ...
///     }
/// }
/// ```
#[proc_macro]
pub fn function_group(input: TokenStream) -> TokenStream {
    let function_group : FunctionGroup = parse_macro_input!(input as FunctionGroup);

    if function_group.self_input.is_some() && function_group.self_type.is_some() {
        return function_group_method(function_group);
    }
    return function_group_fn(function_group);
}

// creates a function group for use with structs
fn function_group_method(group : FunctionGroup) -> TokenStream {
    let FunctionGroup {visibility,name,output,self_input,self_type,self_mut,functions} = group;

    // if there hasn't been an output type, set it to the unit type, else use the output type
    let output = if output.is_none() {
        quote!{ () }
    } else {
        quote!{ #output }
    };

    let self_input = self_input.unwrap();
    let self_input = if self_mut {
        quote!{mut #self_input}
    } else {
        quote!{#self_input}
    };
    let self_type = self_type.unwrap();

    // create the trait that will be used to accept multiple types
    let group_trait = quote! {
        #[allow(non_camel_case_types)]
        #visibility trait #name<Arg> {
            fn #name(#self_input, _args : Arg) -> #output;
        }
    };

     // get the types for the function
    let mut group_impl = quote!{};
    for function in functions {
        // get the types of each argument
        let mut func_types = quote!{};
        for fn_type in function.argument_types{
            func_types = quote!{ #func_types #fn_type,};
        }
        // get the idents of each argument
        let mut func_idents = quote!{};
        for (fn_ident, fn_mut) in function.argument_idents.iter().zip(function.arugment_mutability.iter())  {
            if *fn_mut {
                func_idents = quote!{ #func_idents mut #fn_ident,};
            } else {
                func_idents = quote!{ #func_idents #fn_ident,};
            }
        }
        // get the user block for each function
        let func_block = function.body;
        // implement the trait for the input arguments as tuples
        group_impl = quote! {
            #group_impl
            impl #name<(#func_types)> for #self_type {
                fn #name(#self_input, (#func_idents) : (#func_types)) -> #output {
                    #func_block
                }
            }
        }
    }

    let expanded = quote! {
        #group_trait
        #group_impl
    };

    TokenStream::from(expanded)
}

/// creates a function group for use outside of structs
fn function_group_fn(group : FunctionGroup) -> TokenStream {
    let FunctionGroup {visibility,name,output,self_input: _,self_type: _,self_mut: _,functions} = group;

    // if there hasn't been an output type, set it to the unit type, else use the output type
    let output = if output.is_none() {
        quote!{ () }
    } else {
        quote!{ #output }
    };

    // create the trait that will be used to accept multiple types
    let group_trait = quote! {
        #[allow(non_camel_case_types)]
        #visibility trait #name {
            fn #name(self) -> #output;
        }
    };

    // create the generic function that will be called by the user
    let group_fn = quote! {
        #visibility fn #name<T: #name>(a : T) -> #output {
            a.#name()
        }
    };

    // get the types for the function
    let mut group_impl = quote!{};
    for function in functions {
        // get the types of each argument
        let mut func_types = quote!{};
        for fn_type in function.argument_types {
            func_types = quote!{ #func_types #fn_type,};
        }
        // get the idents of each argument
        let mut func_idents = quote!{};
        for (fn_ident, fn_mut) in function.argument_idents.iter().zip(function.arugment_mutability.iter())  {
            if *fn_mut {
                func_idents = quote!{ #func_idents mut #fn_ident,};
            } else {
                func_idents = quote!{ #func_idents #fn_ident,};
            }
        }
        // get the user block for each function
        let func_block = function.body;
        // implement the trait for the input arguments as tuples
        group_impl = quote! {
            #group_impl
            impl #name for (#func_types) {
                fn #name(self) -> #output {
                    let (#func_idents) = self;
                    #func_block
                }
            }
        }
    }

    // put it all together
    let expanded = quote! {
        #group_trait
        #group_fn
        #group_impl
    };

    // export
    TokenStream::from(expanded)
}