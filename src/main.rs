use std::env;
use std::process::Command;

mod acl;

use crate::acl::{create_acl_map, Access, AccessControl};

fn deny(path: &str) {
	println!("[POLICY] You do not have access to push to {}", path);
	std::process::exit(1);
}

fn check_directory_perms(
	oldrev: Option<&str>,
	newrev: &str,
	access: &Vec<AccessControl>,
	user: &str,
	acl_file: &str,
) {
	let output = Command::new("git")
		.arg("rev-list")
		.arg(if let Some(oldrev) = oldrev {
			format!("{}..{}", oldrev, newrev)
		} else {
			newrev.to_owned()
		})
		.output()
		.expect("failed to execute git command");

	let stdout = String::from_utf8_lossy(&output.stdout);
	let new_commits: Vec<&str> = stdout.split('\n').collect();

	for rev in new_commits {
		let output = Command::new("git")
			.arg("log")
			.arg("-1")
			.arg("--name-only")
			.arg("--pretty=format:")
			.arg(rev)
			.output()
			.expect("failed to execute git command");

		let stdout = String::from_utf8_lossy(&output.stdout);
		let files_modified = stdout.trim().split('\n').collect::<Vec<&str>>();

		for path in files_modified {
			if path.is_empty() {
				continue;
			}

			let rule = access.iter().find(|a| a.match_regex.is_match(path));

			match rule {
				Some(r) => match r.access {
					Access::ReadOnly => deny(path),
					Access::ReadWrite => {
						if !r.users.contains(&"*".to_string())
							&& !r.users.contains(&user.to_string())
						{
							deny(path);
						}
					}
				},
				None => {
					deny(path);
				}
			}
		}
	}

	let rule = access.iter().find(|a| a.match_regex.is_match(acl_file));

	match rule {
		Some(r) => match r.access {
			Access::ReadWrite => {
				if r.users.contains(&"*".to_string()) {
					println!(
						"[POLICY] WARNING: ACL file is writable by anyone: {}",
						acl_file
					);
				}
			}
			_ => {}
		},
		None => {
			println!(
				"[POLICY] WARNING: ACL file is writable by anyone: {}",
				acl_file
			);
		}
	}
}

pub fn main() {
	let acl_file = env::var("GIT_ACL_HOOK_FILE").unwrap_or("acl".to_owned());

	let args: Vec<String> = env::args().collect();

	let oldrev = if args[2].starts_with("0000000") {
		None
	} else {
		Some(args[2].as_ref())
	};

	let newrev = &args[3];
	let user = &env::var("USER").unwrap();

	let acl_map = create_acl_map(&acl_file);

	check_directory_perms(oldrev, newrev, &acl_map, user, &acl_file);
}
