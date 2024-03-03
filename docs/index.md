# RustBag
<img src="./assets/logo.png" alt="logo" width="20%"/>

RustBag is a yet another ROSBag reader. It's main feature is the ability to read directly from an Object Storage, like AWS S3.
Additionally, it is *fast*, with speeds somewhere in between [rosbags](https://ternaris.gitlab.io/rosbags/) and [embag](https://github.com/embarktrucks/embag).

!!! warning

    RustBag is still in development.
    While the basic functionality (reading bags) is provided, there are still feature, documentation, and performance improvements that we want to add before making a full release.

## Getting Started
See [Getting Started page](./getting_started.md)

## Issues
If you have a feature request, or a suggestion please open an [issue](https://github.com/balbok0/rustbag/issues).

## Related Projects

While I do hope that RustBag can be widely adopted, here are alternative bag readers that perform well in my opinion:

* [embag](https://github.com/embarktrucks/embag) - By far the fastest local rosbag reading library. Warning: might be stale and it does suffer from some memory leaks that I have not been able to identify, when working with many bags.
* [rosbags](https://ternaris.gitlab.io/rosbags/) - Probably the most stable/bug-free implementation of rosbag reading. Supports many robotics data formats. It's on little slower side when working with remote bags, but still good experience.

Other ROS + Rust projects that were inspiration for this code:

* [rosrust](https://github.com/adnanademovic/rosrust) - Specifically *ros_message* crate, which *ros_msg* crate in this repo is loosely based off of (it's really a mix of this crate and embag parser).
* [rosbag-rs](https://github.com/SkoltechRobotics/rosbag-rs) - Pure Rust ROSBag reader. Does not appear finished, but also influenced overall design of code.
