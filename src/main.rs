use std::env;
use std::process::Command;

mod acl;

use crate::acl::{Access, AccessControl, create_acl_map};

fn check_directory_perms(oldrev: Option<&str>, newrev: &str, access: &Vec<AccessControl>, user: &str) {
	let output = Command::new("git")
		.arg("rev-list")
		.arg(
			if let Some(oldrev) = oldrev {
				format!("{}..{}", oldrev, newrev)
			} else {
				newrev.to_owned()
			}
		)
		.output()
		.expect("failed to execute git command");

	let stdout = String::from_utf8_lossy(&output.stdout);
	let new_commits: Vec<&str> = stdout.split('\n').collect();

	for rev in new_commits {
		let output = Command::new("git")
			.arg("log")
			.arg("-1")
			.arg("--name-only")
			.arg("--pretty=format:''")
			.arg(rev)
			.output()
			.expect("failed to execute git command");

		let stdout = String::from_utf8_lossy(&output.stdout);
		let files_modified = stdout.split('\n').collect::<Vec<&str>>();

		for path in files_modified {
			if path.is_empty() {
				continue;
			}

			let mut has_file_access = false;

			for access_control in access {
				if !access_control.match_regex.is_match(path) {
					continue;
				}

				match access_control.access {
					Access::ReadOnly => {
						if access_control.users.contains(&user.to_string())
						|| access_control.users.contains(&"*".to_string()) {
							// Pass for now
						}
					}
					Access::ReadWrite => {
						if access_control.users.contains(&user.to_string())
						|| access_control.users.contains(&"*".to_string()) {
							has_file_access = true;
						}
					}
				}
			}

			if !has_file_access {
				println!("[POLICY] You do not have access to push to {}", path);
				std::process::exit(1);
			}
		}
	}
}

pub fn main() {
	let args: Vec<String> = env::args().collect();
	let refname = &args[1];
	let oldrev = if args[2].starts_with("0000000") {
		None
	} else {
		Some(args[2].as_ref())
	};
	let newrev = &args[3];
	let user = &env::var("USER").unwrap();
	println!("Enforcing Policies...");
	println!("({}) ({}) ({})", refname, oldrev.map_or("", |x: &str| &x[0..6]), &newrev[0..6]);
	let acl_map = create_acl_map("./acl");
	check_directory_perms(oldrev, newrev, &acl_map, user);
}

