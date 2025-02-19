# AMD GPU plugin

Allows to measure AMD GPU hardware metrics with the ROCm software and AMD SMI library.

## Table of Contents

- [Prepare your environment](#prepare-environment)
- [How to use](#how-to-use)

### Prepare environment

For the integration of AMD GPU plugin in ALUMET, we must using the Rust interface provided by the AMD SMI library (https://github.com/ROCm/amdsmi/tree/amd-mainline/rust-interface). However, we don't currently have an installable and usable Rust crate on https://crates.io/ for compilation, like any library in the project. So to compile this plugin like any other, we need to place ourselves in the **agent** directory from the ALUMET github repository, and also integrate and install first the AMD SMI library on the machine that compiling ALUMET, to do that you can follow the command lines bellow :

```bash
apt-get update && apt-get install -y apt-utils libdrm-dev cmake
cd ~/
git clone https://github.com/ROCm/amdsmi.git && cd amdsmi/ && mkdir build/
cmake .. && make -j$(nproc) && make install
```

### How to use

After the installation succeed, we can compiling ALUMET to generate the usable **alumet-agent** binary file.

```bash
cd alumet/agent/
cargo build --release -p "alumet-agent" --bins --target="x86_64-unknown-linux-gnu" --all-features
```

The binary was finally created and is located in your ALUMET repository in "~/alumet/target/<your_target>/release/<binary-name>" folder.
To start ALUMET, we need to install amd-smi on system, and just run the binary **alumet-agent**.

```bash
apt-get install amd-smi-lib
```

You can see now the result of collected metrics stored by default in the **alumet-output.csv** file.
