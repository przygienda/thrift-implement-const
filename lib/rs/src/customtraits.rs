//! this implements trivial custom traits that are normally done by the outside module to
//! provide implementation of special traits on the generated structures.
//!
//! REASON
//! ======
//! Rust will not allow to implement a trait that is in a different module for a type
//! that it is also from different module. That catches on Vec<$name> which is considered
//! not in the same crate even if $name is. Hence the trait must be $crate running the
//! `strukt!` macros and patched in

#[macro_export]
/// example of macro implementing complete custom trait
macro_rules! custom_struct_traits {
	( $name:ident,
	 { $($fname:ident: $fty:ty => $id:expr,)+ }) => {

	 	impl $crate::customtraits::NoTrait for $name {}
	 };
	( $name:ident, {}) => {
		impl $crate::customtraits::NoTrait for $name {}
	}
}

#[macro_export]
macro_rules! custom_enum_traits {
	( $name: ident,
	{ $( $vname:ident = $val:expr, )* }) => {
		impl $ crate::customtraits::NoTrait for $ name {}
	};
}

pub trait NoTrait {

}
