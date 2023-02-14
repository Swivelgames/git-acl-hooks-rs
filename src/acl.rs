use regex::Regex;
use std::fs;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
pub enum Access {
	ReadWrite,
	ReadOnly,
}

#[derive(Debug)]
pub struct AccessControl {
	pub match_regex: Regex,
	pub access: Access,
	pub users: Vec<String>,
}

impl AccessControl {
	fn new(match_regex: Regex, access: Access, users: Vec<String>) -> AccessControl {
		AccessControl {
			match_regex,
			access,
			users,
		}
	}
}

fn parse_line(line: &str) -> (String, String) {
	let mut parts = line.splitn(2, '=');
	let key = parts.next().unwrap().trim();
	let value = parts.next().unwrap().trim();

	let read_len = value.find('#').unwrap_or(value.len());
	let value = value[..read_len].trim();

	(key.to_owned(), value.to_owned())
}

fn process_default_var(line: &str) -> Access {
	let (key, value) = parse_line(line);

	match key.as_ref() {
		"default" => {
			match value.as_ref() {
				"read-write" => Access::ReadWrite,
				"read-only" => Access::ReadOnly,
				_ => panic!("Unexpected default value: {}", value),
			}
		}
		_ => {
			panic!("Unexpected key outside of ACL section: {}", key);
		}
	}
}

fn process_acl_var(current_acl: &mut AccessControl, line: &str) {
	let (key, value) = parse_line(line);

	match key.as_ref() {
		"access" => {
			let access = match value.as_ref() {
				"read-write" => Access::ReadWrite,
				"read-only" => Access::ReadOnly,
				_ => panic!("Unexpected access value: {}", value),
			};
			current_acl.access = access;
		}
		"users" => {
			let users: Vec<String> = value.split(' ').map(|s| s.to_string()).collect();
			current_acl.users = users;
		}
		_ => {
			panic!("Unexpected key: {}", key);
		}
	}
}

pub fn create_acl_map(acl_file: &str) -> Vec<AccessControl> {
	let file = fs::File::open(acl_file).expect("file not found");
	let reader = BufReader::new(file);

	let mut access_controls = Vec::new();
	let mut current_acl = AccessControl::new(
		Regex::new("").unwrap(),
		Access::ReadOnly,
		Vec::new()
	);
	let mut acl_started = false;
	let mut default_access = Access::ReadWrite;

	for line in reader.lines() {
		let line = line.unwrap();
		let line = line.trim();

		if line.is_empty() || line.starts_with('#') {
			continue;
		}

		if line.starts_with('[') {
			if !line.ends_with(']') {
				panic!("Unexpected token at end of line: {}", line);
			}

			if acl_started == true {
				access_controls.push(current_acl);
			}

			let match_value = line.trim_matches(|c| c == '[' || c == ']');
			let match_regex = Regex::new(match_value).unwrap();

			current_acl = AccessControl::new(
				match_regex.clone(),
				Access::ReadOnly,
				Vec::new()
			);

			acl_started = true;
			continue;
		} else if acl_started == false {
			default_access = process_default_var(line);
			continue;
		}

		process_acl_var(&mut current_acl, line);
	}

	access_controls.push(current_acl);

	match default_access {
		Access::ReadWrite => {
			let users: Vec<String> = ["*".to_owned()].into();

			access_controls.push(AccessControl {
				match_regex: Regex::new(".+").unwrap(),
				access: Access::ReadWrite,
				users,
			});
		},
		_ => {}
	};

	access_controls
}

