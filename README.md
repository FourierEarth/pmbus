# Power Management Bus over I2c Interfaces

This crate is a work-in-progress. Contributions subject to review are welcome.
Feedback will be ignored until feature-completeness.

The bulk of this crate is based on the specification
[Part II](https://pmbusprod.wpenginepowered.com/wp-content/uploads/2022/01/PMBus-Specification-Rev-1-3-1-Part-II-20150313.pdf).

Extensions and features for robustness can be found in
[Part I](https://pmbusprod.wpenginepowered.com/wp-content/uploads/2022/01/PMBus-Specification-Rev-1-3-1-Part-I-20150313.pdf).

## Capabilities

- [ ] Generic `SmBus` wrapper trait compatible with `embedded-hal`.
- [x] Generic `SmBus` wrapper trait compatible with `embedded-hal-async`.
- [x] Write and send commands.
- [ ] SMBus alert interface (`SMBALERT#`).
- [ ] Read and process call commands.
- [ ] Strong type wrappers and deserialization for well-defined bit-fields.
- [ ] Packet Error Checking.
- [ ] Extended Commands.
- [ ] Interface for manufacturer specific commands.
- [ ] Interface for manufacturer specific data payloads.
- [ ] More...

# Copyright

This repository and all code herein is owned wholly and exclusively by Fourier Earth Incorporated.

> Original author & code owner: Jacob Birkett (`jacob_birkett (at) fourier (dot) earth`).

```
Copyright (C) 2024, Fourier Earth Inc.
All rights reserved.
```
