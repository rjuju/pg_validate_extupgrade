/*----------------------------------------------------------------------------
 * Author: Julien Rouhaud
 * Copyright: Copyright (c) 2021 : Julien Rouhaud - All rights reserved
 *---------------------------------------------------------------------------*/
use std::collections::HashMap;

pub trait Compare {
	fn compare(&self, other: &Self, msg: &mut String);
	fn typname() -> &'static str {
		panic!("Should not be called.");
	}
}

impl Compare for String {
	fn compare(&self, other: &String, msg: &mut String) {
		if self != other {
			msg.push_str(&format!("\t- {}\n\t+ {}", self, other));
		}
	}
}

impl<T: Compare> Compare for Vec<T> {
	fn compare(&self, other: &Vec<T>, msg: &mut String) {

		for (i,(a,b)) in self.iter().zip(other.iter()).enumerate() {
			let mut res = String::new();
			a.compare(b, &mut res);
			if res != "" {
				msg.push_str(&format!("elem {}: {}", i, res));
			}
		}
	}

	fn typname() -> &'static str {
		<T>::typname()
	}
}

impl<T: Compare> Compare for Option<T> {
	fn compare(&self, other: &Option<T>, msg: &mut String) {
		if self.is_none() && !other.is_none() {
			let res = format!("installed has no {}, while upgraded has",
				<T>::typname(),
				);
			msg.push_str(&res);
			return;
		}

		if !self.is_none() && other.is_none() {
			let res = format!("Upgraded has no {}, while installed has",
				<T>::typname(),
				);
			msg.push_str(&res);
			return;
		}

		if self.is_none() && other.is_none() {
			return;
		}

		let src = self.as_ref().unwrap();
		let dst = other.as_ref().unwrap();

		src.compare(dst, msg);
	}

	fn typname() -> &'static str {
		<T>::typname()
	}
}

impl<T: Compare> Compare for HashMap<String, T> {
	fn compare(&self, other: &HashMap<String, T>, msg: &mut String) {
		if self.len() < other.len() {
			let mut res = format!("Upgraded has {} more {t} than installed\n\
				Missing {t}:\n",
				other.len() - self.len(),
				t = <T>::typname(),
			);

			for ident in other.keys() {
				if !self.contains_key(ident) {
					res.push_str(&format!("\t- {}\n", ident));
				}
			}

			msg.push_str(&res);
			return;
		}

		if self.len() > other.len() {
			let mut res = format!("Installed version has {} more {t} than \
				upgraded\nMissing {t}:\n",
				self.len() - other.len(),
				t = <T>::typname(),
			);

			for ident in self.keys() {
				if !other.contains_key(ident) {
					res.push_str(&format!("\t- {}\n", ident));
				}
			}

			msg.push_str(&res);
			return;
		}


		let mut missing = HashMap::new();
		let mut res = String::new();
		let mut tmp = String::new();

		// Find missing or different objects in upgraded version
		for (n, r) in self {
			let other = other.get(n);

			if other.is_none() {
				tmp.push_str(&format!("\t- {}\n", n));
			} else {
				let other = other.unwrap();
				r.compare(other, &mut res);
			}
		}
		missing.insert("installed", tmp);

		// Find missing objects in installed version.  Different objects are
		// already checked.
		tmp = String::new();
		for (n, _) in other {
			let src = self.get(n);

			if src.is_none() {
				tmp.push_str(&format!("\t- {}\n", n));
			}
		}
		missing.insert("upgraded", tmp);

		for (k, v) in missing {
			if v != "" {
				msg.push_str(&format!("Missing {} in {} version:\n",
						<T>::typname(), k));
				msg.push_str(&v);
				msg.push('\n');
			}
		}

		if res != "" {
			msg.push_str(&res);
		}
	}

	fn typname() -> &'static str {
		<T>::typname()
	}
}

#[macro_export]
macro_rules! DbStruct {
	($struct:ident {$( $field:ident:$type:ty ),*,}) => {
		#[derive(Debug)]
		pub struct $struct {
			ident: String,
			$($field: $type),*
		}

		impl Compare for $struct {
			fn compare(&self, other: &Self, msg: &mut String) {
				$(
					let mut res = String::new();
					self.$field.compare(&other.$field, &mut res);
					if res != "" {
						msg.push_str(&format!(
								"Mismatch found for {} {} in {}:\n{}\n",
								&stringify!($struct).to_string(),
								&self.ident,
								&stringify!($field).to_string(),
								&res,
							));
					}
				)*
			}

			fn typname() -> &'static str {
				stringify!($struct)
			}
		}
	};
}
