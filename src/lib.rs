/*! The function_group macro is used to create a single function that accepts mutltiple
 types of arguments. 
 Function groups can take multiple types of arguments and even be recursive, the general form
 of the function_group macro is displayed below.
 # Syntax
 ```
 function_group! {
     fn name_of_function -> return_type_of_function {
         (argument_name_of_function : arugment_type_of_function...) {
             // body of function
         }
         (argument_name_of_function : ArgumentName...) {
             // body of function
         }
         ...
     }
 }
 ```
 
 Function groups can also be declared on types to do this there is an additonal parameter that comes after
 the function name and follows the form of ((&self | self | &mut self) : TypeTheFunctionWillBeImplementedOn).
 for example:
 ```
 function_group! {
     fn name_of_function(self : AwsomeStruct) {
         (argument_name_of_function : ArgumentName ...) {
             // body of function
         }
         (argument_name_of_function : ArgumentName...) {
             // body of function
         }
         ...
     }
 }
 ```
*/

extern crate proc_macro;
use self::proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream, Result};
use syn::{parenthesized, braced, parse_macro_input, Block, Ident, Token, token::Paren, Type, Visibility};

// Represents a single sub-function in the function_group macro
struct Function {
    argument_idents : Vec<Ident>,   // The names of the input arguments
    argument_types: Vec<Type>,      // The types of the input arguments
    arugment_mutability : Vec<bool>,// The mutability of the input arguments
    body : Block,                   // The user code for the function
}

// The main parse struct of the function_group macro
struct FunctionGroup {
    visibility : Visibility,    // The visibility of the function
    name: Ident,                // The name of the function and trait to be defined
    output: Option<Type>,       // the output the function will produce
    self_input : Option<Type>,  // the self type that will be passed in if method i.e. &self, self, &mut self
    self_type : Option<Type>,   // the type self will refer to, for example some user type Foo
    self_mut : bool,
    functions : Vec<Function>,  // the functions that will be implemented  
}

// The Parse implementation of a Function parses the (arg1 : Type1, arg2 : Type2...) {} part of the macro
impl Parse for Function {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut argument_idents = vec!();
        let mut argument_types = vec!();
        let mut arugment_mutability = vec!();
        // Each set of arguments are surrounded by parens
        let content;
        parenthesized!(content in input);
        // loop since each function can have multiple arguments 
        loop {
            // check to see if the mut ident is ahead
            let lookahead = content.lookahead1();
            if lookahead.peek(Token![mut]){
                // if the mut ident was found, mark this argument as being mutable
                content.parse::<Token![mut]>()?;
                arugment_mutability.push(true)
            } else {
                // if the mut ident was not found, mark this argument as not being mutable
                arugment_mutability.push(false)
            }

            // check to see if there is an identifier, if there is, continue on the with
            // adding the argument, otherwise, the last identifier was found last iteration
            // so terminate the loop
            let lookahead = content.lookahead1();
            if !lookahead.peek(Ident) {
                break;
            }
            argument_idents.push(content.parse()?);
            // after every argument name there is a : 
            content.parse::<Token![:]>()?;
            // after the colon there is a type denoting the variable type
            argument_types.push(content.parse()?);
            let lookahead = content.lookahead1();
            // there may or may not be a comma, usually if there is no comma, that means the last
            // argument has been read, but this statement doesn't make that assumption
            if lookahead.peek(Token![,]) {
                content.parse::<Token![,]>()?;
            }
        }

        // The code block that is ment to be executed as the body
        // of the function comes next
        let body : Block = input.parse()?;

        // If there happens to be a semicolon after each pattern, ignore it
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![;]) {
            input.parse::<Token![;]>()?;
        }

        // return the Function struct
        Ok(Function{argument_idents, argument_types, arugment_mutability, body})
    }
}

