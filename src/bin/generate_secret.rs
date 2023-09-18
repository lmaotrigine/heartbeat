use base64ct::{Base64Url, Encoding};
use rand::{thread_rng, RngCore};

const NUM_BYTES: usize = 48;
const STR_LEN: usize = 64;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let mut buf = [0u8; NUM_BYTES];
    thread_rng().fill_bytes(&mut buf);
    let mut s = [0u8; STR_LEN];
    println!("{}", Base64Url::encode(&buf, &mut s).map_err(color_eyre::Report::msg)?);
    Ok(())
}
