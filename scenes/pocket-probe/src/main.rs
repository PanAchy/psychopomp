//! Probe: one line narrated by the local Pocket TTS server, carried by the plan.
//!
//! Run it with a server up (`pocket-tts serve`), then render the plan it writes.
//! The audio is placed into the Scene Plan, so `plan render` muxes it without a
//! separate FFmpeg step. Deleting `media.lock.json` and editing the line
//! re-narrates.

use std::{fs, path::PathBuf};

use anyhow::Result;
use psychopomp::{
    author::{PlanBuilder, SECOND},
    stage::{StageActor, StageElement, StagePlan},
};
use psychopomp_media::{Media, Voice};

fn main() -> Result<()> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let media = Media::open(&root)?;
    let said = media.say(
        "probe",
        &Voice::pocket("alba"),
        "A card drifts into place, a wire draws on, and a packet flies to the server.",
    )?;
    media.finish()?;
    println!("{}", psychopomp_media::show(&root)?);

    let plan = StagePlan {
        post: Default::default(),
        elements: vec![
            StageElement::card("client", [560.0, 540.0, 0.0], [300.0, 110.0], "client"),
            StageElement::orb("server", [1360.0, 540.0, 0.0], 140.0),
            StageElement::beam("link", "client", "server"),
            StageElement::packet("hello", "link").labeled("GET /hello"),
        ],
    };
    let mut scene = PlanBuilder::new("pocket-probe", 6 * SECOND);
    let mut stage = StageActor::declare(&mut scene, "stage", &plan)?;
    let ready = stage.settle_in(&mut scene, "client", 0);
    let wired = stage.connect(&mut scene, "link", ready, 0.6);
    let landed = stage.send(&mut scene, "hello", wired + SECOND / 2, 0.8);
    stage.land(&mut scene, "server", landed);
    stage.jolt(&mut scene, landed, [1.0, 0.0], 0.6);
    said.place(&mut scene, SECOND / 4);

    fs::create_dir_all("target")?;
    fs::write(
        "target/pocket-probe.json",
        serde_json::to_string_pretty(&scene.finish()?)?,
    )?;
    Ok(())
}
