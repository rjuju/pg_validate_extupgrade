/*----------------------------------------------------------------------------
 * Author: Julien Rouhaud
 * Copyright: Copyright (c) 2021-2024 : Julien Rouhaud - All rights reserved
 *---------------------------------------------------------------------------*/
use std::cmp::Ordering;
use diffy::Patch;

#[derive(Debug)]
pub enum DiffSource {
	Installed,
	Upgraded,
}

impl DiffSource {
	pub fn str_self(&self) -> &'static str {
		match self {
			DiffSource::Installed => { "installed" },
			DiffSource::Upgraded => { "upgraded" },
		}
	}

	pub fn str_other(&self) -> &'static str {
		match self {
			DiffSource::Installed => { "upgraded" },
			DiffSource::Upgraded => { "installed" },
		}
	}
}

#[derive(Debug)]
pub enum SchemaDiff<'a> {
	// (installed, upgraded)
	Diff(String, String),
	// (what, installed, upgraded)
	NamedDiff(&'a str, &'a str, &'a str),
	// (installed len, upgraded len, diffs)
	VecDiff(usize, usize, Vec<(usize, Box<SchemaDiff<'a>>)>),
	NoneDiff(DiffSource, String),
	// (installed len, upgraded len, type name, missings, diffs)
	HashMapDiff(usize, usize, &'static str,
		Vec<(DiffSource, Vec<&'a str>)>, Vec<SchemaDiff<'a>>),
	// (struct type, struct name, Vec<(Option<field>, detail)>)
	StructDiff(&'a str, &'a str, Vec<(Option<&'a str>, SchemaDiff<'a>)>),
	// (source, patch)
	UnifiedDiff(Option<String>, Patch<'a, str>),
	// extver, Vec<(guc name, new val)>
	GucDiff(String, Vec<(&'a str, &'a str)>),
}

impl<'a> SchemaDiff<'a> {
	fn indent(level: u8) -> String {
		"  ".repeat(level as usize)
	}

	fn decode(&self, level: u8) -> String {
		let ind0 = SchemaDiff::indent(level);
		let ind1 = SchemaDiff::indent(level + 1);
		let ind2 = SchemaDiff::indent(level + 2);

		match self {
			SchemaDiff::Diff(a, b) => {
				format!(
					"{i}- {}\n{i}+ {}\n\n",
					a, b, i = ind0,
				)
			},
			SchemaDiff::NamedDiff(w, a, b) => {
				format!(
					"{i}- mismatch found for {}:\n{i1}- {}\n{i1}+ {}\n\n",
					w, a, b, i = ind0, i1 = ind1,
				)
			},
			SchemaDiff::VecDiff(s1, s2, diffs) => {
				let mut res = String::new();

				match s1.cmp(s2) {
					Ordering::Less => {
						res.push_str(&format!(
								"{i}- {} has {} more elements ({}) than {} \
									({})\n",
								DiffSource::Upgraded.str_self(),
								s2 - s1,
								s2,
								DiffSource::Installed.str_self(),
								s1,
								i = ind0,
						));
					},
					Ordering::Equal => {
					//	res.push_str(&format!(
					//			"{i}- {} and {} both have {} elements\n",
					//			DiffSource::Installed.str_self(),
					//			DiffSource::Upgraded.str_self(),
					//			s1,
					//			i = ind0,
					//	));
					},
					Ordering::Greater => {
						res.push_str(&format!(
								"{i}- {} has {} more elements ({}) than {} \
									({})\n",
								DiffSource::Installed.str_self(),
								s1 - s2,
								s1,
								DiffSource::Upgraded.str_self(),
								s2,
								i = ind0,
						));
					},
				}

				for d in diffs {
					res.push_str(&format!(
							"{i}- mismatch for elem #{}:\n{}",
							d.0, d.1.decode(level + 1),
							i = ind0,
					));
				}

				res
			},
			SchemaDiff::NoneDiff(src, s) => {
				format!("{i}- {} has no value, while {} has\n{i1}+ {}\n\n",
					src.str_self(),
					src.str_other(),
					s,
					i = ind0,
					i1 = ind1,
				)
			},
			SchemaDiff::HashMapDiff(s1, s2, typname, missings, diffs) => {
				let mut res = String::new();
				match s1.cmp(s2) {
					Ordering::Less => {
						res.push_str(&format!(
								"{i}{} has {} more {} ({}) than {} ({})\n",
								DiffSource::Upgraded.str_self(),
								s2 - s1,
								typname,
								s2,
								DiffSource::Installed.str_self(),
								s1,
								i = ind0,
						));
					},
					Ordering::Equal => {
						res.push_str(&format!(
								"{i}{} and {} both have {} {} but some \
								mismatch in them:\n",
								DiffSource::Installed.str_self(),
								DiffSource::Upgraded.str_self(),
								s1,
								typname,
								i = ind0,
						));
					},
					Ordering::Greater => {
						res.push_str(&format!(
								"{i}{} has {} more {} ({}) than {} ({})\n",
								DiffSource::Installed.str_self(),
								s1 - s2,
								typname,
								s1,
								DiffSource::Upgraded.str_self(),
								s2,
								i = ind0,
						));
					},
				}

				for (s, vec) in missings {
					res.push_str(&format!("{i1}{} {} missing in {}:\n",
							vec.len(),
							typname, s.str_self(),
							i1 = ind1,
					));

					for ident in vec {
						res.push_str(&format!("{i2}- {}\n",
								ident,
								i2 = ind2,
						));
					}

					res.push('\n');
				}

				for d in diffs {
					res.push_str(&format!("{}",
							d.decode(level + 1),
					));
				}

				res
			},
			SchemaDiff::StructDiff(t, n, vec) => {
				let mut res = String::new();

				res.push_str(&format!("{i}- mismatch found for {} {}:\n",
					t, n,
					i = ind0,
				));

				for v in vec {
					match &v.0 {
						None => {
							res.push_str(&v.1.decode(level + 1));
						},
						Some(i) => {
							res.push_str(&format!("{i1}- in {}:\n{}",
									i,
									v.1.decode(level + 2),
									i1 = ind1,
							));
						},
					}
				}
				res
			},
			SchemaDiff::UnifiedDiff(s, patch) => {
				let mut diff = patch.to_string();
				// XXX I didn't find any API to specify the filenames
				diff.replace_range(.."--- original\n+++modified".len() + 1,
						&format!("--- {}\n+++ {}",
							DiffSource::Installed.str_self(),
							DiffSource::Upgraded.str_self(),
							));
				let info = match s {
					Some(s) => format!("{}\n", s),
					None => String::from(""),
				};

				format!("{}{}\n", info, diff)
			},
			SchemaDiff::GucDiff(extver, vec) => {
				let mut res = String::new();

				res.push_str(&format!("Some GUC changes leaked the script \
					for version {}:\n", extver));

				for (guc, val) in vec {
					res.push_str(&format!(
						"{i} - {} changed to: {}\n",
						guc, val, i = ind0,
					));
				}

				res.push_str("\n");

				res
			}
		}
	}
}

impl<'a> ToString for SchemaDiff<'a> {
	fn to_string(&self) -> String {
		self.decode(0)
	}
}
