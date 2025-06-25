// Build script for analytics_server
//
// Generates protobuf message types for analytics events
// Uses the unified protocol library from fechatter_protos

use std::io::Result;

fn main() -> Result<()> {
  // Generate protobuf message types only (no gRPC)
  prost_build::Config::new()
    .out_dir("src/pb")
    .compile_protos(
      &["../fechatter_protos/fechatter/v1/analytics.proto"], 
      &["../fechatter_protos/"]
    )?;
  
  println!("cargo:rerun-if-changed=../fechatter_protos/fechatter/v1/analytics.proto");
  
  Ok(())
}
