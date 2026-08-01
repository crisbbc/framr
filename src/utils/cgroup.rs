use anyhow::Result;
use std::os::unix::process::CommandExt;
use std::process;

/// KWin on Wayland identifies calling applications by their systemd cgroup,
/// matching `app-<desktop-name>-*.scope` against installed desktop files.
/// When framr is launched from a terminal (Konsole, etc.), it inherits the
/// terminal's cgroup, causing KWin to check the terminal's authorization
/// instead of framr's — resulting in "The process is not authorized to take
/// a screenshot" even after granting permission.
///
/// This function detects the situation and re-executes framr inside its own
/// transient scope via `systemd-run`, so KWin can correctly match it to
/// `framr.desktop` and honour the X-KDE-DBUS-Restricted-Interfaces key.
pub fn ensure_systemd_cgroup() -> Result<()> {
	// Read current cgroup to check if we're already properly scoped
	let cgroup = match std::fs::read_to_string("/proc/self/cgroup") {
		Ok(c) => c,
		Err(_) => return Ok(()),
	};

	// Already in an app-framr-*.scope under app.slice — nothing to do.
	// Checking the full hierarchy avoids trusting a manually-created scope
	// with the right name but the wrong parent slice.
	if cgroup.contains("/app.slice/app-framr-") {
		return Ok(());
	}

	// Not running under a systemd user session (containers, non-systemd
	// distros, etc.) — skip silently; KWin falls back to PID path matching.
	if !cgroup.contains("user.slice") {
		return Ok(());
	} // systemd-run must be available — if we're in a terminal's cgroup
	// but systemd-run is missing, we can't fix the cgroup and KWin will
	// reject screen capture. Return an error so main() can warn the user.
	if process::Command::new("systemd-run")
		.arg("--version")
		.stdout(process::Stdio::null())
		.stderr(process::Stdio::null())
		.status()
		.is_err()
	{
		return Err(anyhow::anyhow!(
			"systemd-run is not available but is required to move out of the \
			 terminal's cgroup for KWin authorization. Install systemd or \
			 launch framr via the application launcher instead of a terminal."
		));
	}

	let exe = std::env::current_exe()?;
	let args: Vec<String> = std::env::args().collect();
	let pid = std::process::id();
	let scope_name = format!("app-framr-{}.scope", pid);

	// Re-execute ourselves inside a transient systemd scope.
	// `exec()` replaces the current process image — it only returns on error.
	let err = process::Command::new("systemd-run")
		.args([
			"--user",
			"--quiet",
			"--scope",
			"--unit",
			&scope_name,
			"--slice=app.slice",
			"--same-dir",
			"--",
		])
		.arg(&exe)
		.args(&args[1..])
		.exec();

	// `exec()` only reaches here on failure
	Err(anyhow::anyhow!(
		"Failed to re-execute framr in its own systemd scope for KWin \
         authorization. Try launching framr via the application launcher \
         instead of a terminal, or run:\n  \
         systemd-run --user --quiet --scope --unit=app-framr-{pid} \
         --slice=app.slice framr\n  \
         Error: {err}",
	))
}
