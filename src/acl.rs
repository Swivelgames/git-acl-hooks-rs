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

	for line in reader.lines() {
		let line = line.unwrap();
		let line = line.trim();

		if line.is_empty() || line.starts_with('#') {
			continue;
		}

		if line.starts_with('[') {
			if acl_started == true {
				access_controls.push(current_acl);
			}

			acl_started = true;

			current_acl = AccessControl::new(
				Regex::new("").unwrap(),
				Access::ReadOnly,
				Vec::new()
			);
		} else if acl_started == false {
			continue;
		}

		let mut parts = line.splitn(2, '=');
		let key = parts.next().unwrap().trim();
		let value = parts.next().unwrap().trim();

		let read_len = value.find('#').unwrap_or(value.len());
		let value = value[..read_len].trim();

		match key {
			"match" => {
				let match_regex = Regex::new(value).unwrap();
				current_acl.match_regex = match_regex.clone();
			}
			"access" => {
				let access = match value {
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

	access_controls
}

