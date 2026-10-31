//! Probe: one line narrated by the local Pocket TTS server.
//!
//! Run it with a server up (`pocket-tts serve`), then read `media.lock.json`
//! beside this file. Deleting the lock and editing the line re-narrates.

use std::{fs, path::PathBuf};

use anyhow::Result;
use psychopomp_media::{Media, Voice};

fn main() -> Result<()> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let media = Media::open(&root)?;
    let voice = Voice::pocket("alba");
    media.say(
        "probe",
        &voice,
        "This line is spoken by the local Pocket TTS server.",
    )?;
    media.finish()?;
    println!("{}", psychopomp_media::show(&root)?);
    for entry in fs::read_dir(root.join("media"))? {
        println!("file: {:?}", entry?.file_name());
    }
    Ok(())
}
