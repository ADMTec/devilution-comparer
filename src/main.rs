use clap::{App, Arg};
use notify::{DebouncedEvent, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::time::Duration;

mod corelogic;
use self::corelogic::CoreError::*;
use self::corelogic::{run_compare, Opts};

fn main() {
    let cmdline = App::new("devilution-comparer")
        .about(
            "Generates orig.asm and compare.asm in the current working directory. \
             Finds the function specified in the devilution binary, disassembles it, \
             then disassembles the original binary with the same length at the specified offset. \
             The disassembled original code will be written into orig.asm, the devilution code \
             into compare.asm.

             Note that the disassembler will use the function offset read from the PDB for both \
             decompilations in order to align the addresses in the output files \
             (including relative jumps).",
        ).arg(
            Arg::with_name("DIABLO_FILE")
                .help("Path to the original Diablo.exe to use")
                .required(true),
        ).arg(
            Arg::with_name("DEVILUTION_FILE")
                .help(
                    "Sets the debug binary file to use. \
                     The respective .pdb file needs to exist in the same folder as well. \
                     Currently for files generated by VC6 only.",
                ).required(true),
        ).arg(
            Arg::with_name("DIABLO_OFFSET_START")
                .help("Offset into the original file, decimal or hex number (0xDEADBEEF)")
                .required(true)
                .validator(is_vaild_number),
        ).arg(
            Arg::with_name("DEBUG_SYMBOL")
                .help(
                    "Function name/debug symbol to compare. This also defines the length \
                     of code in the original file to compare to.",
                ).required(true),
        ).arg(Arg::with_name("watch").short("w").long("watch").help(
            "Enable watching for changes to the PDB file, updating the output files \
             on change.",
        )).arg(
            Arg::with_name("noaddr")
                .long("no-addr")
                .help("Removes the leading addresses from the output."),
        ).get_matches();

    let compare_file_path: PathBuf = cmdline.value_of_os("DEVILUTION_FILE").unwrap().into();
    let compare_pdb_file = compare_file_path.with_extension("pdb");

    let opts = Opts {
        orig: cmdline.value_of_os("DIABLO_FILE").unwrap().into(),
        compare_file_path,
        compare_pdb_file,
        orig_offset_start: cmdline
            .value_of("DIABLO_OFFSET_START")
            .map(|s| parse_offset(s).unwrap())
            .unwrap(),
        debug_symbol: cmdline.value_of("DEBUG_SYMBOL").unwrap().into(),
        print_adresses: !cmdline.is_present("noaddr"),
        last_offset_length: None,
        enable_watcher: cmdline.is_present("watch"),
    };

    if let Err(e) = watch(opts) {
        println!("Error: {:?}", e)
    }
}

fn watch(mut opts: Opts) -> notify::Result<()> {
    // initial run
    run_disassemble(&mut opts);

    if !opts.enable_watcher {
        return Ok(());
    }

    let (tx, rx) = channel();

    let mut watcher: RecommendedWatcher = Watcher::new(tx, Duration::from_secs(2))?;

    watcher.watch(&opts.compare_pdb_file, RecursiveMode::NonRecursive)?;
    println!(
        "Started watching {} for changes. CTRL+C to quit.",
        opts.compare_pdb_file.to_string_lossy()
    );

    loop {
        match rx.recv() {
            Ok(DebouncedEvent::Create(_)) | Ok(DebouncedEvent::Write(_)) => {
                run_disassemble(&mut opts);
            }
            Err(e) => println!("Watch error: {:?}", e),
            _ => {}
        }
    }
}

fn run_disassemble(opts: &mut Opts) {
    match run_compare(&opts) {
        Ok((offset, length)) => {
            println!(
                "Found {} at offset: {}{}, length: {}{}",
                &opts.debug_symbol,
                format!("{:X}", offset),
                if let Some((old_offset, _)) = opts.last_offset_length {
                    format!(" ({:+X})", offset - old_offset)
                } else {
                    "".into()
                },
                format!("{:X}", length),
                if let Some((_, old_length)) = opts.last_offset_length {
                    format!(" ({:+X})", length - old_length)
                } else {
                    "".into()
                },
            );

            opts.last_offset_length = Some((offset, length));
        }
        Err(e) => match e {
            CvDumpFail(e) => println!("CvDump.exe error: {:?}", e),
            CvDumpUnsuccessful => println!("CvDump exited with errorcode != 0."),
            SymbolNotFound => println!("Symbol not found in the pdb."),
            IoError(e) => println!("IO error: {:?}", e),
            CapstoneError(e) => println!("Capstone disassembly engine error: {:?}", e),
        },
    };
}

fn is_vaild_number(v: String) -> Result<(), String> {
    parse_offset(&v)
        .map(|_| ())
        .map_err(|_| "Argument has to be a decimal or hex (0xDEADBEEF) number.".into())
}

fn parse_offset(v: &str) -> Result<u64, std::num::ParseIntError> {
    if v.starts_with("0x") {
        u64::from_str_radix(&v[2..], 16)
    } else {
        u64::from_str_radix(&v, 10)
    }
}
