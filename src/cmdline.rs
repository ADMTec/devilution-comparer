use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

use super::{Command, CompareCommandInfo, CompareOpts, DisasmOpts, GenerateFullCommandInfo};

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn parse_cmdline() -> Command {
    let diablo_file = Arg::with_name("DIABLO_FILE")
        .help("Path to the original Diablo.exe to use")
        .required(true);

    let devilution_file = Arg::with_name("DEVILUTION_FILE")
        .help(
            "Sets the debug binary file to use. \
             The respective .pdb file needs to exist in the same folder as well. \
             Currently for files generated by VC6 only.",
        )
        .required(true);

    let debug_symbol = Arg::with_name("DEBUG_SYMBOL")
        .help(
            "Function name/debug symbol to compare. This has to be defined for the original \
             binary in the comparer-config.toml. Is the size attribute missing, devilution-comparer \
             will use the size of the devilution function for the original binary as well.",
        ).required(true);

    let watch = Arg::with_name("watch").short("w").long("watch").help(
        "Enable watching for changes to the PDB file, updating the output files \
         on change.",
    );

    let show_ip = Arg::with_name("show-ip")
        .short("i")
        .long("show-ip")
        .help("Shows leading addresses in the output.")
        .global(true);

    let no_mem_disp = Arg::with_name("no-mem-disp")
        .long("no-mem-disp")
        .help(
            "Hide memory displacements and indirect calls. This cleans up the output tremendously, \
             but can cause you to miss wrong stack variables or globals. Use only with caution.")
        .global(true);

    let no_imms = Arg::with_name("no-imms")
        .long("no-imms")
        .help("Hides all immediate values. Use with caution.")
        .global(true);

    let truncate_to_original = Arg::with_name("truncate-to-original")
        .long("truncate-to-original")
        .help(
            "Truncate the number bytes disassembled in the compared binary to the length of the \
             original function instead of the reported length in the pdb file.",
        )
        .global(true);

    let app = App::new("devilution-comparer")
        .setting(AppSettings::SubcommandsNegateReqs)
        .version(VERSION)
        .about(
            "Generates orig.asm and compare.asm in the current working directory. \
             Finds the function specified in the devilution binary, disassembles it, \
             then disassembles the original binary with the same length at the specified offset. \
             The disassembled original code will be written into orig.asm, the devilution code \
             into compare.asm.\n\nNote that the disassembler will use the function offset read \
             from the PDB for both decompilations in order to align the addresses in the output files \
             (including relative jumps).",
        )
        .arg(diablo_file)
        .arg(devilution_file)
        .arg(debug_symbol)
        .arg(watch)
        .arg(show_ip)
        .arg(no_mem_disp)
        .arg(no_imms)
        .arg(truncate_to_original)
        .subcommand(SubCommand::with_name("generate-full")
            .about("Generates a disassembly file with all functions defined in comparer-config.toml.")
            .arg(
                Arg::with_name("FILE")
                    .required(true)
                    .help("The file to generate the disassembly output for.")
                    .validator_os(file_exists)
            )
            .arg(
                Arg::with_name("orig-file")
                    .long("orig-file")
                    .help(
                        "Generate the file for the original binary for all functions defined within \
                         comparer-config.toml, skipping functions without defined sizes.")
            ));

    let matches = &app.get_matches();

    if let Some(matches) = matches.subcommand_matches("generate-full") {
        Command::GenerateFull(parse_generate_full_args(&matches))
    } else {
        Command::Compare(parse_compare_args(&matches))
    }
}

fn parse_compare_args(matches: &ArgMatches) -> CompareCommandInfo {
    let compare_file_path: PathBuf = matches.value_of_os("DEVILUTION_FILE").unwrap().into();
    let compare_pdb_file = compare_file_path.with_extension("pdb");

    CompareCommandInfo {
        compare_opts: CompareOpts {
            orig: matches.value_of_os("DIABLO_FILE").unwrap().into(),
            compare_file_path,
            compare_pdb_file,
            debug_symbol: matches.value_of("DEBUG_SYMBOL").unwrap().into(),
        },
        disasm_opts: parse_disasm_opts(&matches),
        enable_watcher: matches.is_present("watch"),
        last_offset_size: None,
        truncate_to_original: matches.is_present("truncate-to-original"),
    }
}

fn parse_generate_full_args(matches: &ArgMatches) -> GenerateFullCommandInfo {
    GenerateFullCommandInfo {
        file_path: matches.value_of_os("FILE").unwrap().into(),
        orig_file: matches.is_present("orig-file"),
        disasm_opts: parse_disasm_opts(&matches),
        truncate_to_original: matches.is_present("truncate-to-original"),
    }
}

fn parse_disasm_opts(matches: &ArgMatches) -> DisasmOpts {
    DisasmOpts {
        print_adresses: matches.is_present("show-ip"),
        show_mem_disp: !matches.is_present("no-mem-disp"),
        show_imms: !matches.is_present("no-imms"),
    }
}

// #[allow(unknown_lints)]
// #[allow(needless_pass_by_value)] // clap returns an owned string
// fn is_vaild_number(v: String) -> Result<(), String> {
//     parse_hex(&v)
//         .map(|_| ())
//         .map_err(|_| "Argument has to be a decimal or hex (0xDEADBEEF) number.".into())
// }

// fn parse_hex(v: &str) -> Result<u64, std::num::ParseIntError> {
//     if v.starts_with("0x") {
//         u64::from_str_radix(&v[2..], 16)
//     } else {
//         u64::from_str_radix(&v, 10)
//     }
// }

fn file_exists(path: &OsStr) -> Result<(), OsString> {
    let p = Path::new(path);
    if p.exists() && p.is_file() {
        Ok(())
    } else {
        Err(OsString::from("The file specified does not exist"))
    }
}
