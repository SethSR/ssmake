
use std::path::{Path,PathBuf};

use duct::cmd;

use toml::{Table, Value};
use tracing::{trace, debug, warn, error};

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

/// Enable DEBUG on a release build
/// Values:
///   true  -> Enable DEBUG
///   false -> Disable DEBUG
//static mut DEBUG_RELEASE: bool = true;

fn main() -> std::io::Result<()> {
	tracing_subscriber::fmt().init();

	assert!(!YAUL_INSTALL_ROOT.trim().is_empty(), "Undefined YAUL_INSTALL_ROOT (install root directory)");
	assert_eq!(1, YAUL_INSTALL_ROOT.trim().split(' ').count(), "YAUL_INSTALL_ROOT (install root directory) contains spaces");

	assert!(!YAUL_ARCH_SH_PREFIX.trim().is_empty(), "Undefined YAUL_ARCH_SH_PREFIX (tool-chain prefix)");
	assert_eq!(1, YAUL_ARCH_SH_PREFIX.trim().split(' ').count(), "YAUL_ARCH_SH_PREFIX (tool-chain prefix) contains spaces");

	assert_eq!(1, YAUL_PROG_SH_PREFIX.trim().split(' ').count(), "YAUL_PROG_SH_PREFIX (tool-chain program prefix) contains spaces");

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
		error!("expected command: 'build' or 'clean'");
		panic!();
	}

	// TODO - srenshaw - Add better error handling for missing configuration files.
	let config = std::fs::read_to_string("config.toml")
		.expect("missing 'config.toml'")
		.parse::<Table>()
		.unwrap_or_default();

	fn missing_config_string<S: AsRef<str>>(property: &str, value: S) -> S {
		warn!("missing {property} = \"value\" (string)");
		value
	}

	fn missing_config_str_array<T>(property: &str) -> Vec<T> {
		warn!("missing {property} = [] (string array)");
		vec![]
	}

	fn missing_config_path<'a>(property: &str, value: &'a str) -> &'a str {
		warn!("missing {property} = \"path\" (string)]");
		value
	}

	fn missing_config_integer(property: &str, value: u32) -> u32 {
		warn!("missing {property} = \"value\" (u32)");
		value
	}

	// Project Directory Configuration
	let dir_image  = PathBuf::from(config["dirs"]["image"].as_str()  // ISO/CUE
		.unwrap_or_else(|| missing_config_path("dirs.image", "cd")));
	let dir_audio  = PathBuf::from(config["dirs"]["audio"].as_str()  // ISO/CUE
		.unwrap_or_else(|| missing_config_path("dirs.audio", "audio")));
	let dir_build  = PathBuf::from(config["dirs"]["build"].as_str()  // ISO/CUE
		.unwrap_or_else(|| missing_config_path("dirs.build", "build")));
	let dir_asset  = PathBuf::from(config["dirs"]["assets"].as_str() // ISO/CUE
		.unwrap_or_else(|| missing_config_path("dirs.assets", "assets")));
	let dir_output = PathBuf::from(config["dirs"]["output"].as_str() // ISO/CUE
		.unwrap_or_else(|| missing_config_path("dirs.output", ".")));

	// TODO - srenshaw - Do we need this to be configurable?
	let image_1st_read_bin    = "A.BIN"; // ISO/CUE

	trace!("project config");
	trace!("  image  = '{}'", dir_asset.display());
	trace!("  build  = '{}'", dir_build.display());
	trace!("  asset  = '{}'", dir_asset.display());
	trace!("  build  = '{}'", dir_build.display());
	trace!("  output = '{}'", dir_output.display());

	// SH2 Program Configuration
	let sh_program = config["sh"]["program"].as_str()
		.expect("missing sh.program = \"name\" (string)");
	let sh_flags: Vec<String> = config["sh"]["flags"].as_array()
		.cloned()
		.unwrap_or_else(|| missing_config_str_array("sh.flags"))
		.into_iter()
		.flat_map(|v| v.as_str().map(str::to_owned))
		.collect();
	let mut sh_symbols: Vec<String> = config["sh"].get("symbols")
		.map(|v| v.as_array())
		.flatten()
		.cloned()
		.unwrap_or_else(|| missing_config_str_array("sh.symbols"))
		.into_iter()
		.flat_map(|v| v.as_str().map(str::to_owned))
		.collect();
	let mut sh_srcs: Vec<PathBuf> = config["sh"]["srcs"].as_array()
		.expect("missing sh.srcs = [] (string array)")
		.into_iter()
		.flat_map(Value::as_str)
		.map(PathBuf::from)
		.collect();

	trace!("SH2 program config");
	trace!("  program = '{sh_program}'");
	trace!("  flags   = [{}]", sh_flags.join(","));
	trace!("  symbols = [{}]", sh_symbols.join(","));
	trace!("  sources = [{}]", sh_srcs.iter()
		.map(|s| s.display().to_string())
		.collect::<Vec<String>>()
		.join(","));

	// IP Configuration
	let ip_version         = config["ip"]["version"].as_str()             // ISO/CUE, SS
		.unwrap_or_else(|| missing_config_string("ip.version", "V1.000"));
	let ip_release_date    = config["ip"]["release-date"].as_str()        // ISO/CUE, SS
		.map(str::to_owned)
		.unwrap_or_else(|| {
			let date = chrono::Utc::now().format("%Y%m%d").to_string();
			missing_config_string("ip.release-date", date)
		});
	let ip_areas           = config["ip"]["areas"].as_str()               // ISO/CUE, SS
		.unwrap_or_else(|| missing_config_string("ip.areas", "JTUBKAEL"));
	let ip_peripherals     = config["ip"]["peripherals"].as_str()         // ISO/CUE, SS
		.unwrap_or_else(|| missing_config_string("ip.peripherals", "JAMKST"));
	let ip_title           = config["ip"]["title"].as_str()               // ISO/CUE, SS
		.unwrap_or_else(|| missing_config_string("ip.title", sh_program));
	let ip_main_stack_addr = config["ip"]["main-stack-addr"].as_integer() // ISO/CUE, SS
		.map(|v| v as u32)
		.unwrap_or_else(|| missing_config_integer("ip.main-stack-addr", 0x06004000));
	let ip_sub_stack_addr  = config["ip"]["sub-stack-addr"].as_integer()  // ISO/CUE, SS
		.map(|v| v as u32)
		.unwrap_or_else(|| missing_config_integer("ip.sub-stack-addr", 0x06001E00));
	let ip_1st_read_addr   = config["ip"]["1st-read-addr"].as_integer()   // ISO/CUE, SS
		.map(|v| v as u32)
		.unwrap_or_else(|| missing_config_integer("ip.1st-read-addr", 0x06004000));
	let ip_1st_read_size   = config["ip"]["1st-read-size"].as_integer()   // ISO/CUE, SS
		.map(|v| v as u32)
		.unwrap_or_else(|| missing_config_integer("ip.1st-read-size", 0));

	trace!("IP config");
	trace!("  version            = '{ip_version}'");
	trace!("  release-date       = '{ip_release_date}'");
	trace!("  areas              = '{ip_areas}'");
	trace!("  peripherals        = '{ip_peripherals}'");
	trace!("  title              = '{ip_title}'");
	trace!("  main-stack-address = '{ip_main_stack_addr}'");
	trace!("  sub-stack-address  = '{ip_sub_stack_addr}'");
	trace!("  1st-read-address   = '{ip_1st_read_addr}'");
	trace!("  1st-read-size      = '{ip_1st_read_size}'");

	sh_symbols.extend([
		format!("-Wl,--defsym=___master_stack=0x{ip_main_stack_addr:x}"),
		format!("-Wl,--defsym=___slave_stack=0x{ip_sub_stack_addr:x}"),
	]);

	let assets: Vec<(String, String)> = config["assets"].as_array()
		.cloned()
		.unwrap_or_default()
		.into_iter()
		.flat_map(|v| v["file"].as_str().map(str::to_owned).zip(v["name"].as_str().map(str::to_owned)))
		.collect();

	let sh_build_path = std::path::absolute(&dir_build)
		.expect(&format!("unable to find path to '{}'", dir_build.display()));

	let sh_output_path = std::path::absolute(&dir_output)
		.expect(&format!("unable to find path to '{}'", dir_output.display()));

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

	std::fs::create_dir_all(dir_build)?;
	std::fs::create_dir_all(dir_output)?;

	trace!("builtin assets");
	for (file, name) in assets {
		trace!("  {}/{file} -> {name}", dir_asset.display());

		let asset_path = PathBuf::from(file.clone() + ".o");
		let target = match convert_build_path(&sh_build_path, &asset_path) {
			Ok(path) => path,
			Err(e) => {
				error!("{e}");
				continue;
			}
		};

		cmd!(format!("{YAUL_INSTALL_ROOT}/bin/bin2o"),
			format!("{}/{file}", dir_asset.display()),
			name,
			target.display().to_string(),
		).stderr_capture().run().expect("unable to convert bin to object file");

		sh_srcs.push(asset_path);
	}

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

	trace!("generating unique SH objects list");
	let mut sh_objs_uniq = Vec::<PathBuf>::new();
	for file in sh_srcs_uniq.iter() {
		match convert_build_path(&sh_build_path, &file) {
			Ok(path) => {
				trace!("  {}", path.with_extension("o").display());
				sh_objs_uniq.push(path.with_extension("o"));
			}
			Err(e) => error!("{e}"),
		}
	}

	sh_ldflags.extend(sh_symbols);

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
	trace!("SH system include directories");
	for dir in sh_system_include_dirs {
		trace!("  {dir}");
	}

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

	let wrap_error = format!("{YAUL_INSTALL_ROOT}/share/wrap-error");

	let yaul_ip_sx = format!("{YAUL_INSTALL_ROOT}/share/yaul/ip/ip.sx");
	let build_program_bin = PathBuf::from(format!("{}/{sh_program}.bin", sh_build_path.display()));
	let build_ip_bin = format!("{}/IP.BIN", sh_build_path.display());
	let out_program_iso = format!("{}/{sh_program}.iso", sh_output_path.display());

	let build_c_options = |src: &Path, target: &Path| [
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
		]);
	let build_cxx_options = |src: &Path, target: &Path| [
		"-MT".into(), target.display().to_string(),
		"-MF".into(), target.with_extension("d").display().to_string(),
		"-MD".into(),
	].into_iter()
		.chain(sh_cxxflags.clone())
		.chain(specs.clone())
		.chain([
			"-c".into(),
			"-o".into(),
			target.display().to_string(),
			src.display().to_string(),
		]);

	trace!("generating SH C build objects");
	debug!("    '{}'", build_c_options(Path::new("source.c"), Path::new("target.o")).collect::<Vec<String>>().join(" "));
	for src in sh_srcs_c.iter() {
		print!("  {}", src.display());
		match convert_build_path(&sh_build_path, &src.with_extension("o")) {
			Err(e) => error!("{e}"),
			Ok(target) => if get_mod_date(src) > get_mod_date(&target) {
				trace!(" -> {}", src.with_extension("o").display());

				let mut args = vec![format!("{sh_cc}")];
				args.extend(build_c_options(src, &target));
				cmd(&wrap_error, args)
					.run()
					.expect(&format!("failed to compile {}", target.display()));
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

	trace!("generating SH C++ build objects");
	debug!("    '{}'", build_cxx_options(Path::new("source.c"), Path::new("target.o")).collect::<Vec<String>>().join(" "));
	for src in sh_srcs_cxx.iter() {
		match convert_build_path(&sh_build_path, &src.with_extension("o")) {
			Err(e) => error!("{e}"),
			Ok(target) => if get_mod_date(src) > get_mod_date(&target) {
				trace!("  {} -> {}", src.display(), src.with_extension("o").display());

				cmd(format!("{sh_cxx}"), build_cxx_options(src, &target))
					.run()
					.expect(&format!("failed to compile {}", target.display()));
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

	let build_asm_options = |src: &Path, target: &Path| sh_cflags.clone()
		.into_iter()
		.chain([
			"-c".into(),
			"-o".into(),
			target.display().to_string(),
			src.display().to_string(),
		]);

	trace!("generating SH asm build objects");
	debug!("    '{}'", build_asm_options(Path::new("source.asm"), Path::new("target.o")).collect::<Vec<String>>().join(" "));
	for src in sh_srcs_s.iter() {
		match convert_build_path(&sh_build_path, &src.with_extension("o")) {
			Err(e) => error!("{e}"),
			Ok(target) => if get_mod_date(src) > get_mod_date(&target) {
				trace!("  {} -> {}", src.display(), src.with_extension("o").display());

				cmd(format!("{sh_cc}"), build_asm_options(src, &target))
					.run()
					.expect(&format!("failed to compile {}", target.display()));
			}
		}
	}

	let build_program_elf = build_program_bin.with_extension("elf");

	let build_elf_options = specs.clone()
		.into_iter()
		.chain(cpp_specs.clone())
		.chain(sh_objs_uniq.iter().map(|obj| format!("{}", obj.display())))
		.chain(sh_ldflags)
		.chain([
			"-o".into(),
			build_program_elf.display().to_string(),
		]);

	let newest_obj = sh_objs_uniq.iter()
		.flat_map(|obj| std::fs::metadata(obj).ok())
		.flat_map(|obj_md| obj_md.modified().ok())
		.reduce(|a,b| if a > b { a } else { b });
	trace!("attempting elf build: newest_obj({:?}) > {sh_program}.elf({:?})",
		newest_obj, get_mod_date(&build_program_elf));
	if let Some(obj) = newest_obj {
		if obj > get_mod_date(&build_program_elf) {
			trace!("building {}", build_program_elf.display());
			debug!("  '{}'", build_elf_options.clone().collect::<Vec<String>>().join(" "));

			cmd(format!("{sh_ld}"), build_elf_options)
				.run()
				.expect("failed to execute link");

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

	trace!("attempting bin build: {sh_program}.elf({:?}) > {sh_program}.bin({:?})",
		get_mod_date(&build_program_elf), get_mod_date(&build_program_bin));
	if get_mod_date(&build_program_elf) > get_mod_date(&build_program_bin) {
		trace!("building {}", build_program_bin.display());

		cmd!(format!("{sh_objcopy}"),
			"-O", "binary",
			build_program_elf.display().to_string(),
			build_program_bin.display().to_string(),
		).run()?;

		cmd!("du", "-hs", build_program_bin.display().to_string())
			.pipe(cmd!("awk", r#"{ print $1; }"#))
			.run()?;
	}

	trace!("attempting IP.BIN build: ip.sx({:?}) > ip.bin({:?}) || {sh_program}.bin({:?}) > ip.bin({:?})",
		get_mod_date(&yaul_ip_sx), get_mod_date(&build_ip_bin), get_mod_date(&build_program_bin), get_mod_date(&build_ip_bin));
	if get_mod_date(yaul_ip_sx) > get_mod_date(&build_ip_bin)
	|| get_mod_date(&build_program_bin) > get_mod_date(&build_ip_bin)
	{
		trace!("building IP.BIN");

		cmd!(&wrap_error, format!("{YAUL_INSTALL_ROOT}/bin/make-ip"),
			build_program_bin.display().to_string(),
			ip_version,
			ip_release_date,
			ip_areas,
			ip_peripherals,
			format!("'{ip_title}'"),
			format!("0x{ip_main_stack_addr:0x}"),
			format!("0x{ip_sub_stack_addr:0x}"),
			format!("0x{ip_1st_read_addr:0x}"),
			format!("0x{ip_1st_read_size:0x}"),
		).run()?;
	}

	trace!("attempting iso build: ip.bin({:?}) > {sh_program}.iso({:?}) || {sh_program}.bin({:?}) > {sh_program}.iso({:?})",
		get_mod_date(&build_ip_bin), get_mod_date(&out_program_iso), get_mod_date(&build_program_bin), get_mod_date(&out_program_iso));
	let mut is_iso_built = false;
	if get_mod_date(&build_ip_bin) > get_mod_date(&out_program_iso)
	|| get_mod_date(&build_program_bin) > get_mod_date(&out_program_iso)
	{
		trace!("building {sh_program}.iso");

		cmd!("mkdir", "-p", dir_image.display().to_string()).run()?;

		cmd!("cp",
			build_program_bin.display().to_string(),
			format!("{}/{image_1st_read_bin}", dir_image.display()),
		).run()?;

		for txt in ["ABS.TXT", "BIB.TXT", "CPY.TXT"] {
			match std::fs::exists(&format!("{}/{txt}", dir_image.display())) {
				Ok(false) => {
					cmd!("printf", "--", "empty").stdout_path(format!("{}/{txt}", dir_image.display())).run()?;
				}
				Ok(true) => {}
				Err(e) => error!("{e}"),
			}
		}

		cmd!(&wrap_error, format!("{YAUL_INSTALL_ROOT}/bin/make-iso"),
			dir_image,
			build_ip_bin,
			sh_output_path.display().to_string(),
			sh_program,
		).run()?;

		is_iso_built = true;
	}

	trace!("attempting cue build");
	let cue_file = [ sh_output_path.clone(), sh_program.into() ].iter()
		.collect::<PathBuf>()
		.with_extension("cue");
	if !std::fs::exists(&cue_file)? || is_iso_built {
		trace!("building {}", cue_file.display());

		cmd!("mkdir", "-p", dir_audio.display().to_string()).run()?;

		cmd!(&wrap_error, format!("{YAUL_INSTALL_ROOT}/bin/make-cue"),
			dir_audio.display().to_string(),
			out_program_iso,
		).run()?;
	}
	
	Ok(())
}
