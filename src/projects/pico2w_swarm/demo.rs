#![cfg_attr(all(target_arch = "arm", target_os = "none"), no_std)]
#![cfg_attr(all(target_arch = "arm", target_os = "none"), no_main)]

#[cfg(all(target_arch = "arm", target_os = "none"))]
mod runtime;

#[cfg(all(target_arch = "arm", target_os = "none"))]
const SWARM_KERNEL_ROLE: runtime::SwarmKernelRole = runtime::SwarmKernelRole::Dynamic;

#[cfg(not(all(target_arch = "arm", target_os = "none")))]
fn main() {}
