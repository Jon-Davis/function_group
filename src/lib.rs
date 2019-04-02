#[macro_export] macro_rules! function_group {
    // get the visibility of the function and add it to the trait and func, call @Name
    (@Visibilty ($vis:vis fn $($tail:tt)*) -> (($($trait:tt)*), ($($func:tt)*))) => {
        function_group!(@Name (fn $($tail)*) -> (($($trait)* $vis), ($($func)* $vis)));
    };
    (@Visibilty (fn $($tail:tt)*) -> (($($trait:tt)*), ($($func:tt)*))) => {
        function_group!(@Name (fn $($tail)*) -> (($($trait)*), ($($func)*)));
    };
    // get the name of the function and add it to the trait and func, while passing it down to @ReturnType
    (@Name (fn $name:ident $($tail:tt)*) -> (($($trait:tt)*), ($($func:tt)*))) => {
        function_group!(@ReturnType ($name $($tail)*) -> (($($trait)* trait $name), ($($func)* fn $name<T : $name>)));
    };
    // get the return type of the function and passthrough the name and return type do @Define
    (@ReturnType ($name:ident -> $ret:ty {$($tail:tt)*}) -> (($($trait:tt)*), ($($func:tt)*))) => {
        function_group!(@Define ($name, $ret, {$($tail)*}) -> (($($trait)*), ($($func)*)));
    };
    (@ReturnType ($name:ident {$($tail:tt)*}) -> (($($trait:tt)*), ($($func:tt)*))) => {
        function_group!(@Define ($name, (), {$($tail)*}) -> (($($trait)*), ($($func)*)));
    };
    // construct the trait and function and call @Implementations
    (@Define ($name:ident, $ret:ty, {$($tail:tt)*}) -> (($($trait:tt)*), ($($func:tt)*))) => {
        #[allow(non_camel_case_types)]
        $($trait)*{
            fn $name(self) -> $ret;
        }
        $($func)*(a : T) -> $ret { 
            a.$name() 
        }
        function_group!(@Implementations ($name, $ret, $($tail)*));
    };
    // Implementations fills out the different functions, this is the final step
    (@Implementations ($name:ident, $ret:ty, $(($( $($var:ident)* : $type:ty),*) $code:block $(;)*)*)) => {
        $(impl $name for ($($type,)*){
            fn $name(self) -> $ret {
                let ($($($var)*,)*) = self;
                $code
            }
        })*
    };
    // This is simply to catch errors
    (@Implementations ($($tail:tt)*)) => {
        $($tail)*
    };
    // start the macro from the top @Visibility
    ($($tail:tt)*) => {
        function_group!(@Visibilty ($($tail)*) -> ((), ()));
    };
}
