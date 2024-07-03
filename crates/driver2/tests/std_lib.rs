use fe_driver2::DriverDataBase;
use library::std_lib_input_ingot;
#[test]
fn check_std_lib() {
    let mut driver = DriverDataBase::default();
    // let std_ingot = library::std_lib_input_ingot(&mut driver);
    // driver.run_on_ingot(std_ingot);                                  What do you expect from this function ?? Is there an example of a driver run ??
    library::std_src_files(); // This is POC that the std dir is calling correctly
    let diags = driver.format_diags();
    if !diags.is_empty() {
        panic!("{diags}")
    }
}
