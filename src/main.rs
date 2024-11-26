
use std::path::{Path,PathBuf};

use duct::cmd;

/// Path to tool-chain installation directory
const YAUL_INSTALL_ROOT: &'static str = "/home/seth/.local/x-tools/sh2eb-elf";

/// SH-2 tool-chain prefix
const YAUL_ARCH_SH_PREFIX: &'static str = "sh2eb-elf";

/// SH-2 tool-chain program prefix
const YAUL_PROG_SH_PREFIX: &'static str = YAUL_ARCH_SH_PREFIX;

/// M68k tool-chain prefix
//const YAUL_ARCH_M68K_PREFIX: &'static str = "m68keb-elf";

/// Path to where the build is to be located
//const YAUL_BUILD_ROOT: &'static str = "/home/seth/libyaul";

/// Name of build directory
//const YAUL_BUILD: &'static str = "build";

/// Compilation verbosity
/// Values:
///   true  -> Display build step line only
///   false -> Verbose
static mut SILENT: bool = true;

/// Enable DEBUG on a release build
/// Values:
///   true  -> Enable DEBUG
///   false -> Disable DEBUG
//static mut DEBUG_RELEASE: bool = true;

fn main() -> std::io::Result<()> {
	assert!(!YAUL_INSTALL_ROOT.trim().is_empty(), "Undefined YAUL_INSTALL_ROOT (install root directory)");
	assert_eq!(1, YAUL_INSTALL_ROOT.trim().split(' ').count(), "YAUL_INSTALL_ROOT (install root directory) contains spaces");

	assert!(!YAUL_ARCH_SH_PREFIX.trim().is_empty(), "Undefined YAUL_ARCH_SH_PREFIX (tool-chain prefix)");
	assert_eq!(1, YAUL_ARCH_SH_PREFIX.trim().split(' ').count(), "YAUL_ARCH_SH_PREFIX (tool-chain prefix) contains spaces");

	assert_eq!(1, YAUL_PROG_SH_PREFIX.trim().split(' ').count(), "YAUL_PROG_SH_PREFIX (tool-chain program prefix) contains spaces");

	fn word_split(s: &str, n: usize) -> Option<&str> {
		s.split(';').nth(n)
	}

	let yaul_cflags_shared = format!("-I{YAUL_INSTALL_ROOT}/{YAUL_ARCH_SH_PREFIX}/include/yaul");

	let yaul_cflags = yaul_cflags_shared.clone();
	let yaul_cxxflags = yaul_cflags_shared.clone();

	let mut args = std::env::args();
	args.next(); // remove the executable name

	let command = args.next().expect("expected command 'build' or 'clean'");
	if command == "clean" {
		cmd!("rm", "-rf", "audio-tracks", "build", "cd").run()?;
		cmd!("rm", "*.cue").run()?;
		cmd!("rm", "*.iso").run()?;
		return Ok(());
	}

	if command != "build" {
		panic!("expected command: 'build' or 'clean'");
	}

	// Customizable
	let sh_program = args.next().expect("no program name");
	let mut sh_defsyms: Vec<String> = vec![];
	let mut sh_srcs = args.map(PathBuf::from).collect::<Vec<PathBuf>>();
	let sh_build_dir = "build";
	let sh_output_dir = ".";

	println!("building {sh_program}");
	println!("  defined symbols [{}]", sh_defsyms.join(","));
	println!("  sources [{}]", sh_srcs.iter()
		.map(|s| s.display().to_string())
		.collect::<Vec<String>>()
		.join(","));
	println!();

	// Customizable variables
	let image_directory        = "cd";           // ISO/CUE
	let audio_tracks_directory = "audio-tracks"; // ISO/CUE
	let image_1st_read_bin     = "A.BIN";        // ISO/CUE
	let ip_version             = "V1.000";       // ISO/CUE, SS
	let ip_release_date        = "20241030";     // ISO/CUE, SS
	let ip_areas               = "JTUBKAEL";     // ISO/CUE, SS
	let ip_peripherals         = "JAMKST";       // ISO/CUE, SS
	let ip_title               = "Test";         // ISO/CUE, SS
	let ip_main_stack_addr     = "0x06004000";   // ISO/CUE, SS
	let ip_service_stack_addr  = "0x06001E00";   // ISO/CUE, SS
	let ip_1st_read_addr       = "0x06004000";   // ISO/CUE, SS
	let ip_1st_read_size       = "0";            // ISO/CUE, SS

	println!("iso directory='{image_directory}'");
	println!("audio tracks directory='{audio_tracks_directory}'");
	println!("iso 1st read binary ='{image_1st_read_bin}'");
	println!("ip version='{ip_version}'");
	println!("ip release date='{ip_release_date}'");
	println!("ip areas='{ip_areas}'");
	println!("ip peripherals='{ip_peripherals}'");
	println!("ip title='{ip_title}'");
	println!("ip main stack address='{ip_main_stack_addr}'");
	println!("ip sub stack address='{ip_service_stack_addr}'");
	println!("ip 1st read address='{ip_1st_read_addr}'");
	println!("ip 1st read size='{ip_1st_read_size}'");
	println!();

	let sh_build_path = std::path::absolute(sh_build_dir)
		.expect(&format!("unable to find path to '{sh_build_dir}'"));

	let sh_output_path = std::path::absolute(sh_output_dir)
		.expect(&format!("unable to find path to '{sh_output_dir}'"));

	fn convert_build_path<P: AsRef<Path> + Copy>(build_path: P, s: P) -> Result<PathBuf, String> {
		let s = std::path::absolute(s)
			.map_err(|_| format!("unable to find path to '{}'", s.as_ref().display()))?
			.to_str()
			.ok_or(format!("unable to convert path '{}' to a string", s.as_ref().display()))?
			.replace('/', "@");

		Ok([ build_path.as_ref(), s.as_ref() ].iter().collect())
	}

	let yaul_prefix = format!("{YAUL_INSTALL_ROOT}/bin/{YAUL_PROG_SH_PREFIX}");

	let sh_cc      = format!("{yaul_prefix}-gcc");
	let sh_cxx     = format!("{yaul_prefix}-g++");
	let sh_ld      = format!("{yaul_prefix}-gcc");
	let sh_nm      = format!("{yaul_prefix}-gcc-nm");
	let sh_objcopy = format!("{yaul_prefix}-objcopy");
	let sh_objdump = format!("{yaul_prefix}-objdump");

	let sh_cflags_shared = vec![
		"-W".to_string(),
		"-Wall".to_string(),
		"-Wduplicated-branches".to_string(),
		"-Wduplicated-cond".to_string(),
		"-Wextra".to_string(),
		"-Winit-self".to_string(),
		"-Wmissing-include-dirs".to_string(),
		"-Wno-format".to_string(),
		"-Wno-main".to_string(),
		"-Wnull-dereference".to_string(),
		"-Wshadow".to_string(),
		"-Wstrict-aliasing".to_string(),
		"-Wunused".to_string(),
		"-Wunused-parameter".to_string(),
		"-save-temps=obj".to_string(),
	];

	let mut sh_ldflags = vec![
		"-static".to_string(),
		"-Wl,--gc-sections".to_string(),
		format!("-Wl,-Map,{}/{sh_program}.map", sh_build_path.display()),
	];

	let sh_cflags: Vec<String> = vec![
		"-std=c11",
		"-Wbad-function-cast",
	].into_iter()
		.map(|s| s.to_string())
		.chain(sh_cflags_shared.iter().cloned())
		.chain(std::iter::once(yaul_cflags))
		.collect();

	let sh_cxxflags: Vec<String> = vec![
		"-std=c++17",
		"-fno-exceptions",
		"-fno-rtti",
		"-fno-unwind-tables",
		"-fno-asynchronous-unwind-tables",
		"-fno-threadsafe-statics",
		"-fno-use-cxa-atexit",
	].into_iter()
		.map(|s| s.to_string())
		.chain(sh_cflags_shared.iter().cloned())
		.chain(std::iter::once(yaul_cxxflags))
		.collect();

	let builtin_assets = vec![];

	assert!(!ip_version.is_empty(), "Undefined IP_VERSION");
	assert!(!ip_release_date.is_empty(), "Undefined IP_RELEASE_DATE");
	assert!(!ip_areas.is_empty(), "Undefined IP_AREAS");
	assert!(!ip_peripherals.is_empty(), "Undefined IP_PERIPHERALS");
	assert!(!ip_title.is_empty(), "Undefined IP_TITLE");
	assert!(!ip_main_stack_addr.is_empty(), "Undefined IP_MAIN_STACK_ADDR");
	assert!(!ip_service_stack_addr.is_empty(), "Undefined IP_SERVICE_STACK_ADDR");
	assert!(!ip_1st_read_addr.is_empty(), "Undefined IP_1ST_READ_ADDR");
	assert!(!ip_1st_read_size.is_empty(), "Undefined IP_1ST_READ_SIZE");

	sh_defsyms.extend_from_slice(&[
		format!("-Wl,--defsym=___master_stack={ip_main_stack_addr}"),
		format!("-Wl,--defsym=___slave_stack={ip_service_stack_addr}"),
	]);

	assert!(!sh_build_dir.is_empty(), "Empty SH_BUILD_DIR (SH build directory)");
	assert!(!sh_output_dir.is_empty(), "Empty SH_OUTPUT_DIR (SH output directory)");
	assert!(!sh_program.is_empty(), "Empty SH_PROGRAM (SH program name)");

	std::fs::create_dir_all(sh_build_dir)?;
	std::fs::create_dir_all(sh_output_dir)?;

	let mut builtin_asset_rule = |asset_file: &str, asset_name: &str| -> Result<(), String> {
		println!("builtin_asset_rule({asset_file},{asset_name})");

		let asset_path = Path::new(asset_file).with_extension("o");
		let target = convert_build_path(&sh_build_path, &asset_path)?;

		cmd!(format!("{YAUL_INSTALL_ROOT}/bin/bin2o"),
			asset_file,
			asset_name,
			target.display().to_string(),
		).stderr_capture().run().expect("unable to convert bin to object file");

		sh_srcs.push(asset_path);
		Ok(())
	};

	println!("builtin assets");
	for builtin_asset in builtin_assets {
		use std::io::{Error, ErrorKind};

		if let Err(e) = builtin_asset_rule(
			word_split(builtin_asset, 1)
				.ok_or(Error::new(ErrorKind::Other, "no path to builtin asset"))?,
			word_split(builtin_asset, 2)
				.ok_or(Error::new(ErrorKind::Other, "invalid builtin asset name"))?,
		) {
			eprintln!("{e}");
		}
	}

	// Check that SH_SRCS don't include duplicates. Be mindful that sort remove
	// duplicates.
	let sh_srcs_uniq = {
		let mut temp = sh_srcs.clone();
		temp.sort_unstable();
		temp.dedup();
		temp
	};

	let sh_srcs_c: Vec<PathBuf> = sh_srcs_uniq.iter()
		.filter(|file| file.extension().filter(|&x| x == "c").is_some())
		.cloned()
		.collect();
	let sh_srcs_cxx: Vec<PathBuf> = sh_srcs_uniq.iter()
		.filter(|file| file.extension().filter(|&x| x == "cxx" || x == "cpp" || x == "cc" || x == "C").is_some())
		.cloned()
		.collect();
	let sh_srcs_s: Vec<PathBuf> = sh_srcs_uniq.iter()
		.filter(|file| file.extension().filter(|&x| x == "sx").is_some())
		.cloned()
		.collect();

	println!("generating unique SH objects list");
	let mut sh_objs_uniq = Vec::<PathBuf>::new();
	for file in sh_srcs_uniq.iter() {
		match convert_build_path(&sh_build_path, &file) {
			Ok(path) => {
				println!("  {}", path.with_extension("o").display());
				sh_objs_uniq.push(path.with_extension("o"));
			}
			Err(e) => eprintln!("{e}"),
		}
	}

	sh_ldflags.extend(sh_defsyms);

	let sh_specs = vec!["yaul.specs", "yaul-main.specs"];

	// If there are any C++ files, add the specific C++ specs file. This is done
	// to avoid adding (small) bloat to any C-only projects.
	let sh_cxx_specs = if !sh_srcs_cxx.is_empty() {
		vec!["yaul-main-c++.specs"]
	} else {
		vec![]
	};

	// Parse out included paths from GCC when the specs files are used. This is used
	// to explicitly populate each command database entry with include paths
	let sh_system_include_dirs: Vec<String> = cmd!("echo")
		.pipe(cmd!(format!("{sh_cc}"), "-E", "-Wp,-v", "-").stderr_to_stdout())
		.pipe(cmd!("awk", r#"/^\s/ { sub(/^\s+/,""); gsub(/\\/,"/"); print }"#))
		.read()
		.expect("failed to execute piped commands")
		.split('\n')
		.map(str::to_owned)
		.collect();
	println!("SH system include directories [\n{}\n]", sh_system_include_dirs.join("\n"));

	fn get_mod_date<P: AsRef<Path>>(a: P) -> std::time::SystemTime {
		std::fs::metadata(a.as_ref())
			.and_then(|data| data.modified())
			.unwrap_or(std::time::SystemTime::UNIX_EPOCH)
	}

	let specs: Vec<String> = sh_specs.iter()
		.map(|spec| format!("-specs={spec}"))
		.collect();
	let cpp_specs: Vec<String> = sh_cxx_specs.iter()
		.map(|spec| format!("-specs={spec}"))
		.collect();

	println!();

	let wrap_error = format!("{YAUL_INSTALL_ROOT}/share/wrap-error");

	let yaul_ip_sx = format!("{YAUL_INSTALL_ROOT}/share/yaul/ip/ip.sx");
	let build_program_bin = PathBuf::from(format!("{}/{sh_program}.bin", sh_build_path.display()));
	let build_ip_bin = format!("{}/IP.BIN", sh_build_path.display());
	let out_program_iso = format!("{}/{sh_program}.iso", sh_output_path.display());

	println!("generating SH C build objects");
	for src in sh_srcs_c.iter() {
		match convert_build_path(&sh_build_path, &src.with_extension("o")) {
			Err(e) => eprintln!("{e}"),
			Ok(target) => if get_mod_date(src) > get_mod_date(&target) {
				println!("  {}", src.with_extension("o").display());

				cmd(format!("{sh_cc}"), [
					"-MT".into(), target.display().to_string(),
					"-MF".into(), target.with_extension("d").display().to_string(),
					"-MD".into(),
				].into_iter()
					.chain(sh_cflags.clone())
					.chain(specs.clone())
					.chain([
						"-c".into(),
						"-o".into(),
						target.display().to_string(),
						src.display().to_string(),
					])
				).run().expect(&format!("failed to compile {}", target.display()));
			}
		}

/*
		println!("compiling {}", src.display());
		cmd(format!("/usr/bin/gcc"), [
			format!("{}", src.display()),
			"-D__INTELLISENSE__".into(),
			"-m32".into(),
			"-nostdinc".into(),
			"-Wno-gnu-statement-expression".into(),
		].into_iter()
			.chain(sh_cflags.clone())
			.chain(sh_system_include_dirs
				.iter()
				.map(std::path::absolute)
				.filter_map(|m| m.ok())
				.map(|path| format!("-isystem{}", path.display())))
			.chain([
				format!("--include={YAUL_INSTALL_ROOT}/{YAUL_PROG_SH_PREFIX}/include/intellisense.h"),
				"-c".into(),
				src.display().to_string(),
			]),
		).stdout_to_stderr().run().expect(&format!("unable to compile {}", src.display()));
*/
	}
	println!();

	println!("generating SH C++ build objects");
	for src in sh_srcs_cxx.iter() {
		match convert_build_path(&sh_build_path, &src.with_extension("o")) {
			Err(e) => eprintln!("{e}"),
			Ok(target) => if get_mod_date(src) > get_mod_date(&target) {
				println!("  {}", src.with_extension("o").display());

				cmd(format!("{sh_cxx}"), [
					"-MT".into(), target.display().to_string(),
					"-MF".into(), target.with_extension("d").display().to_string(),
					"-MD".into(),
				].into_iter()
					.chain(sh_cxxflags.clone())
					.chain(specs.iter().cloned())
					.chain(cpp_specs.clone())
					.chain([
						"-c".into(),
						"-o".into(),
						target.display().to_string(),
						src.display().to_string(),
					])
				).run().expect(&format!("failed to compile {}", target.display()));
			}
		}

/*
		println!("compiling {}", src.display());
		cmd(format!("/usr/bin/g++"), [
			format!("{}", src.display()),
			"-D__INTELLISENSE__".into(),
			"-m32".into(),
			"-nostdinc++".into(),
			"-nostdlibinc".into(),
			"-Wno-gnu-statement-expression".into(),
		].into_iter()
			.chain(sh_cxxflags.clone())
			.chain(sh_system_include_dirs
				.iter()
				.map(std::path::absolute)
				.filter_map(|m| m.ok())
				.map(|path| format!("-isystem{}", path.display())))
			.chain([
				format!("--include={YAUL_INSTALL_ROOT}/{YAUL_PROG_SH_PREFIX}/include/intellisense.h"),
				"-c".into(),
				src.display().to_string(),
			]),
		).stdout_to_stderr().run().expect(&format!("unable to compile {}", src.display()));
*/
	}
	println!();

	println!("generating SH asm build objects");
	for src in sh_srcs_s.iter() {
		match convert_build_path(&sh_build_path, &src.with_extension("o")) {
			Err(e) => eprintln!("{e}"),
			Ok(target) => if get_mod_date(src) > get_mod_date(&target) {
				println!("  {}", src.with_extension("o").display());

				cmd(format!("{sh_cc}"),
					sh_cflags.clone()
						.into_iter()
						.chain([
							"-c".into(),
							"-o".into(),
							target.display().to_string(),
							src.display().to_string(),
						])
				).run().expect(&format!("failed to compile {}", target.display()));
			}
		}
	}
	println!();

	let build_program_elf = build_program_bin.with_extension("elf");

	let newest_obj = sh_objs_uniq.iter()
		.flat_map(|obj| std::fs::metadata(obj).ok())
		.flat_map(|obj_md| obj_md.modified().ok())
		.reduce(|a,b| if a > b { a } else { b });
	println!("attempting elf build: newest_obj({:?}) > {sh_program}.elf({:?})",
		newest_obj, get_mod_date(&build_program_elf));
	if let Some(obj) = newest_obj {
		if obj > get_mod_date(&build_program_elf) {
			println!("building {}", build_program_elf.display());

			cmd(format!("{sh_ld}"),
				specs.clone()
					.into_iter()
					.chain(cpp_specs.clone())
					.chain(sh_objs_uniq.iter().map(|obj| format!("{}", obj.display())))
					.chain(sh_ldflags)
					.chain([
						"-o".into(),
						format!("{}", build_program_elf.display()),
					])
			).run().expect("failed to execute link");

			cmd!(format!("{sh_nm}"), format!("{}", build_program_elf.display()))
				.stdout_path(build_program_elf.with_extension("sym"))
				.run()
				.expect("failed to execute symbol dump");

			cmd!(format!("{sh_objdump}"), "-S", format!("{}", build_program_elf.display()))
				.stdout_path(build_program_elf.with_extension("asm"))
				.run()
				.expect("failed to execute assembly dump");
		}
	}
	println!();

	println!("attempting bin build: {sh_program}.elf({:?}) > {sh_program}.bin({:?})",
		get_mod_date(&build_program_elf), get_mod_date(&build_program_bin));
	if get_mod_date(&build_program_elf) > get_mod_date(&build_program_bin) {
		println!("building {}", build_program_bin.display());

		cmd!(format!("{sh_objcopy}"),
			"-O", "binary",
			build_program_elf.display().to_string(),
			build_program_bin.display().to_string(),
		).run()?;

		if unsafe { !SILENT } {
			eprintln!("E");

			cmd!("du", "-hs", build_program_bin.display().to_string())
				.pipe(cmd!("awk", r#"'{ print $1; }'"#))
				.run()?;
		}
	}
	println!();

	println!("attempting IP.BIN build: ip.sx({:?}) > ip.bin({:?}) || {sh_program}.bin({:?}) > ip.bin({:?})",
		get_mod_date(&yaul_ip_sx), get_mod_date(&build_ip_bin), get_mod_date(&build_program_bin), get_mod_date(&build_ip_bin));
	if get_mod_date(yaul_ip_sx) > get_mod_date(&build_ip_bin)
	|| get_mod_date(&build_program_bin) > get_mod_date(&build_ip_bin)
	{
		println!("building IP.BIN");

		cmd!(&wrap_error, format!("{YAUL_INSTALL_ROOT}/bin/make-ip"),
			build_program_bin.display().to_string(),
			ip_version,
			ip_release_date,
			ip_areas,
			ip_peripherals,
			ip_title,
			ip_main_stack_addr,
			ip_service_stack_addr,
			ip_1st_read_addr,
			ip_1st_read_size,
		).run()?;
	}
	println!();

	println!("attempting iso build: ip.bin({:?}) > {sh_program}.iso({:?}) || {sh_program}.bin({:?}) > {sh_program}.iso({:?})",
		get_mod_date(&build_ip_bin), get_mod_date(&out_program_iso), get_mod_date(&build_program_bin), get_mod_date(&out_program_iso));
	let mut is_iso_built = false;
	if get_mod_date(&build_ip_bin) > get_mod_date(&out_program_iso)
	|| get_mod_date(&build_program_bin) > get_mod_date(&out_program_iso)
	{
		println!("building {sh_program}.iso");

		cmd!("mkdir", "-p", image_directory).run()?;

		cmd!("cp",
			build_program_bin.display().to_string(),
			format!("{image_directory}/{image_1st_read_bin}"),
		).run()?;

		for txt in ["ABS.TXT", "BIB.TXT", "CPY.TXT"] {
			match std::fs::exists(&format!("{image_directory}/{txt}")) {
				Ok(false) => {
					cmd!("printf", "--", "empty").stdout_path(format!("{image_directory}/{txt}")).run()?;
				}
				Ok(true) => {}
				Err(e) => eprintln!("{e}"),
			}
		}

		cmd!(&wrap_error, format!("{YAUL_INSTALL_ROOT}/bin/make-iso"),
			image_directory,
			build_ip_bin,
			sh_output_path.display().to_string(),
			sh_program.clone(),
		).run()?;

		is_iso_built = true;
	}
	println!();

	println!("attempting cue build");
	let cue_file = [ sh_output_path.clone(), sh_program.clone().into() ].iter().collect::<PathBuf>().with_extension("cue");
	if !std::fs::exists(&cue_file)? || is_iso_built {
		println!("building {}", cue_file.display());

		cmd!("mkdir", "-p", format!("{audio_tracks_directory}")).run()?;

		cmd!(&wrap_error, format!("{YAUL_INSTALL_ROOT}/bin/make-cue"),
			audio_tracks_directory,
			out_program_iso,
		).run()?;
	}
	
	Ok(())
}
