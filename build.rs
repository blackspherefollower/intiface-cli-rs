use vergen::{ConstantsFlags, generate_cargo_keys};

fn main() {
  prost_build::compile_protos(&["src/IntifaceGui.proto"],
                              &["src/"]).unwrap();
  // Setup the flags, toggling off the 'SEMVER_FROM_CARGO_PKG' flag
  let mut flags = ConstantsFlags::all();
  flags.toggle(ConstantsFlags::SEMVER_FROM_CARGO_PKG);
  flags.toggle(ConstantsFlags::REBUILD_ON_HEAD_CHANGE);

  // Generate the 'cargo:' key output
  generate_cargo_keys(flags).expect("Unable to generate the cargo keys!");
}