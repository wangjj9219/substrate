// This file is part of Substrate.

// Copyright (C) 2018-2020 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Macros that define an Origin type. Every function call to your runtime has an origin which
//! specifies where the extrinsic was generated from.

/// Constructs an Origin type for a runtime. This is usually called automatically by the
/// construct_runtime macro. See also __create_decl_macro.
#[macro_export]
macro_rules! impl_outer_origin {

	// Macro transformations (to convert invocations with incomplete parameters to the canonical
	// form)
	(
		$(#[$attr:meta])*
		pub enum $name:ident for $runtime:ident {
			$( $rest_without_system:tt )*
		}
	) => {
		$crate::impl_outer_origin! {
			$(#[$attr])*
			pub enum $name for $runtime where system = system {
				$( $rest_without_system )*
			}
		}
	};

	(
		$(#[$attr:meta])*
		pub enum $name:ident for $runtime:ident where system = $system:ident {
			$( $rest_with_system:tt )*
		}
	) => {
		$crate::paste::item! {
			$crate::impl_outer_origin!(
				$( #[$attr] )*;
				$name;
				[< $name Caller >];
				$runtime;
				$system;
				Modules { $( $rest_with_system )* };
			);
		}
	};

	// Generic + Instance
	(
		$(#[$attr:meta])*;
		$name:ident;
		$caller_name:ident;
		$runtime:ident;
		$system:ident;
		Modules {
			$module:ident $instance:ident <T>
			$(, $( $rest_module:tt )* )?
		};
		$( $parsed:tt )*
	) => {
		$crate::impl_outer_origin!(
			$( #[$attr] )*;
			$name;
			$caller_name;
			$runtime;
			$system;
			Modules { $( $( $rest_module )* )? };
			$( $parsed )* $module <$runtime> { $instance },
		);
	};

	// Instance
	(
		$(#[$attr:meta])*;
		$name:ident;
		$caller_name:ident;
		$runtime:ident;
		$system:ident;
		Modules {
			$module:ident $instance:ident
			$(, $rest_module:tt )*
		};
		$( $parsed:tt )*
	) => {
		$crate::impl_outer_origin!(
			$( #[$attr] )*;
			$name;
			$caller_name;
			$runtime;
			$system;
			Modules { $( $rest_module )* };
			$( $parsed )* $module { $instance },
		);
	};

	// Generic
	(
		$(#[$attr:meta])*;
		$name:ident;
		$caller_name:ident;
		$runtime:ident;
		$system:ident;
		Modules {
			$module:ident <T>
			$(, $( $rest_module:tt )* )?
		};
		$( $parsed:tt )*
	) => {
		$crate::impl_outer_origin!(
			$( #[$attr] )*;
			$name;
			$caller_name;
			$runtime;
			$system;
			Modules { $( $( $rest_module )* )? };
			$( $parsed )* $module <$runtime>,
		);
	};

	// No Generic and no Instance
	(
		$(#[$attr:meta])*;
		$name:ident;
		$caller_name:ident;
		$runtime:ident;
		$system:ident;
		Modules {
			$module:ident
			$(, $( $rest_module:tt )* )?
		};
		$( $parsed:tt )*
	) => {
		$crate::impl_outer_origin!(
			$( #[$attr] )*;
			$name;
			$caller_name;
			$runtime;
			$system;
			Modules { $( $( $rest_module )* )? };
			$( $parsed )* $module,
		);
	};

	// The main macro expansion that actually renders the Origin enum code.
	(
		$(#[$attr:meta])*;
		$name:ident;
		$caller_name:ident;
		$runtime:ident;
		$system:ident;
		Modules { };
		$( $module:ident $( < $generic:ident > )? $( { $generic_instance:ident } )? ,)*
	) => {
		#[derive(Clone)]
		pub struct $name {
			caller: $caller_name,
			filter: $crate::sp_std::rc::Rc<Box<dyn Fn(&<$runtime as $system::Trait>::Call) -> bool>>,
		}

		#[cfg(not(feature = "std"))]
		impl $crate::sp_std::fmt::Debug for $name {
			fn fmt(
				&self,
				fmt: &mut $crate::sp_std::fmt::Formatter
			) -> $crate::sp_std::result::Result<(), $crate::sp_std::fmt::Error> {
				fmt.write_str("<wasm:stripped>")
			}
		}

		#[cfg(feature = "std")]
		impl $crate::sp_std::fmt::Debug for $name {
			fn fmt(
				&self,
				fmt: &mut $crate::sp_std::fmt::Formatter
			) -> $crate::sp_std::result::Result<(), $crate::sp_std::fmt::Error> {
				fmt.debug_struct(stringify!($name))
					.field("caller", &self.caller)
					.field("filter", &"[function ptr]")
					.finish()
			}
		}

		impl $crate::traits::OriginTrait for $name {
			type Call = <$runtime as $system::Trait>::Call;
			type PalletsOrigin = $caller_name;

			fn add_filter(&mut self, add_filter: impl Fn(&Self::Call) -> bool + 'static) {
				let no_op = $crate::sp_std::rc::Rc::new(Box::new(|_: &Self::Call| true)
					as Box<dyn Fn(&Self::Call) -> bool>);

				let f = $crate::sp_std::mem::replace(&mut self.filter, no_op);

				self.filter = $crate::sp_std::rc::Rc::new(Box::new(move |call| {
					f(call) && add_filter(call)
				}));
			}

			fn reset_filter(&mut self) {
				let filter = <
					<$runtime as $system::Trait>::BasicCallFilter
					as $crate::traits::Filter<<$runtime as $system::Trait>::Call>
				>::filter;

				self.filter = $crate::sp_std::rc::Rc::new(Box::new(filter));
			}

			fn set_caller_from(&mut self, other: impl Into<Self>) {
				self.caller = other.into().caller
			}

			fn filter_call(&self, call: &Self::Call) -> bool {
				(self.filter)(call)
			}
		}

		$crate::paste::item! {
			#[derive(Clone, PartialEq, Eq, $crate::RuntimeDebug)]
			$(#[$attr])*
			#[allow(non_camel_case_types)]
			pub enum $caller_name {
				system($system::Origin<$runtime>),
				$(
					[< $module $( _ $generic_instance )? >]
					($module::Origin < $( $generic, )? $( $module::$generic_instance )? > ),
				)*
				#[allow(dead_code)]
				Void($crate::Void)
			}
		}

		#[allow(dead_code)]
		impl $name {
			pub fn none() -> Self {
				let mut o = $name {
					caller: $caller_name::system($system::RawOrigin::None),
					filter: $crate::sp_std::rc::Rc::new(Box::new(|_| true)),
				};
				$crate::traits::OriginTrait::reset_filter(&mut o);
				o
			}
			pub fn root() -> Self {
				let mut o = $name {
					caller: $caller_name::system($system::RawOrigin::Root),
					filter: $crate::sp_std::rc::Rc::new(Box::new(|_| true)),
				};
				$crate::traits::OriginTrait::reset_filter(&mut o);
				o
			}
			pub fn signed(by: <$runtime as $system::Trait>::AccountId) -> Self {
				let mut o = $name {
					caller: $caller_name::system($system::RawOrigin::Signed(by)),
					filter: $crate::sp_std::rc::Rc::new(Box::new(|_| true)),
				};
				$crate::traits::OriginTrait::reset_filter(&mut o);
				o
			}
		}

		impl From<$system::Origin<$runtime>> for $name {
			fn from(x: $system::Origin<$runtime>) -> Self {
				let mut o = $name {
					caller: $caller_name::system(x),
					filter: $crate::sp_std::rc::Rc::new(Box::new(|_| true)),
				};
				$crate::traits::OriginTrait::reset_filter(&mut o);
				o
			}
		}
		impl Into<$crate::sp_std::result::Result<$system::Origin<$runtime>, $name>> for $name {
			fn into(self) -> $crate::sp_std::result::Result<$system::Origin<$runtime>, Self> {
				if let $caller_name::system(l) = self.caller {
					Ok(l)
				} else {
					Err(self)
				}
			}
		}
		impl From<Option<<$runtime as $system::Trait>::AccountId>> for $name {
			fn from(x: Option<<$runtime as $system::Trait>::AccountId>) -> Self {
				<$system::Origin<$runtime>>::from(x).into()
			}
		}
		$(
			$crate::paste::item! {
				impl From<$module::Origin < $( $generic )? $(, $module::$generic_instance )? > > for $name {
					fn from(x: $module::Origin < $( $generic )? $(, $module::$generic_instance )? >) -> Self {
						let mut o = $name {
							caller: $caller_name::[< $module $( _ $generic_instance )? >](x),
							filter: $crate::sp_std::rc::Rc::new(Box::new(|_| true)),
						};
						$crate::traits::OriginTrait::reset_filter(&mut o);
						o
					}
				}
				impl Into<
					$crate::sp_std::result::Result<
						$module::Origin < $( $generic )? $(, $module::$generic_instance )? >,
						$name,
					>>
				for $name {
					fn into(self) -> $crate::sp_std::result::Result<
						$module::Origin < $( $generic )? $(, $module::$generic_instance )? >,
						Self,
					> {
						if let $caller_name::[< $module $( _ $generic_instance )? >](l) = self.caller {
							Ok(l)
						} else {
							Err(self)
						}
					}
				}
			}
		)*
	}
}

#[cfg(test)]
mod tests {
	use crate::traits::{Filter, OriginTrait};
	mod system {
		pub trait Trait {
			type AccountId;
			type Call;
			type BasicCallFilter;
		}

		#[derive(Clone, PartialEq, Eq, Debug)]
		pub enum RawOrigin<AccountId> {
			Root,
			Signed(AccountId),
			None,
		}

		impl<AccountId> From<Option<AccountId>> for RawOrigin<AccountId> {
			fn from(s: Option<AccountId>) -> RawOrigin<AccountId> {
				match s {
					Some(who) => RawOrigin::Signed(who),
					None => RawOrigin::None,
				}
			}
		}

		pub type Origin<T> = RawOrigin<<T as Trait>::AccountId>;
	}

	mod origin_without_generic {
		#[derive(Clone, PartialEq, Eq, Debug)]
		pub struct Origin;
	}

	mod origin_with_generic {
		#[derive(Clone, PartialEq, Eq, Debug)]
		pub struct Origin<T> {
			t: T
		}
	}

	#[derive(Clone, PartialEq, Eq, Debug)]
	pub struct TestRuntime;

	pub struct BasicCallFilter;
	impl Filter<u32> for BasicCallFilter {
		fn call_filter() -> Box<dyn Fn(&u32) -> bool> {
			Box::new(|a| *a % 2 == 0)
		}
	}

	impl system::Trait for TestRuntime {
		type AccountId = u32;
		type Call = u32;
		type BasicCallFilter = BasicCallFilter;
	}

	impl_outer_origin!(
		pub enum OriginWithoutSystem for TestRuntime {
			origin_without_generic,
			origin_with_generic<T>,
		}
	);

	impl_outer_origin!(
		pub enum OriginWithoutSystem2 for TestRuntime {
			origin_with_generic<T>,
			origin_without_generic
		}
	);

	impl_outer_origin!(
		pub enum OriginWithSystem for TestRuntime where system = system {
			origin_without_generic,
			origin_with_generic<T>
		}
	);

	impl_outer_origin!(
		pub enum OriginWithSystem2 for TestRuntime where system = system {
			origin_with_generic<T>,
			origin_without_generic,
		}
	);

	impl_outer_origin!(
		pub enum OriginEmpty for TestRuntime where system = system {}
	);

	#[test]
	fn test_default_filter() {
		assert_eq!(OriginWithSystem::root().filter_call(&0), true);
		assert_eq!(OriginWithSystem::root().filter_call(&1), false);
		assert_eq!(OriginWithSystem::none().filter_call(&0), true);
		assert_eq!(OriginWithSystem::none().filter_call(&1), false);
		assert_eq!(OriginWithSystem::signed(0).filter_call(&0), true);
		assert_eq!(OriginWithSystem::signed(0).filter_call(&1), false);
		assert_eq!(OriginWithSystem::from(Some(0)).filter_call(&0), true);
		assert_eq!(OriginWithSystem::from(Some(0)).filter_call(&1), false);
		assert_eq!(OriginWithSystem::from(None).filter_call(&0), true);
		assert_eq!(OriginWithSystem::from(None).filter_call(&1), false);
		assert_eq!(OriginWithSystem::from(origin_without_generic::Origin).filter_call(&0), true);
		assert_eq!(OriginWithSystem::from(origin_without_generic::Origin).filter_call(&1), false);
	}
}