// The Parse function for parse group gets the name, visibility, self params, and return type
// then calls the Function parse function for each sub-function it finds
impl Parse for FunctionGroup {
    fn parse(input: ParseStream) -> Result<Self> {
        // parse the visibilty of the functions
        let visibility: Visibility = input.parse()?;
        input.parse::<Token![fn]>()?;

        // parse the name of the functions
        let name: Ident = input.parse()?;

        // optionally parse the self type and input of the functions
        let lookahead = input.lookahead1();
        // self needs both an input ie &self and a Type ie T to implement the methods
        let mut self_input : Option<Type> = None;
        let mut self_type : Option<Type> = None;
        let mut self_mut : bool = false;
        // The self params are optional, if they are there the function group is a function,
        // otherwise it is a method
        if lookahead.peek(Paren) {
            // The Self params are wraped in Parens
            let content;
            parenthesized!(content in input);
            // It is possible that the mut ident will come before self, although this might actually
            // be becoming deprecated in rust, however the macro still acounts for it
            let lookahead = content.lookahead1();
            if lookahead.peek(Token![mut]){
                content.parse::<Token![mut]>()?;
                self_mut = true;
            } else {
                self_mut = false;
            }
            // get the input type ie self | &self | &mut self
            self_input = Some(content.parse()?);
            // colon comes after input
            content.parse::<Token![:]>()?;
            // get the type this method is being implemented on
            self_type = Some(content.parse()?);
        }

        // optionally parse the output, if there is none assume the output is the unit type
        let lookahead = input.lookahead1();
        let mut output : Option<Type> = None;
        // output types are defined by the -> tokens followed by the output type
        if lookahead.peek(Token![-]) {
            input.parse::<Token![-]>()?;
            input.parse::<Token![>]>()?;
            output = Some(input.parse()?);
        }

        // parse all the internal functions
        let mut functions : Vec<Function> = vec!();
        // all the subfunctions are stored in a block
        let content;
        braced!(content in input);
        // loop because an arbitrary number of subfunctions are allowed
        loop {
            // sub functions always start with a paren to denote the expected inputs
            let lookahead = content.lookahead1();
            if !lookahead.peek(Paren) {
                break;
            }
            // call the Functions parse function
            functions.push(content.parse()?);
        }
        // Return the FunctionGroup
        Ok(FunctionGroup {visibility,name,output,self_input,self_type,self_mut,functions})
    }
}

///
/// Function groups can take multiple types of arguments and even be recursive.
/// ```rust
/// function_group! {
///     fn add -> usize {
///         (one : usize, two : usize) {
///             one + two
///         }
///         (one : usize, two : usize, three: usize) {
///             add((one, two)) + three
///         }
///     }
/// }
///
/// assert!(add((5, 5)) == 10);
/// assert!(add((5, 5, 5)) == 15);
/// ```
///
/// The arguments can be mutable or immutable refrences.
/// ```rust
/// function_group! {
///     fn add_to {
///         (one : &mut usize, two : usize) {
///             *one += two;
///         }
///         (one : &mut usize, two : usize, three : usize) {
///             *one += two + three;
///         }
///     }
/// }
///
/// let mut x = 10;
/// add_to((&mut x, 5));
/// add_to((&mut x, 5, 5));
/// assert!(x == 25);
/// ```
///
/// Function Groups can even be associated with a Type. In the example below, each sub function will be passed a mutable refrence to self, and these functions will be usable by the TestStruct type.
/// ```rust
/// struct TestStruct(usize);
/// function_group! {
///     fn add_to_struct(&mut self : TestStruct) {
///         (one : usize) {
///             self.0 += one;
///         }
///         (one : usize, two : usize){
///             self.0 += one + two; 
///         }
///     }
/// }
///
/// let mut x = TestStruct(10);
/// x.add_to_struct((1,2));
/// assert!(x.0 == 13);
/// ```
#[proc_macro]
pub fn function_group(input: TokenStream) -> TokenStream {
    // Call the FunctionGroup Parse function
    let function_group : FunctionGroup = parse_macro_input!(input as FunctionGroup);

    // If the self types are defined, then the macro should generate a method on the entered type
    if function_group.self_input.is_some() && function_group.self_type.is_some() {
        return function_group_method(function_group);
    }
    // Otherwise the macro should generate stand alone functions
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

    // Since these types are wraped in options, unwrap them or set a defualt type if None
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

    // Expand the quotes to get the final output
    let expanded = quote! {
        #group_trait
        #group_impl
    };

    // Return the final output
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