fn main() {
    #[cfg(not(any(feature = "default", feature = "tui-mode", feature = "gui-mode")))]
    println!("cargo:warning=Building without at least one of the features \"default\", \"tui-mode\" or \"gui-mode\" will result in a useless binary");
}
