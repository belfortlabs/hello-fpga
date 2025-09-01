use std::io::Result;

use fhe_aes::demo::app::App;

fn main() -> Result<()> {
    let mut app = App::init()?;
    app.run()?;
    Ok(())
}
