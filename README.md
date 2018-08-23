# Devilution-comparer

Small binary comparison helper tool for devilution.

Generates an orig.asm and a compare.asm in the current directory and can watch the respective *.pdb for changes.

Use `--help` for parameter info.

Example call:

```plain
devilution-comparer -w --no-addr path\to\Diablo_orig.exe devilution\bld\Diablo.exe 0x303EF InitMonsterTRN
```

## Requirements

This uses Rust in the 2018 edition (so currently nightly only).

Since the `pdb` crate doesn't support the old PDB file format generated by VC++ < 7 yet,
you will need to put `cvdump.exe` from https://github.com/Microsoft/microsoft-pdb/tree/master/cvdump
into the folder of the binary. If you aren't on windows, this tool tries to run `wine cvdump.exe` instead.