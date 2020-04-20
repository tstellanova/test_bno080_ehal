fn main() {
    #[cfg(any(
        feature = "stm32f3x",
        feature = "stm32f4x",
        feature = "stm32h7x"
    ))]
    {
        use std::env;
        use std::fs::File;
        use std::io::Write;
        use std::path::PathBuf;
        #[cfg(feature = "stm32h7x")]
        let memfile_bytes = include_bytes!("stm32h743_memory.x");

        #[cfg(feature = "stm32f4x")]
        let memfile_bytes = include_bytes!("stm32f401_memory.x");

        #[cfg(feature = "stm32f3x")]
        let memfile_bytes = include_bytes!("stm32f303_memory.x");

        // Put the linker script somewhere the linker can find it
        let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());

        File::create(out.join("memory.x"))
            .unwrap()
            .write_all(memfile_bytes)
            .unwrap();
        println!("cargo:rustc-link-search={}", out.display());
    }
}
